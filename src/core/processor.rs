use crossbeam_channel::select;
use log::{debug, error, info, trace, warn};
use rbx_dom_weak::types::Ref;
use std::{
	path::PathBuf,
	sync::{Arc, Mutex},
	thread::Builder,
};

use super::{
	changes::{Changes, UpdatedSnapshot},
	meta::Meta,
	queue::Queue,
	snapshot::Snapshot,
	tree::Tree,
};
use crate::{
	argon_error,
	ext::PathExt,
	lock, messages,
	middleware::new_snapshot,
	project::Project,
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
						Ok(()) => info!("Project reloaded"),
						Err(err) => warn!("Failed to reload project: {}", err),
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

	let path = {
		let paths = tree.get_paths(id);

		if paths.is_empty() {
			error!("Failed to get path for instance: {:?}", id);
			return changes;
		// Get path of a regular file or super path of a child file
		} else {
			paths
				.iter()
				.fold(paths[0], |acc, path| if path.len() < acc.len() { path } else { acc })
		}
	};

	// Merge all meta entries associated with the given `id`
	let meta = tree.get_meta_all(id).into_iter().fold(Meta::new(), |mut acc, meta| {
		acc.extend(meta.clone());
		acc
	});

	let snapshot = match new_snapshot(path, &meta, vfs) {
		Ok(snapshot) => snapshot,
		Err(err) => {
			error!("Failed to process changes: {}, path", err);
			return changes;
		}
	};

	// Handle additions, modifications and
	// removals of instances without paths
	if let Some(snapshot) = snapshot {
		process_child_changes(id, snapshot, &mut changes, tree);
	// Handle removals of regular instances
	} else {
		tree.remove_instance(id);
		changes.remove(id);
	}

	changes
}

fn process_child_changes(id: Ref, mut snapshot: Snapshot, changes: &mut Changes, tree: &mut Tree) {
	// Update meta if it's different
	match (snapshot.meta, tree.get_meta_mut(id)) {
		(Some(snapshot_meta), Some(meta)) => {
			if snapshot_meta != *meta {
				if snapshot_meta.is_empty() {
					tree.remove_meta(id);
				} else {
					*meta = snapshot_meta;
				}
			}
		}
		(Some(snapshot_meta), None) => {
			tree.insert_meta(id, snapshot_meta);
		}
		_ => {}
	}

	// Update paths if they're different

	let paths: Vec<PathBuf> = tree.get_paths(id).into_iter().cloned().collect();

	for path in &paths {
		if !snapshot.paths.contains(path) {
			tree.remove_path(path, id);
		}
	}

	for path in &snapshot.paths {
		if !paths.contains(path) {
			tree.insert_path(path, id);
		}
	}

	// Process instance changes

	let instance = tree.get_instance_mut(id).unwrap();

	let mut modified_snapshot = UpdatedSnapshot::new(id);

	modified_snapshot.name = if snapshot.name != instance.name {
		instance.name = snapshot.name.clone();
		Some(snapshot.name)
	} else {
		None
	};

	modified_snapshot.class = if snapshot.class != instance.class {
		instance.class = snapshot.class.clone();
		Some(snapshot.class)
	} else {
		None
	};

	modified_snapshot.properties = if snapshot.properties != instance.properties {
		instance.properties = snapshot.properties.clone();
		Some(snapshot.properties)
	} else {
		None
	};

	if !modified_snapshot.is_empty() {
		changes.update(modified_snapshot);
	}

	let mut hydrated = vec![false; instance.children().len()];

	// Pair instances and find removed children
	#[allow(clippy::unnecessary_to_owned)]
	'outer: for child_id in instance.children().to_owned() {
		let paths = tree.get_paths(child_id);

		// Assign instances with known path to snapshot children
		if !paths.is_empty() {
			for child in snapshot.children.iter_mut() {
				if paths.iter().any(|path| child.paths.contains(path)) {
					child.set_id(child_id);

					continue 'outer;
				}
			}

			// Skip instances that are part of the project
			// but have different paths
			// if let Some(meta) = tree.get_meta(id) {
			// 	if let Some(project_data) = &meta.project_data {
			// 		if project_data.affects.exists() {
			// 			continue 'outer;
			// 		}
			// 	}
			// }

			tree.remove_instance(child_id);
			changes.remove(child_id);

		// Hydrate instances without path by their name and class
		} else {
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
	}

	// Process child changes and find new children
	for child in snapshot.children {
		if let Some(child_id) = child.id {
			process_child_changes(child_id, child, changes, tree);
		} else {
			let mut child = child;

			insert_children(&mut child, id, tree);

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
