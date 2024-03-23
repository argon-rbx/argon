use anyhow::{bail, Result};
use colored::Colorize;
use log::error;
use rbx_dom_weak::types::Tags;
use std::{collections::HashMap, path::Path};

use super::new_snapshot;
use crate::{
	argon_warn,
	core::{meta::Meta, snapshot::Snapshot},
	ext::PathExt,
	project::{Project, ProjectNode},
	util,
	vfs::Vfs,
};

#[profiling::function]
pub fn snapshot_project(path: &Path, vfs: &Vfs) -> Result<Snapshot> {
	let project: Project = Project::load(path)?;

	let super_path = path.get_parent();

	let meta = Meta::from_project(&project);
	let snapshot = walk(&project.name, super_path, &meta, vfs, project.node)?.with_meta(meta);

	vfs.watch(path)?;

	Ok(snapshot)
}

fn walk(name: &str, path: &Path, meta: &Meta, vfs: &Vfs, node: ProjectNode) -> Result<Snapshot> {
	if node.class_name.is_some() && node.path.is_some() {
		bail!("Failed to load project: $className and $path cannot be set at the same time");
	}

	let class = {
		if let Some(class_name) = node.class_name {
			class_name
		} else if util::is_service(name) {
			name.to_owned()
		} else {
			String::from("Folder")
		}
	};

	let properties = {
		let mut properties = HashMap::new();

		for (property, value) in node.properties {
			match value.resolve(&class, &property) {
				Ok(value) => {
					properties.insert(property, value);
				}
				Err(err) => {
					error!("Failed to parse property: {}", err);
				}
			}
		}

		if let Some(attributes) = node.attributes {
			match attributes.resolve(&class, "Attributes") {
				Ok(value) => {
					properties.insert(String::from("Attributes"), value);
				}
				Err(err) => {
					error!("Failed to parse attributes: {}", err);
				}
			}
		}

		if !node.tags.is_empty() {
			properties.insert(String::from("Tags"), Tags::from(node.tags).into());
		}

		properties
	};

	let mut snapshot = Snapshot::new()
		.with_name(name)
		.with_class(&class)
		.with_properties(properties);

	if let Some(node_path) = node.path {
		let path = path.join(path_clean::clean(node_path));

		if path.is_file() {
			vfs.watch(&path)?;
		}

		if let Some(mut path_snapshot) = new_snapshot(&path, meta, vfs)? {
			path_snapshot.extend_properties(snapshot.properties);
			path_snapshot.set_name(&snapshot.name);

			if path_snapshot.class == "Folder" {
				path_snapshot.set_class(&snapshot.class);
			}

			snapshot = path_snapshot
		} else {
			argon_warn!(
				"Path specified in the project does not exist: {}. Please create this path and restart Argon \
				to watch for file changes in this path or remove it from the project to suppress this warning",
				path.to_string().bold()
			);
		}

		// If path does not exist, we still want
		// to keep it in the snapshot
		if snapshot.paths.is_empty() {
			snapshot.add_path(&path);
		}
	}

	for (name, node) in node.tree {
		let child = walk(&name, path, meta, vfs, node)?;
		snapshot.add_child(child);
	}

	Ok(snapshot)
}
