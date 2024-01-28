use crossbeam_channel::{select, Receiver, Sender};
use log::{error, trace};
use std::{
	sync::{Arc, Mutex},
	thread,
};

use super::{queue::Queue, tree::Tree};
use crate::{
	lock,
	middleware::new_snapshot,
	vfs::{Vfs, VfsEvent},
};

pub struct Processor {
	handler: Arc<Handler>,
	callback: Receiver<()>,
}

impl Processor {
	pub fn new(queue: Arc<Mutex<Queue>>, tree: Arc<Mutex<Tree>>, vfs: Arc<Mutex<Vfs>>) -> Self {
		let (sender, receiver) = crossbeam_channel::unbounded();

		let handler = Arc::new(Handler {
			queue: queue.clone(),
			tree: tree.clone(),
			vfs: vfs.clone(),
			callback: sender,
		});

		{
			let handler = handler.clone();

			thread::spawn(move || {
				let vfs_receiver = lock!(vfs).receiver();

				loop {
					select! {
						recv(vfs_receiver) -> event => {
							handler.on_vfs_event(event.unwrap());
						}
					}
				}
			});
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
	vfs: Arc<Mutex<Vfs>>,
	callback: Sender<()>,
}

impl Handler {
	fn new(queue: Arc<Mutex<Queue>>, tree: Arc<Mutex<Tree>>, vfs: Arc<Mutex<Vfs>>, callback: Sender<()>) -> Self {
		Self {
			queue,
			tree,
			vfs,
			callback,
		}
	}

	fn on_vfs_event(&self, event: VfsEvent) {
		let mut vfs = lock!(self.vfs);
		let mut tree = lock!(self.tree);
		let mut queue = lock!(self.queue);

		vfs.process_event(&event);

		let changed = match event {
			VfsEvent::Create(path) | VfsEvent::Write(path) | VfsEvent::Delete(path) => {
				let is_new = tree.get_ids(&path).is_none();
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
					let meta = tree.get_meta(id);
					let snapshot = match new_snapshot(&path, meta, &vfs) {
						Ok(snapshot) => snapshot,
						Err(err) => {
							error!("Failed to create snapshot: {}, path: {:?}", err, path);
							continue;
						}
					};

					if let Some(snapshot) = snapshot {
						if is_new {
							trace!("Inserting {:?}", path);
							tree.insert(snapshot, id);
						} else {
							trace!("Updating {:?}", path);
							println!("{:?}", snapshot);
						}
					} else if !is_new {
						trace!("Removing {:?}", path);
						tree.remove(id);
					}
				}

				true
			}
		};

		if changed {
			self.callback.send(()).unwrap();
		}
	}
}
