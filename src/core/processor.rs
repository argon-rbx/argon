use crossbeam_channel::{select, Receiver, Sender};
use log::error;
use rbx_dom_weak::types::Ref;
use std::{
	sync::{Arc, Mutex},
	thread::Builder,
};

use super::{queue::Queue, tree::Tree};
use crate::{
	lock,
	middleware::{new_snapshot, FileType},
	vfs::{Vfs, VfsEvent},
};

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
					process_changes(id, &tree, &self.vfs);
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
fn process_changes(id: Ref, tree: &Tree, vfs: &Vfs) {
	profiling::start_frame!();

	let path = tree.get_path(id).unwrap();
	let meta = tree.get_meta(id);

	// println!("{:?}", path);

	let snapshot = match new_snapshot(path, meta, vfs) {
		Ok(snapshot) => snapshot,
		Err(err) => {
			error!("Failed to create snapshot: {}, path: {:?}", err, path);
			return;
		}
	};

	if let Some(snapshot) = snapshot {
		let instance = tree.get_instance(id).unwrap();

		if snapshot.name != instance.name
			|| snapshot.class != instance.class
			|| snapshot.properties != instance.properties
		{
			//update
		}

	// match snapshot.file_type.clone().unwrap() {
	// 	FileType::Project | FileType::RbxmModel | FileType::RbxmxModel => {
	// 		process_child_changes();
	// 	}
	// 	_ => {}
	// }

	// println!("{:#?}", snapshot);
	// println!("{:#?}", instance);
	} else {

		//delete
	}
}

fn process_child_changes() {}
