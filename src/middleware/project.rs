use anyhow::{bail, Result};
use log::error;
use rbx_dom_weak::types::{Attributes, Tags};
use std::{collections::HashMap, fs, path::Path};

use super::new_snapshot;
use crate::{
	core::{
		meta::{Meta, ProjectData},
		snapshot::Snapshot,
	},
	project::{Project, ProjectNode},
	util,
	vfs::Vfs,
};

#[profiling::function]
pub fn snapshot_project(path: &Path, meta: &Meta, vfs: &Vfs) -> Result<Snapshot> {
	let project = fs::read_to_string(path)?;
	let project: Project = serde_json::from_str(&project)?;

	let super_path = path.parent().unwrap();

	let snapshot = snapshot_project_node(&project.name, super_path, meta, vfs, project.node)?
		.with_meta(meta.to_owned())
		.with_path(path);

	Ok(snapshot)
}

pub fn snapshot_project_node(name: &str, path: &Path, meta: &Meta, vfs: &Vfs, node: ProjectNode) -> Result<Snapshot> {
	if node.class_name.is_some() && node.path.is_some() {
		bail!("Failed to load project: $className and $path cannot be set at the same time");
	}

	let class = {
		if let Some(class_name) = node.class_name {
			class_name
		} else if util::is_service(name) {
			name.to_string()
		} else {
			String::from("Folder")
		}
	};

	let properties = {
		let mut properties = HashMap::new();

		if let Some(meta_properties) = node.properties {
			for (property, value) in meta_properties {
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

		if let Some(node_attributes) = node.attributes {
			let mut attributes = Attributes::new();

			for (key, unresolved) in node_attributes {
				match unresolved.resolve_unambiguous() {
					Ok(value) => {
						attributes.insert(key, value);
					}
					Err(err) => {
						error!("Failed to parse attribute: {}", err);
					}
				}
			}

			properties.insert(String::from("Attributes"), attributes.into());
		}

		if let Some(tags) = node.tags {
			properties.insert(String::from("Tags"), Tags::from(tags).into());
		}

		properties
	};

	let mut snapshot = Snapshot::new(name).with_class(&class).with_properties(properties);

	if let Some(node_path) = node.path {
		let path = path.join(node_path);

		if let Some(mut path_snapshot) = new_snapshot(&path, meta, vfs)? {
			let meta = {
				let mut project_data = ProjectData::new(name, &path);

				if class != "Folder" {
					project_data.set_class(class.clone());
				}

				if !snapshot.properties.is_empty() {
					project_data.set_properties(snapshot.properties.clone());
				}

				Meta::new().with_project_data(project_data)
			};

			path_snapshot.set_meta(meta);
			path_snapshot.set_name(&snapshot.name);
			path_snapshot.extend_properties(snapshot.properties);

			if path_snapshot.class == "Folder" {
				path_snapshot.set_class(&snapshot.class);
			}

			snapshot = path_snapshot
		}

		if snapshot.path.is_none() {
			snapshot.set_path(&path);
		}
	}

	for (name, node) in node.tree {
		let child = snapshot_project_node(&name, path, meta, vfs, node)?;
		snapshot.add_child(child);
	}

	Ok(snapshot)
}
