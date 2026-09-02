use anyhow::{bail, Result};
use log::trace;
use rbx_dom_weak::{types::Ref, Ustr};
use serde::Serialize;
use snapshot::AddedSnapshot;
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
pub mod helpers;
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
		let snapshot = new_snapshot(&project.path, &meta.context, &vfs)?.expect(
			"Failed to snapshot root project. \
		Note that projects cannot be empty. \
		If you are using custom sync rules make sure you have one with the `Project` type. \
		Otherwise, this is a bug.",
		);

		trace!("Building Tree and Queue");

		let vfs = Arc::new(vfs);
		let tree = Arc::new(Mutex::new(Tree::new(snapshot)));
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
		self.project().name.clone()
	}

	pub fn host(&self) -> Option<String> {
		self.project().host.clone()
	}

	pub fn port(&self) -> Option<u16> {
		self.project().port
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

	/// Create snapshot of the tree or a subtree
	pub fn snapshot(&self, instance: Ref) -> Option<AddedSnapshot> {
		let tree = self.tree();

		fn walk(children: &[Ref], tree: &Tree) -> Vec<Snapshot> {
			let mut snapshot_children = Vec::new();

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

		let root = if instance.is_some() {
			tree.get_instance(instance)?
		} else {
			tree.root()
		};
		let meta = tree.get_meta(root.referent()).unwrap();

		Some(
			Snapshot::new()
				.with_id(root.referent())
				.with_name(&root.name)
				.with_class(&root.class)
				.with_properties(root.properties.clone())
				.with_children(walk(root.children(), &tree))
				.with_meta(meta.clone())
				.as_new(root.parent()),
		)
	}

	/// Build the tree into a file, either XML or binary
	pub fn build(&self, path: &Path, xml: bool) -> Result<()> {
		let writer = BufWriter::new(File::create(path)?);
		let tree = lock!(&self.tree);

		let root_refs = if self.project().is_place() {
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

		let workspace_dir = &self.project().workspace_dir;

		fn walk(tree: &Tree, id: Ref, workspace_dir: &Path, non_scripts: bool) -> Option<SourcemapNode> {
			let instance = tree.get_instance(id).unwrap();

			let children: Vec<SourcemapNode> = instance
				.children()
				.iter()
				.filter_map(|&child_id| walk(tree, child_id, workspace_dir, non_scripts))
				.collect();

			if children.is_empty() && (!non_scripts && !util::is_script(&instance.class)) {
				return None;
			}

			let file_paths = tree.get_meta(id).map_or(Vec::new(), |meta| {
				meta.source
					.relevant()
					.iter()
					.filter_map(|entry| match entry {
						SourceEntry::File(path) | SourceEntry::Data(path) | SourceEntry::Project(path) => {
							Some(path.strip_prefix(workspace_dir).unwrap_or(path).to_owned())
						}
						_ => None,
					})
					.collect()
			});

			Some(SourcemapNode {
				name: instance.name.clone(),
				class_name: instance.class,
				file_paths,
				children,
			})
		}

		let sourcemap = walk(&tree, dom.root_ref(), workspace_dir, non_scripts);

		if let Some(path) = path {
			let writer = BufWriter::new(File::create(path)?);
			serde_json::to_writer(writer, &sourcemap)?;
		} else {
			println!("{}", serde_json::to_string(&sourcemap)?);
		}

		Ok(())
	}

	pub fn open(&self, instance: Ref) -> Result<()> {
		let tree = self.tree();

		let mut sources = if let Some(meta) = tree.get_meta(instance) {
			meta.source.relevant().to_owned()
		} else {
			Vec::new()
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
	class_name: Ustr,
	#[serde(skip_serializing_if = "Vec::is_empty")]
	file_paths: Vec<PathBuf>,
	#[serde(skip_serializing_if = "Vec::is_empty")]
	children: Vec<SourcemapNode>,
}
