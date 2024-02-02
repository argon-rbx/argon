use anyhow::Result;
use crossbeam_channel::Receiver;
use std::{
	fs::File,
	io::BufWriter,
	path::Path,
	sync::{Arc, Mutex, MutexGuard},
	thread,
	time::Duration,
};

use self::{meta::Meta, processor::Processor, queue::Queue, tree::Tree};
use crate::{lock, middleware::new_snapshot, project::Project, vfs::Vfs};

pub mod change;
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
	_vfs: Arc<Vfs>,
}

impl Core {
	#[profiling::function]
	pub fn new(project: Project, watch: bool) -> Result<Self> {
		profiling::start_frame!();

		let vfs = Vfs::new(watch);

		if watch {
			vfs.watch(&project.workspace_dir)?;
		}

		let meta = Meta::from_project(&project);
		let snapshot = new_snapshot(&project.workspace_dir, &meta, &vfs)?;

		let vfs = Arc::new(vfs);
		let tree = Arc::new(Mutex::new(Tree::new(snapshot.unwrap())));
		let queue = Arc::new(Mutex::new(Queue::new()));

		let processor = Processor::new(queue.clone(), tree.clone(), vfs.clone());

		Ok(Core {
			project: Arc::new(project),
			processor: Arc::new(processor),
			queue,
			tree,
			_vfs: vfs,
		})
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

	pub fn tree_changed(&self) -> Receiver<()> {
		self.processor.callback()
	}

	pub fn build(&self, path: &Path, xml: bool) -> Result<()> {
		let writer = BufWriter::new(File::create(path)?);

		// We want to proritize event processing over building
		// so we can wait for the Mutex lock to release
		let tree = loop {
			match self.tree.try_lock() {
				Ok(guard) => {
					break guard;
				}
				Err(_) => {
					thread::sleep(Duration::from_millis(1));
				}
			}
		};

		let root_refs = if self.project.is_place() {
			tree.place_root_refs().to_vec()
		} else {
			vec![tree.root_ref()]
		};

		if xml {
			rbx_xml::to_writer_default(writer, tree.inner(), &root_refs)?;
		} else {
			rbx_binary::to_writer(writer, tree.inner(), &root_refs)?;
		}

		Ok(())
	}
}
