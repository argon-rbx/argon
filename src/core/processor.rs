use crossbeam_channel::{select, Receiver, Sender};
use std::{
	sync::{Arc, Mutex},
	thread,
};

use super::{queue::Queue, tree::Tree};
use crate::{
	lock, middleware,
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
			sender,
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
}

struct Handler {
	queue: Arc<Mutex<Queue>>,
	tree: Arc<Mutex<Tree>>,
	vfs: Arc<Mutex<Vfs>>,
	sender: Sender<()>,
}

impl Handler {
	fn new(queue: Arc<Mutex<Queue>>, tree: Arc<Mutex<Tree>>, vfs: Arc<Mutex<Vfs>>, sender: Sender<()>) -> Self {
		Self {
			queue,
			tree,
			vfs,
			sender,
		}
	}

	fn on_vfs_event(&self, event: VfsEvent) {
		let mut vfs = lock!(self.vfs);
		let mut tree = lock!(self.tree);
		let mut queue = lock!(self.queue);

		vfs.process_event(&event);

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
					let meta = tree.get_meta(id).unwrap();
					let snapshot = middleware::from_path(&path, meta, &vfs);
					println!("{:?}", snapshot);
				}

				true
			}
		};

		if changed {
			self.sender.send(()).unwrap();
		}
	}
}
