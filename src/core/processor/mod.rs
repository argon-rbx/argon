use anyhow::Result;
use colored::Colorize;
use crossbeam_channel::{select, Sender};
use log::{debug, error, info, trace, warn};
use std::{
	sync::{Arc, Mutex},
	thread::Builder,
	time::{Duration, Instant},
};

use super::{changes::Changes, queue::Queue, snapshot::Snapshot, tree::Tree};
use crate::{
	argon_error,
	constants::{BLACKLISTED_PATHS, CHANGES_TRESHOLD},
	lock, logger, messages,
	project::{Project, ProjectDetails},
	stats,
	vfs::{Vfs, VfsEvent},
};

pub mod read;
pub mod write;

pub struct Processor {
	writer: Sender<Changes>,
}

impl Processor {
	pub fn new(queue: Arc<Queue>, tree: Arc<Mutex<Tree>>, vfs: Arc<Vfs>, project: Arc<Mutex<Project>>) -> Self {
		let handler = Arc::new(Handler {
			queue,
			tree,
			vfs: vfs.clone(),
			project,
		});

		let handler = handler.clone();
		let (sender, receiver) = crossbeam_channel::unbounded();

		Builder::new()
			.name("processor".to_owned())
			.spawn(move || {
				let mut last_client_event = Instant::now();

				let vfs_receiver = vfs.receiver();
				let client_receiver = receiver;

				loop {
					select! {
						recv(vfs_receiver) -> event => {
							if last_client_event.elapsed() > Duration::from_millis(200) {
								handler.on_vfs_event(event.unwrap());
							}
						}
						recv(client_receiver) -> changes => {
							handler.on_client_event(changes.unwrap());
							last_client_event = Instant::now();
						}
					}
				}
			})
			.unwrap();

		Self { writer: sender }
	}

	pub fn write(&self, changes: Changes) {
		self.writer.send(changes).unwrap();
	}

	pub fn write_all(&self, _snapshot: Snapshot) {
		// TODO
	}
}

struct Handler {
	queue: Arc<Queue>,
	tree: Arc<Mutex<Tree>>,
	vfs: Arc<Vfs>,
	project: Arc<Mutex<Project>>,
}

impl Handler {
	fn on_vfs_event(&self, event: VfsEvent) {
		trace!("Received VFS event: {:?}", event);

		let mut tree = lock!(self.tree);
		let path = event.path();

		let changes = {
			if BLACKLISTED_PATHS.iter().any(|blacklisted| path.ends_with(blacklisted)) {
				trace!("Processing of {:?} aborted: blacklisted", path);
				return;
			}

			if lock!(self.project).path == path {
				if let VfsEvent::Write(_) = event {
					debug!("Project file was modified. Reloading project..");

					match lock!(self.project).reload() {
						Ok(project) => {
							info!("Project reloaded");

							let details = messages::SyncDetails(ProjectDetails::from_project(project, &tree));

							match self.queue.push(details, None) {
								Ok(()) => trace!("Project details synced"),
								Err(err) => warn!("Failed to sync project details: {}", err),
							}
						}
						Err(err) => error!("Failed to reload project: {}", err),
					}
				} else if let VfsEvent::Delete(_) = event {
					argon_error!("Warning! Top level project file was deleted. This might cause unexpected behavior. Skipping processing of changes!");
					return;
				}
			}

			let ids = {
				let mut current_path = path;

				loop {
					if let Some(ids) = tree.get_ids(current_path) {
						break ids.to_owned();
					}

					match current_path.parent() {
						Some(parent) => current_path = parent,
						None => break vec![],
					}
				}
			};

			let mut changes = Changes::new();

			for id in ids {
				changes.extend(read::process_changes(id, &mut tree, &self.vfs));
			}

			changes
		};

		if !changes.is_empty() {
			stats::files_synced(changes.total() as u32);

			let result = self.queue.push(messages::SyncChanges(changes), None);

			match result {
				Ok(()) => trace!("Added changes to the queue"),
				Err(err) => {
					error!("Failed to add changes to the queue: {}", err);
				}
			}
		} else {
			trace!("No ID found for path {:?}", path);
		}
	}

	fn on_client_event(&self, changes: Changes) {
		trace!("Received client event: {:?} changes", changes.total());

		if changes.total() > CHANGES_TRESHOLD {
			let accept = logger::prompt(
				&format!(
					"You are about to apply {}, {} and {}. Do you want to continue?",
					format!("{} additions", changes.additions.len()).bold(),
					format!("{} updates", changes.updates.len()).bold(),
					format!("{} removals", changes.removals.len()).bold(),
				),
				true,
			);

			if !accept {
				trace!(
					"Aborted applying client event! {} changes were not applied",
					changes.total()
				);

				return;
			}
		}

		let mut tree = lock!(self.tree);

		let result = || -> Result<()> {
			for snapshot in changes.additions {
				write::apply_addition(snapshot, &mut tree, &self.vfs)?;
			}

			for snapshot in changes.updates {
				write::apply_update(snapshot, &mut tree, &self.vfs)?;
			}

			for id in changes.removals {
				write::apply_removal(id, &mut tree, &self.vfs)?;
			}

			Ok(())
		};

		match result() {
			Ok(()) => trace!("Changes applied successfully"),
			Err(err) => error!("Failed to apply changes: {}", err),
		}
	}
}
