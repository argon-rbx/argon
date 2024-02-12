use anyhow::{bail, Result};
use colored::Colorize;
use log::error;
use rbx_dom_weak::types::Tags;
use std::{collections::HashMap, path::Path};

use super::new_snapshot;
use crate::{
	argon_warn,
	core::{
		meta::{Meta, ProjectData},
		snapshot::Snapshot,
	},
	project::{Project, ProjectNode},
	util::{self, PathExt},
	vfs::Vfs,
};

#[profiling::function]
pub fn snapshot_project(path: &Path, vfs: &Vfs) -> Result<Snapshot> {
	let project: Project = Project::load(path)?;

	let super_path = path.get_parent();

	let mut meta = Meta::from_project(&project);
	let mut snapshot = walk(&project.name, super_path, &meta, vfs, project.node)?;

	// We don't have to keep project data in project snapshot
	meta.project_data = None;
	snapshot.set_meta(meta);

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

		if let Some(node_properties) = node.properties {
			for (property, value) in node_properties {
				match value.resolve(&class, &property) {
					Ok(value) => {
						properties.insert(property, value);
					}
					Err(err) => {
						error!("Failed to parse property: {}", err);
					}
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

		if let Some(tags) = node.tags {
			properties.insert(String::from("Tags"), Tags::from(tags).into());
		}

		properties
	};

	let mut snapshot = Snapshot::new()
		.with_name(name)
		.with_class(&class)
		.with_properties(properties);

	if let Some(node_path) = node.path {
		let path = path.join(node_path);

		if path.is_file() {
			vfs.watch(&path)?;
		}

		let meta = {
			let source = meta.project_data.clone().unwrap().source;
			let mut project_data = ProjectData::new(name, &path, &source);
			let mut meta = meta.clone();

			if class != "Folder" {
				project_data.set_class(class.clone());
			}

			if !snapshot.properties.is_empty() {
				project_data.set_properties(snapshot.properties.clone());
			}

			meta.set_project_data(project_data);
			meta
		};

		if let Some(mut path_snapshot) = new_snapshot(&path, &meta, vfs)? {
			// We want to keep project data only
			let meta = Meta::new().with_project_data(meta.project_data.unwrap());

			path_snapshot.extend_properties(snapshot.properties);
			path_snapshot.set_name(&snapshot.name);
			path_snapshot.extend_meta(meta);

			if path_snapshot.class == "Folder" {
				path_snapshot.set_class(&snapshot.class);
			}

			snapshot = path_snapshot
		} else {
			argon_warn!(
				"Path specified in the project does not exist: {}. Please create this path and restart Argon \
				to watch for file changes in this path or remove it from the project to suppress this warning.",
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
