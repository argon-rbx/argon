use crossbeam_channel::select;
use log::{debug, error, info, trace, warn};
use rbx_dom_weak::types::Ref;
use std::{
	sync::{Arc, Mutex},
	thread::Builder,
};

use super::{
	changes::{Changes, UpdatedSnapshot},
	meta::SourceKind,
	queue::Queue,
	snapshot::Snapshot,
	tree::Tree,
};
use crate::{
	argon_error, lock, messages,
	middleware::{new_snapshot, project::snapshot_node},
	project::{Project, ProjectDetails},
	stats, util,
	vfs::{Vfs, VfsEvent},
	BLACKLISTED_PATHS,
};

pub struct Processor {}

impl Processor {
	pub fn new(queue: Arc<Queue>, tree: Arc<Mutex<Tree>>, vfs: Arc<Vfs>, project: Arc<Mutex<Project>>) -> Self {
		let handler = Arc::new(Handler {
			queue,
			tree,
			vfs: vfs.clone(),
			project,
		});

		let handler = handler.clone();

		Builder::new()
			.name("processor".to_owned())
			.spawn(move || {
				let vfs_receiver = vfs.receiver();

				loop {
					select! {
						recv(vfs_receiver) -> event => {
							handler.on_vfs_event(event.unwrap());
						}
					}
				}
			})
			.unwrap();

		Self {}
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
				changes.extend(process_changes(id, &mut tree, &self.vfs));
			}

			changes
		};

		if !changes.is_empty() {
			stats::files_synced(changes.len() as u32);

			let result = self.queue.push(messages::SyncChanges(changes), None);

			match result {
				Ok(()) => trace!("Added changes to the queue"),
				Err(err) => {
					error!("Failed to add changes to the queue: {}", err);
				}
			}
		}
	}
}

#[profiling::function]
fn process_changes(id: Ref, tree: &mut Tree, vfs: &Vfs) -> Changes {
	profiling::start_frame!();
	trace!("Processing changes for instance: {:?}", id);

	let mut changes = Changes::new();

	let meta = tree.get_meta(id).unwrap();
	let source = meta.source.get();

	let snapshot = match source {
		SourceKind::Project(path, name, node) => match snapshot_node(name, path, &meta.context, vfs, node.clone()) {
			Ok(snapshot) => Some(snapshot),
			Err(err) => {
				error!("Failed to process changes: {}, source: {:?}", err, source);
				return changes;
			}
		},
		SourceKind::Path(path) => match new_snapshot(path, &meta.context, vfs) {
			Ok(snapshot) => snapshot,
			Err(err) => {
				error!("Failed to process changes: {}, source: {:?}", err, source);
				return changes;
			}
		},
	};

	// Handle additions, modifications and child removals
	if let Some(snapshot) = snapshot {
		process_child_changes(id, snapshot, &mut changes, tree);
	// Handle regular removals
	} else {
		tree.remove_instance(id);
		changes.remove(id);
	}

	changes
}

fn process_child_changes(id: Ref, mut snapshot: Snapshot, changes: &mut Changes, tree: &mut Tree) {
	// Update meta if it's different
	match (snapshot.meta, tree.get_meta(id)) {
		(snapshot_meta, Some(meta)) => {
			if snapshot_meta != *meta {
				tree.update_meta(id, snapshot_meta);
			}
		}
		(snapshot_meta, None) => {
			tree.insert_meta(id, snapshot_meta);
		}
	}

	// Process instance changes

	let instance = tree.get_instance_mut(id).unwrap();

	let mut updated_snapshot = UpdatedSnapshot::new(id);

	updated_snapshot.name = if snapshot.name != instance.name {
		instance.name = snapshot.name.clone();
		Some(snapshot.name)
	} else {
		None
	};

	updated_snapshot.class = if snapshot.class != instance.class {
		instance.class = snapshot.class.clone();
		Some(snapshot.class)
	} else {
		None
	};

	updated_snapshot.properties = if snapshot.properties != instance.properties {
		instance.properties = snapshot.properties.clone();
		Some(snapshot.properties)
	} else {
		None
	};

	if !updated_snapshot.is_empty() {
		// Track `lines_synced` stat
		if let Some(properties) = &updated_snapshot.properties {
			let loc = util::count_loc_from_properties(properties);
			stats::lines_synced(loc as u32);
		}

		changes.update(updated_snapshot);
	}

	let mut hydrated = vec![false; snapshot.children.len()];

	// Pair instances and find removed children
	#[allow(clippy::unnecessary_to_owned)]
	for child_id in instance.children().to_owned() {
		let instance = tree.get_instance(child_id).unwrap();

		let snapshot = snapshot.children.iter_mut().enumerate().find(|(index, child)| {
			if hydrated[*index] {
				return false;
			}

			if child.name == instance.name && child.class == instance.class {
				hydrated[*index] = true;
				return true;
			}

			false
		});

		if let Some((_, child)) = snapshot {
			child.set_id(child_id);
		} else {
			tree.remove_instance(child_id);
			changes.remove(child_id);
		}
	}

	// Process child changes and find new children
	for child in snapshot.children {
		if child.id.is_some() {
			process_child_changes(child.id, child, changes, tree);
		} else {
			let mut child = child;

			insert_children(&mut child, id, tree);

			// Track `lines_synced` stat
			let loc = util::count_loc_from_properties(&child.properties);
			stats::lines_synced(loc as u32);

			changes.add(child, id);
		}
	}
}

fn insert_children(snapshot: &mut Snapshot, parent: Ref, tree: &mut Tree) {
	let referent = tree.insert_instance_non_recursive(snapshot.clone(), parent);

	snapshot.set_id(referent);

	for child in snapshot.children.iter_mut() {
		insert_children(child, referent, tree);
	}
}
