use anyhow::Result;
use std::{
	sync::{mpsc::Sender, Arc, Mutex, MutexGuard},
	thread,
};

use self::{dom::Dom, processor::Processor, queue::Queue};
use crate::{lock, project::Project, vfs::Vfs};

pub mod dom;
pub mod instance;
pub mod meta;
pub mod processor;
pub mod queue;

pub struct Core {
	project: Arc<Project>,
	processor: Arc<Processor>,
	queue: Arc<Mutex<Queue>>,
	dom: Arc<Mutex<Dom>>,
	vfs: Arc<Mutex<Vfs>>,
}

impl Core {
	pub fn new(project: Project) -> Result<Self> {
		let vfs = Arc::new(Mutex::new(Vfs::new()?));
		let dom = Arc::new(Mutex::new(Dom::new(&project)));
		let queue = Arc::new(Mutex::new(Queue::new()));

		let processor = Processor::new(queue.clone(), dom.clone(), vfs.clone());

		Ok(Core {
			project: Arc::new(project),
			processor: Arc::new(processor),
			queue,
			dom,
			vfs,
		})
	}

	pub fn watch(&self, sender: Option<Sender<()>>) {
		let vfs = self.vfs.clone();

		lock!(vfs).watch(&self.project.workspace_dir).unwrap();
	}

	pub fn name(&self) -> String {
		self.project.name.clone()
	}

	pub fn host(&self) -> Option<String> {
		self.project.host.clone()
	}

	pub fn port(&self) -> Option<u16> {
		self.project.port
	}

	pub fn game_id(&self) -> Option<u64> {
		self.project.game_id
	}

	pub fn place_ids(&self) -> Option<Vec<u64>> {
		self.project.place_ids.clone()
	}

	pub fn queue(&self) -> MutexGuard<'_, Queue> {
		lock!(self.queue)
	}
}
