use anyhow::Result;
use colored::Colorize;
use crossbeam_channel::{select, Sender};
use log::{debug, error, info, trace, warn};
use serde::Deserialize;
use std::{
	sync::{Arc, Mutex},
	thread::Builder,
};

use super::{changes::Changes, queue::Queue, tree::Tree};
use crate::{
	argon_error,
	constants::{BLACKLISTED_PATHS, CHANGES_THRESHOLD},
	lock, logger,
	project::{Project, ProjectDetails},
	server, stats,
	vfs::{Vfs, VfsEvent},
};

mod helpers;

pub mod read;
pub mod write;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WriteRequest {
	pub changes: Changes,
	pub client_id: u32,
}

pub struct Processor {
	writer: Sender<WriteRequest>,
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
			.spawn(move || -> Result<()> {
				let vfs_receiver = vfs.receiver();
				let client_receiver = receiver;

				loop {
					select! {
						recv(vfs_receiver) -> event => {
							handler.on_vfs_event(event?);
						}
						recv(client_receiver) -> request => {
							vfs.pause();
							handler.on_client_event(request?);
							vfs.resume();
						}
					}
				}
			})
			.unwrap();

		Self { writer: sender }
	}

	pub fn write(&self, request: WriteRequest) {
		self.writer.send(request).unwrap();
	}
}

struct Handler {
	queue: Arc<Queue>,
	tree: Arc<Mutex<Tree>>,
	vfs: Arc<Vfs>,
	project: Arc<Mutex<Project>>,
}

impl Handler {
	#[profiling::function]
	fn on_vfs_event(&self, event: VfsEvent) {
		profiling::start_frame!();

		trace!("Received VFS event: {:?}", event);

		let mut tree = lock!(self.tree);
		let path = event.path();

		let changes = {
			if BLACKLISTED_PATHS.iter().any(|blacklisted| path.ends_with(blacklisted)) {
				trace!("Processing of {:?} aborted: blacklisted", path);
				return;
			}

			let ids = {
				let mut current_path = path;

				loop {
					if let Some(ids) = tree.get_ids(current_path) {
						break ids.to_owned();
					}

					match current_path.parent() {
						Some(parent) => current_path = parent,
						None => {
							trace!("No ID found for path {:?}", path);
							return;
						}
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

			let result = self.queue.push(server::SyncChanges(changes), None);

			match result {
				Ok(()) => trace!("Added changes to the queue"),
				Err(err) => {
					error!("Failed to add changes to the queue: {}", err);
				}
			}
		} else {
			trace!("No changes detected when processing path: {:?}", path);
		}

		if lock!(self.project).path == path {
			if let VfsEvent::Write(_) = event {
				debug!("Project file was modified. Reloading project..");

				match lock!(self.project).reload() {
					Ok(project) => {
						info!("Project reloaded");

						let details = server::SyncDetails(ProjectDetails::from_project(project, &tree));

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
	}

	#[profiling::function]
	fn on_client_event(&self, request: WriteRequest) {
		profiling::start_frame!();

		let changes = request.changes;
		let client_id = request.client_id;

		trace!("Received client event: {:?} changes", changes.total());

		if changes.total() > CHANGES_THRESHOLD {
			let accept = logger::prompt(
				&format!(
					"You are about to apply {}, {} and {}. Do you want to continue?",
					format!("{} additions", changes.additions.len()).bold().green(),
					format!("{} updates", changes.updates.len()).bold().blue(),
					format!("{} removals", changes.removals.len()).bold().red(),
				),
				true,
			);

			if !accept {
				trace!(
					"Aborted applying client event! {} changes were not applied",
					changes.total()
				);

				match self.queue.disconnect("Client and server got out of sync!", client_id) {
					Ok(()) => trace!("Client {} disconnected", client_id),
					Err(err) => warn!("Failed to disconnect client: {}", err),
				}

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
