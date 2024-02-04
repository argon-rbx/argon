use crossbeam_channel::{select, Receiver, Sender};
use log::error;
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
	lock,
	middleware::new_snapshot,
	vfs::{Vfs, VfsEvent},
};

const BLACKLISTED_PATHS: [&str; 1] = [".DS_Store"];

pub struct Processor {
	callback: Receiver<()>,
}

impl Processor {
	pub fn new(queue: Arc<Mutex<Queue>>, tree: Arc<Mutex<Tree>>, vfs: Arc<Vfs>) -> Self {
		let (sender, receiver) = crossbeam_channel::unbounded();

		let handler = Arc::new(Handler {
			queue: queue.clone(),
			tree: tree.clone(),
			vfs: vfs.clone(),
			callback: sender,
		});

		{
			let handler = handler.clone();

			Builder::new()
				.name("processor".into())
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
		}

		Self { callback: receiver }
	}

	pub fn callback(&self) -> Receiver<()> {
		self.callback.clone()
	}
}

struct Handler {
	queue: Arc<Mutex<Queue>>,
	tree: Arc<Mutex<Tree>>,
	vfs: Arc<Vfs>,
	callback: Sender<()>,
}

impl Handler {
	fn on_vfs_event(&self, event: VfsEvent) {
		let mut tree = lock!(self.tree);
		let mut queue = lock!(self.queue);

		let changes = match event {
			VfsEvent::Create(path) | VfsEvent::Write(path) | VfsEvent::Delete(path) => {
				if BLACKLISTED_PATHS.iter().any(|blacklisted| path.ends_with(blacklisted)) {
					return;
				}

				let ids = {
					let mut current_path = path.as_path();

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
			}
		};

		if !changes.is_empty() {
			self.callback.send(()).unwrap();

			println!("{:#?}", changes);

			// TODO: add to the queue here
		}
	}
}

#[profiling::function]
fn process_changes(id: Ref, tree: &mut Tree, vfs: &Vfs) -> Changes {
	profiling::start_frame!();

	let mut changes = Changes::new();

	let path = tree.get_path(id).unwrap();
	let meta = meta_from_vec(tree.get_meta(id));

	let snapshot = match new_snapshot(path, &meta, vfs) {
		Ok(snapshot) => snapshot,
		Err(err) => {
			error!("Failed to create snapshot: {}, path: {:?}", err, path);
			return changes;
		}
	};

	// Handle additions, modifications and removals
	// of instances with child source or data
	if let Some(snapshot) = snapshot {
		process_child_changes(id, snapshot, &mut changes, tree);
	// Handle removals of regular instances
	} else {
		tree.remove(id);
		changes.remove(id);
	}

	changes
}

// TODO: should be recursive in case of nested snapshots like projects or models
fn process_child_changes(id: Ref, mut snapshot: Snapshot, chnages: &mut Changes, tree: &mut Tree) {
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

	// Find removed children
	#[allow(clippy::unnecessary_to_owned)]
	for child_id in instance.children().to_owned() {
		// We only care about instances with path as other ones
		// can only be modified from the project or model files
		// which are guaranteed to have path assigned
		if let Some(path) = tree.get_path(child_id) {
			let mut exists = false;

			for child in snapshot.children.iter_mut() {
				if child.path == Some(path.to_owned()) {
					child.set_id(child_id);

					exists = true;
					break;
				}
			}

			if !exists {
				tree.remove(child_id);
				chnages.remove(child_id);
			}
		}
	}

	// Find new children
	for child in snapshot.children {
		if child.id.is_none() {
			let child_id = tree.insert(child.clone(), id);
			let child = child.clone().with_id(child_id);

			chnages.add(child);
		}
	}
}

fn meta_from_vec(meta: VecDeque<&Meta>) -> Meta {
	meta.into_iter().fold(Meta::new(), |mut acc, meta| {
		acc.extend(meta.clone());
		acc
	})
}
