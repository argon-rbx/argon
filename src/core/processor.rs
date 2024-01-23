use crossbeam_channel::{select, Receiver, Sender};
use std::{
	sync::{Arc, Mutex},
	thread,
};

use super::{dom::Dom, queue::Queue};
use crate::{
	lock, middleware,
	vfs::{Vfs, VfsEvent},
};

pub struct Processor {
	handler: Arc<Handler>,
	callback: Receiver<()>,
}

impl Processor {
	pub fn new(queue: Arc<Mutex<Queue>>, dom: Arc<Mutex<Dom>>, vfs: Arc<Mutex<Vfs>>) -> Self {
		let (sender, receiver) = crossbeam_channel::unbounded();

		let handler = Arc::new(Handler {
			queue: queue.clone(),
			dom: dom.clone(),
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
	dom: Arc<Mutex<Dom>>,
	vfs: Arc<Mutex<Vfs>>,
	sender: Sender<()>,
}

impl Handler {
	fn new(queue: Arc<Mutex<Queue>>, dom: Arc<Mutex<Dom>>, vfs: Arc<Mutex<Vfs>>, sender: Sender<()>) -> Self {
		Self {
			queue,
			dom,
			vfs,
			sender,
		}
	}

	fn on_vfs_event(&self, event: VfsEvent) {
		let mut vfs = lock!(self.vfs);
		let mut dom = lock!(self.dom);
		let mut queue = lock!(self.queue);

		vfs.process_event(&event);

		let changed = match event {
			VfsEvent::Create(path) | VfsEvent::Write(path) | VfsEvent::Delete(path) => {
				// middleware::from_path(&path, &context, &vfs, &dom);
				println!("{:?}", path);

				true
			}
		};

		if changed {
			self.sender.send(()).unwrap();
		}
	}
}
