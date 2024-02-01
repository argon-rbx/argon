use anyhow::Result;
use log::error;
use rbx_dom_weak::types::Tags;
use serde::Deserialize;
use std::{collections::HashMap, path::Path};

use crate::{core::snapshot::Snapshot, resolution::UnresolvedValue, vfs::Vfs};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct JsonModel {
	#[serde(alias = "Name")]
	name: Option<String>,
	#[serde(alias = "ClassName")]
	class_name: Option<String>,

	#[serde(alias = "Properties")]
	properties: Option<HashMap<String, UnresolvedValue>>,
	#[serde(alias = "Attributes")]
	attributes: Option<UnresolvedValue>,
	#[serde(alias = "Tags")]
	tags: Option<Vec<String>>,

	#[serde(alias = "Children")]
	children: Option<Vec<JsonModel>>,
}

#[profiling::function]
pub fn snapshot_json_model(path: &Path, vfs: &Vfs) -> Result<Snapshot> {
	let model = vfs.read(path)?;
	let model: JsonModel = serde_json::from_str(&model)?;

	let snapshot = walk(model)?;

	Ok(snapshot)
}

fn walk(model: JsonModel) -> Result<Snapshot> {
	let mut snapshot = Snapshot::new();
	let mut properties = HashMap::new();

	// Apply class
	let class = model.class_name.unwrap_or(snapshot.class.clone());
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
					error!("Failed to parse property: {}", err);
				}
			}
		}
	}

	// Resolve attributes
	if let Some(attributes) = model.attributes {
		match attributes.resolve(&class, "Attributes") {
			Ok(value) => {
				properties.insert(String::from("Attributes"), value);
			}
			Err(err) => {
				error!("Failed to parse attributes: {}", err);
			}
		}
	}

	// Resolve tags
	if let Some(tags) = model.tags {
		properties.insert(String::from("Tags"), Tags::from(tags).into());
	}

	snapshot.set_properties(properties);

	// Append children
	for child in model.children.unwrap_or_default() {
		snapshot.add_child(walk(child)?);
	}

	Ok(snapshot)
}
