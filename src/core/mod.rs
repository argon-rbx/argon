use anyhow::Result;
use std::sync::{mpsc::Sender, Arc, Mutex, MutexGuard};

use self::{meta::Meta, processor::Processor, queue::Queue, tree::Tree};
use crate::{lock, middleware, project::Project, vfs::Vfs};

pub mod dom;
pub mod instance;
pub mod meta;
pub mod processor;
pub mod queue;
pub mod snapshot;
pub mod tree;

pub struct Core {
	project: Arc<Project>,
	processor: Arc<Processor>,
	queue: Arc<Mutex<Queue>>,
	tree: Arc<Mutex<Tree>>,
	vfs: Arc<Mutex<Vfs>>,
}

impl Core {
	pub fn new(project: Project) -> Result<Self> {
		let vfs = Vfs::new()?;
		let dom = Tree::new(&project);
		let queue = Queue::new();

		let snapshot = middleware::from_path(&project.workspace_dir, &Meta::empty(), &vfs);
		println!("{:?}", snapshot);

		let vfs = Arc::new(Mutex::new(vfs));
		let tree = Arc::new(Mutex::new(dom));
		let queue = Arc::new(Mutex::new(queue));

		let processor = Processor::new(queue.clone(), tree.clone(), vfs.clone());

		Ok(Core {
			project: Arc::new(project),
			processor: Arc::new(processor),
			queue,
			tree,
			vfs,
		})
	}

	pub fn watch(&self, sender: Option<Sender<()>>) {
		lock!(self.vfs).watch(&self.project.workspace_dir).unwrap();
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
