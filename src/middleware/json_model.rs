use anyhow::Result;
use log::error;
use rbx_dom_weak::{types::Tags, ustr, HashMapExt, Ustr, UstrMap};
use serde::Deserialize;
use std::path::Path;

use super::helpers;
use crate::{core::snapshot::Snapshot, resolution::UnresolvedValue, vfs::Vfs};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct JsonModel {
	#[serde(alias = "Name")]
	name: Option<String>,
	#[serde(alias = "ClassName")]
	class_name: Option<Ustr>,

	#[serde(alias = "Properties")]
	properties: Option<UstrMap<UnresolvedValue>>,
	#[serde(alias = "Attributes")]
	attributes: Option<UnresolvedValue>,
	#[serde(alias = "Tags")]
	tags: Option<Vec<String>>,

	#[serde(alias = "Children")]
	children: Option<Vec<JsonModel>>,
}

#[profiling::function]
pub fn read_json_model(path: &Path, vfs: &Vfs) -> Result<Snapshot> {
	let contents = vfs.read_to_string(path)?;

	if contents.is_empty() {
		return Ok(Snapshot::new().with_class("Folder"));
	}

	let model = serde_json::from_str(&contents)?;
	let snapshot = walk(model, path)?;

	Ok(snapshot)
}

fn walk(model: JsonModel, path: &Path) -> Result<Snapshot> {
	let mut snapshot = Snapshot::new();
	let mut properties = UstrMap::new();

	// Apply class
	let class = model.class_name.unwrap_or(snapshot.class);
	snapshot.set_class(&class);

	// Apply name
	if let Some(name) = model.name {
		snapshot.set_name(&name);
	}

	// Resolve properties
	if let Some(model_properties) = model.properties {
		for (property, value) in model_properties {
			match value.resolve(&class, &property) {
				Ok(value) => {
					properties.insert(property, value);
				}
				Err(err) => {
					error!("Failed to parse property: {} at {}", err, path.display());
				}
			}
		}
	}

	// Resolve attributes
	if let Some(attributes) = model.attributes {
		match attributes.resolve(&class, "Attributes") {
			Ok(value) => {
				properties.insert(ustr("Attributes"), value);
			}
			Err(err) => {
				error!("Failed to parse attributes: {} at {}", err, path.display());
			}
		}
	}

	// Resolve tags
	if let Some(tags) = model.tags {
		properties.insert(ustr("Tags"), Tags::from(tags).into());
	}

	if class == "MeshPart" {
		snapshot.meta.set_mesh_source(helpers::save_mesh(&properties));
	}

	snapshot.set_properties(properties);

	// Append children
	for child in model.children.unwrap_or_default() {
		snapshot.add_child(walk(child, path)?);
	}

	Ok(snapshot)
}
