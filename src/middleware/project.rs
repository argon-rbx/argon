use anyhow::{bail, Result};
use colored::Colorize;
use log::error;
use path_clean::PathClean;
use rbx_dom_weak::{types::Tags, ustr, HashMapExt, UstrMap};
use std::path::Path;

use super::new_snapshot;
use crate::{
	argon_warn,
	core::{
		meta::{Context, Meta, NodePath, Source},
		snapshot::Snapshot,
	},
	ext::PathExt,
	middleware::helpers,
	project::{Project, ProjectNode, ProjectPath},
	util,
	vfs::Vfs,
};

#[profiling::function]
pub fn read_project(path: &Path, vfs: &Vfs) -> Result<Snapshot> {
	let project: Project = Project::load(path)?;

	vfs.watch(path, false)?;

	let meta = Meta::from_project(&project);
	let mut snapshot = new_snapshot_node(&project.name, path, project.node, NodePath::new(), &meta.context, vfs)?;

	snapshot.meta.source.add_project(path);

	Ok(snapshot)
}

#[profiling::function]
pub fn new_snapshot_node(
	name: &str,
	path: &Path,
	node: ProjectNode,
	node_path: NodePath,
	context: &Context,
	vfs: &Vfs,
) -> Result<Snapshot> {
	if node.class_name.is_some() && node.path.is_some() {
		bail!("Failed to load project: $className and $path cannot be set at the same time");
	}

	let class = if let Some(class_name) = &node.class_name {
		class_name.to_owned()
	} else if util::is_service(name) {
		name.to_owned()
	} else {
		String::from("Folder")
	};

	let properties = {
		let mut properties = UstrMap::new();

		for (property, value) in &node.properties {
			match value.clone().resolve(&class, property) {
				Ok(value) => {
					properties.insert(*property, value);
				}
				Err(err) => {
					error!(
						"Failed to parse property: {} at {}, JSON path: {}",
						err,
						path.display(),
						node_path
					);
				}
			}
		}

		if let Some(attributes) = &node.attributes {
			match attributes.clone().resolve(&class, "Attributes") {
				Ok(value) => {
					properties.insert(ustr("Attributes"), value);
				}
				Err(err) => {
					error!(
						"Failed to parse attributes: {} at {}, JSON path: {}",
						err,
						path.display(),
						node_path
					);
				}
			}
		}

		if !node.tags.is_empty() {
			properties.insert(ustr("Tags"), Tags::from(node.tags.clone()).into());
		}

		properties
	};

	let mut meta = Meta::new()
		.with_source(Source::project(name, path, node.clone(), node_path.clone()))
		.with_context(context)
		.with_keep_unknowns(node.keep_unknowns.unwrap_or_else(|| util::is_service(&class)));

	if class == "MeshPart" {
		meta.set_mesh_source(helpers::save_mesh(&properties));
	}

	let mut snapshot = Snapshot::new()
		.with_name(name)
		.with_class(&class)
		.with_properties(properties)
		.with_meta(meta);

	if let Some(path_node) = node.path {
		let path = path.with_file_name(path_node.path()).clean();

		if vfs.exists(&path) {
			vfs.watch(&path, vfs.is_dir(&path))?;

			if let Some(mut path_snapshot) = new_snapshot(&path, context, vfs)? {
				path_snapshot.extend_properties(snapshot.properties);
				path_snapshot.set_name(&snapshot.name);

				if path_snapshot.class == "Folder" {
					path_snapshot.set_class(&snapshot.class);
				}

				// We want to keep the original inner source
				// but with addition of new relevant paths
				snapshot
					.meta
					.source
					.extend_relevant(path_snapshot.meta.source.relevant().to_owned());

				path_snapshot.meta.set_source(snapshot.meta.source);
				path_snapshot
					.meta
					.set_keep_unknowns(path_snapshot.meta.keep_unknowns || snapshot.meta.keep_unknowns);

				snapshot = path_snapshot;
			}
		} else if let ProjectPath::Required(_) = path_node {
			argon_warn!(
				"Path specified in the project does not exist: {}. Please create this path and restart Argon \
				to watch for file changes in this path or remove it from the project to suppress this warning",
				path.to_string().bold()
			);
		}
	}

	for (node_name, node) in node.tree {
		let node_path = node_path.join(&node_name);
		let child = new_snapshot_node(&node_name, path, node, node_path, context, vfs)?;

		snapshot.add_child(child);
	}

	Ok(snapshot)
}
