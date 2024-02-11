use crossbeam_channel::{select, Receiver, Sender};
use log::{debug, error, trace};
use rbx_dom_weak::types::Ref;
use std::{
	collections::VecDeque,
	sync::{Arc, Mutex},
	thread::Builder,
};

use super::{
	change::{Changes, ModifiedSnapshot},
	meta::Meta,
	queue::Queue,
	snapshot::Snapshot,
	tree::Tree,
};
use crate::{
	argon_error, lock, messages,
	middleware::new_snapshot,
	project::Project,
	vfs::{Vfs, VfsEvent},
	BLACKLISTED_PATHS,
};

pub struct Processor {
	callback: Receiver<bool>,
}

impl Processor {
	pub fn new(queue: Arc<Mutex<Queue>>, tree: Arc<Mutex<Tree>>, vfs: Arc<Vfs>, project: Arc<Mutex<Project>>) -> Self {
		let (sender, receiver) = crossbeam_channel::unbounded();

		let handler = Arc::new(Handler {
			queue,
			tree,
			vfs: vfs.clone(),
			callback: sender,
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

		Self { callback: receiver }
	}

	pub fn callback(&self) -> Receiver<bool> {
		self.callback.clone()
	}
}

struct Handler {
	queue: Arc<Mutex<Queue>>,
	tree: Arc<Mutex<Tree>>,
	vfs: Arc<Vfs>,
	callback: Sender<bool>,
	project: Arc<Mutex<Project>>,
}

impl Handler {
	fn on_vfs_event(&self, event: VfsEvent) {
		let mut tree = lock!(self.tree);
		let mut queue = lock!(self.queue);

		let path = event.path();

		let changes = {
			if BLACKLISTED_PATHS.iter().any(|blacklisted| path.ends_with(blacklisted)) {
				trace!("Processing of {:?} aborted: blacklisted", path);
				return;
			}

			if lock!(self.project).path == path {
				if let VfsEvent::Write(_) = event {
					debug!("Project file was modified. Reloading project..");
					lock!(self.project).reload().ok();
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
			let path_changed = !changes.additions.is_empty() || !changes.removals.is_empty();

			self.callback.send(path_changed).unwrap();

			for snapshot in changes.additions {
				queue.push(
					messages::Create {
						parent: snapshot.id.unwrap(),
						name: snapshot.name,
						class: snapshot.class,
						properties: snapshot.properties,
					},
					None,
				)
			}

			for modified_snapshot in changes.modifications {
				queue.push(messages::Update(modified_snapshot), None);
			}

			for id in changes.removals {
				queue.push(messages::Remove { id }, None);
			}
		}
	}
}

#[profiling::function]
fn process_changes(id: Ref, tree: &mut Tree, vfs: &Vfs) -> Changes {
	profiling::start_frame!();
	trace!("Processing changes for instance: {:?}", id);

	let mut changes = Changes::new();

	let path = match tree.get_path(id) {
		Some(path) => path,
		None => {
			error!("Failed to get path for instance: {:?}", id);
			return changes;
		}
	};

	let meta = join_meta_entries(tree.get_meta_all(id));

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
		tree.remove(id);
		changes.remove(id);
	}

	changes
}

fn process_child_changes(id: Ref, mut snapshot: Snapshot, chnages: &mut Changes, tree: &mut Tree) {
	match (snapshot.meta, tree.get_meta_mut(id)) {
		(Some(snapshot_meta), Some(meta)) => {
			if snapshot_meta != *meta {
				*meta = snapshot_meta;
			}
		}
		(Some(snapshot_meta), None) => {
			tree.insert_meta(id, snapshot_meta);
		}
		_ => {}
	}

	let instance = tree.get_instance_mut(id).unwrap();

	let mut modified_snapshot = ModifiedSnapshot::new(id);

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
		chnages.modify(modified_snapshot);
	}

	let mut hydrated = vec![false; instance.children().len()];

	// Pair instances and find removed children
	#[allow(clippy::unnecessary_to_owned)]
	'outer: for child_id in instance.children().to_owned() {
		// Assign instances with known path to snapshot children
		if let Some(path) = tree.get_path(child_id) {
			if snapshot.children.is_empty() {
				tree.remove(child_id);
				chnages.remove(child_id);
			} else {
				for child in snapshot.children.iter_mut() {
					if child.path == Some(path.to_owned()) {
						child.set_id(child_id);

						continue 'outer;
					}
				}
			}

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
				tree.remove(child_id);
				chnages.remove(child_id);
			}
		}
	}

	// Process child changes and find new children
	for child in snapshot.children {
		if let Some(child_id) = child.id {
			process_child_changes(child_id, child, chnages, tree);
		} else {
			let child_id = tree.insert(child.clone(), id);
			let child = child.with_id(child_id);

			chnages.add(child);
		}
	}
}

fn join_meta_entries(meta: VecDeque<&Meta>) -> Meta {
	meta.into_iter().fold(Meta::new(), |mut acc, meta| {
		acc.extend(meta.clone());
		acc
	})
}
