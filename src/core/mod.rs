use anyhow::{bail, Result};
use log::trace;
use rbx_dom_weak::types::Ref;
use serde::Serialize;
use std::{
	fs::File,
	io::BufWriter,
	path::{Path, PathBuf},
	sync::{Arc, Mutex, MutexGuard},
};

use self::{
	meta::{Meta, SourceEntry},
	processor::Processor,
	queue::Queue,
	tree::Tree,
};
use crate::{core::snapshot::Snapshot, lock, middleware::new_snapshot, project::Project, stats, util, vfs::Vfs};

pub mod changes;
pub mod meta;
pub mod processor;
pub mod queue;
pub mod snapshot;
pub mod tree;

pub struct Core {
	project: Arc<Mutex<Project>>,
	tree: Arc<Mutex<Tree>>,
	queue: Arc<Queue>,
	processor: Arc<Processor>,
	_vfs: Arc<Vfs>,
}

impl Core {
	#[profiling::function]
	pub fn new(project: Project, watch: bool) -> Result<Self> {
		profiling::start_frame!();

		trace!("Initializing VFS");

		let vfs = Vfs::new(watch);

		trace!("Snapshotting root project");

		let meta = Meta::from_project(&project);
		let snapshot = new_snapshot(&project.path, &meta.context, &vfs)?;

		trace!("Building Tree and Queue");

		let vfs = Arc::new(vfs);
		let tree = Arc::new(Mutex::new(Tree::new(snapshot.unwrap())));
		let queue = Arc::new(Queue::new());

		trace!("Starting Processor");

		let project = Arc::new(Mutex::new(project));
		let processor = Arc::new(Processor::new(
			queue.clone(),
			tree.clone(),
			vfs.clone(),
			project.clone(),
		));

		trace!("Core initialized successfully!");

		Ok(Core {
			project,
			tree,
			queue,
			processor,
			_vfs: vfs,
		})
	}

	pub fn name(&self) -> String {
		lock!(self.project).name.clone()
	}

	pub fn host(&self) -> Option<String> {
		lock!(self.project).host.clone()
	}

	pub fn port(&self) -> Option<u16> {
		lock!(self.project).port
	}

	pub fn project(&self) -> MutexGuard<'_, Project> {
		lock!(self.project)
	}

	pub fn tree(&self) -> MutexGuard<'_, Tree> {
		lock!(self.tree)
	}

	pub fn queue(&self) -> Arc<Queue> {
		self.queue.clone()
	}

	pub fn processor(&self) -> Arc<Processor> {
		self.processor.clone()
	}

	/// Create snapshot of the tree
	pub fn snapshot(&self) -> Snapshot {
		let tree = lock!(self.tree);

		fn walk(children: &[Ref], tree: &Tree) -> Vec<Snapshot> {
			let mut snapshot_children = vec![];

			for child in children {
				let meta = tree.get_meta(*child).unwrap();
				let child = tree.get_instance(*child).unwrap();

				let snapshot = Snapshot::new()
					.with_id(child.referent())
					.with_name(&child.name)
					.with_class(&child.class)
					.with_properties(child.properties.clone())
					.with_children(walk(child.children(), tree))
					.with_meta(meta.clone());

				snapshot_children.push(snapshot);
			}

			snapshot_children
		}

		let root = tree.root();
		let meta = tree.get_meta(root.referent()).unwrap();

		Snapshot::new()
			.with_id(root.referent())
			.with_name(&root.name)
			.with_class(&root.class)
			.with_properties(root.properties.clone())
			.with_children(walk(tree.root().children(), &tree))
			.with_meta(meta.clone())
	}

	/// Build the tree into a file, either XML or binary
	pub fn build(&self, path: &Path, xml: bool) -> Result<()> {
		let writer = BufWriter::new(File::create(path)?);
		let tree = lock!(&self.tree);

		let root_refs = if lock!(self.project).is_place() {
			tree.place_root_refs().to_vec()
		} else {
			vec![tree.root_ref()]
		};

		if xml {
			rbx_xml::to_writer_default(writer, tree.inner(), &root_refs)?;
		} else {
			rbx_binary::to_writer(writer, tree.inner(), &root_refs)?;
		}

		stats::projects_built(1);

		Ok(())
	}

	/// Write sourcemap of the tree
	pub fn sourcemap(&self, path: Option<PathBuf>, non_scripts: bool) -> Result<()> {
		let tree = lock!(&self.tree);
		let dom = tree.inner();

		fn walk(tree: &Tree, id: Ref, non_scripts: bool) -> Option<SourcemapNode> {
			let instance = tree.get_instance(id).unwrap();

			let children: Vec<SourcemapNode> = instance
				.children()
				.iter()
				.filter_map(|&child_id| walk(tree, child_id, non_scripts))
				.collect();

			if children.is_empty() && (!non_scripts && !util::is_script(&instance.class)) {
				return None;
			}

			let file_paths = tree.get_meta(id).map_or(vec![], |meta| {
				meta.source
					.relevants()
					.iter()
					.filter_map(|entry| match entry {
						SourceEntry::File(path) | SourceEntry::Data(path) => Some(path.to_owned()),
						_ => None,
					})
					.collect()
			});

			Some(SourcemapNode {
				name: instance.name.clone(),
				class_name: instance.class.clone(),
				file_paths,
				children,
			})
		}

		let mut sourcemap = walk(&tree, dom.root_ref(), non_scripts);

		// We need to add root project path manually
		// as we ignore other project paths by default
		if let Some(sourcemap) = &mut sourcemap {
			let root_path = tree.get_meta(tree.root_ref()).unwrap().source.paths()[0].to_owned();
			sourcemap.file_paths.push(root_path);
		}

		if let Some(path) = path {
			let writer = BufWriter::new(File::create(path)?);
			serde_json::to_writer(writer, &sourcemap)?;
		} else {
			println!("{}", serde_json::to_string(&sourcemap)?);
		}

		Ok(())
	}

	pub fn open(&self, instance: Ref) -> Result<()> {
		let tree = lock!(self.tree);

		let mut sources = if let Some(meta) = tree.get_meta(instance) {
			meta.source.relevants().to_owned()
		} else {
			vec![]
		};

		sources.sort_by_key(|source| source.index());

		if let Some(source) = sources.first() {
			open::that(source.path())?;
			Ok(())
		} else {
			bail!("No matching file was found")
		}
	}
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct SourcemapNode {
	name: String,
	class_name: String,
	#[serde(skip_serializing_if = "Vec::is_empty")]
	file_paths: Vec<PathBuf>,
	#[serde(skip_serializing_if = "Vec::is_empty")]
	children: Vec<SourcemapNode>,
}
