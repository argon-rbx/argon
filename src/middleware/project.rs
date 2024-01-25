use anyhow::{bail, Result};
use log::error;
use rbx_dom_weak::types::{Attributes, Tags};
use std::{collections::HashMap, fs, path::Path};

use super::new_snapshot;
use crate::{
	core::{meta::Meta, snapshot::Snapshot},
	project::{Project, ProjectNode},
	util,
	vfs::Vfs,
};

pub fn snapshot_project(_name: &str, path: &Path, meta: &Meta, vfs: &Vfs) -> Result<Option<Snapshot>> {
	let project = fs::read_to_string(path)?;
	let project: Project = serde_json::from_str(&project)?;

	let snapshot = snapshot_project_node(&project.name, project.node, meta, vfs)?.with_path(path);

	Ok(Some(snapshot))
}

pub fn snapshot_project_node(name: &str, node: ProjectNode, meta: &Meta, vfs: &Vfs) -> Result<Snapshot> {
	if node.class_name.is_some() && node.path.is_some() {
		bail!("Failed to load project: $className and $path cannot be set at the same time");
	}

	let mut snapshot = Snapshot::new(name);

	let class = {
		if let Some(class_name) = node.class_name {
			Some(class_name)
		} else if util::is_service(name) {
			Some(name.into())
		} else {
			None
		}
	};

	if let Some(class) = class {
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

		snapshot.class = class;
		snapshot.properties = properties;
	} else {
		snapshot.class = String::from("Folder");
	}

	if let Some(path) = node.path {
		if let Some(snap) = new_snapshot(&path, meta, vfs)? {
			snapshot = snap.with_name(&snapshot.name);
		}

		snapshot.path = util::resolve_path(path)?;
	}

	for (name, node) in node.tree {
		let child = snapshot_project_node(&name, node, meta, vfs)?;
		snapshot.children.push(child);
	}

	Ok(snapshot)
}
