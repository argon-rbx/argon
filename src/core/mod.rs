use anyhow::Result;
use crossbeam_channel::Receiver;
use rbx_xml::EncodeOptions;
use std::{
	fs::File,
	io::BufWriter,
	path::Path,
	sync::{Arc, Mutex, MutexGuard},
};

use self::{meta::Meta, processor::Processor, queue::Queue, tree::Tree};
use crate::{lock, middleware::new_snapshot, project::Project, vfs::Vfs};

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
		let vfs = Vfs::new();

		let snapshot = new_snapshot(&project.workspace_dir, &Meta::default(), &vfs)?;

		let vfs = Arc::new(Mutex::new(vfs));
		let tree = Arc::new(Mutex::new(Tree::new(snapshot.unwrap())));
		let queue = Arc::new(Mutex::new(Queue::new()));

		let processor = Processor::new(queue.clone(), tree.clone(), vfs.clone());

		Ok(Core {
			project: Arc::new(project),
			processor: Arc::new(processor),
			queue,
			tree,
			vfs,
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

	pub fn watch(&self) -> Receiver<()> {
		lock!(self.vfs).watch(&self.project.workspace_dir).unwrap();

		self.processor.callback()
	}

	pub fn build(&self, path: &Path, xml: bool) -> Result<()> {
		let writer = BufWriter::new(File::create(path)?);

		let tree = lock!(self.tree);

		let root_refs = if self.project.is_place() {
			tree.place_root_refs().to_vec()
		} else {
			vec![tree.root_ref()]
		};

		if xml {
			rbx_xml::to_writer(writer, tree.inner(), &root_refs, EncodeOptions::default())?;
		} else {
			rbx_binary::to_writer(writer, tree.inner(), &root_refs)?;
		}

		Ok(())
	}
}
