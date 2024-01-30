use crossbeam_channel::{select, Receiver, Sender};
use log::error;
use rbx_dom_weak::{types::Ref, Instance};
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
	handler: Arc<Handler>,
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

		Self {
			callback: receiver,
			handler,
		}
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

		self.vfs.process_event(&event);

		let changed = match event {
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

				for id in ids {
					process_changes(id, &mut tree, &self.vfs);
				}

				true
			}
		};

		if changed {
			self.callback.send(()).unwrap();
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

	println!("{:#?}", snapshot);

	if let Some(snapshot) = snapshot {
		process_child_changes(id, snapshot, &mut changes, tree);
	} else {
		tree.remove(id);
		changes.remove(id);
	}

	println!("{:?}", "--------------------------");
	println!("{:?}", changes);

	changes
}

fn process_child_changes(id: Ref, snapshot: Snapshot, chnages: &mut Changes, tree: &mut Tree) {
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

	let snapshot_children = snapshot.children.len();
	let instance_children = instance.children().len();

	// println!("{:?}", snapshot_children);
	// println!("{:?}", instance_children);

	#[allow(clippy::unnecessary_to_owned)]
	for child in instance.children().to_owned() {
		let child = tree.get_instance(child).unwrap();
		// println!("{:?}", child);

		// if let Some(path) = child.p
	}

	// match snapshot_children.cmp(&instance_children) {
	// 	// Child added
	// 	Ordering::Greater => {
	// 		for child in snapshot.children {
	// 			println!("{:?}", tree.exists(&snapshot.path.clone().unwrap()));
	// 			//TODO: what if snapshot.path is None?
	// 		}
	// 	}
	// 	// Child removed
	// 	Ordering::Less => for child in instance.children() {},
	// 	_ => {}
	// }
}

fn meta_from_vec(meta: VecDeque<&Meta>) -> Meta {
	meta.into_iter().fold(Meta::new(), |mut acc, meta| {
		acc.extend(meta.clone());
		acc
	})
}
