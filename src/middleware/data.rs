use anyhow::Result;
use log::error;
use rbx_dom_weak::types::{Tags, Variant};
use serde::{Deserialize, Serialize};
use std::{
	collections::HashMap,
	path::{Path, PathBuf},
};

use crate::{ext::PathExt, resolution::UnresolvedValue, util, vfs::Vfs};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Data {
	class_name: Option<String>,

	#[serde(default)]
	properties: HashMap<String, UnresolvedValue>,
	attributes: Option<UnresolvedValue>,
	#[serde(default)]
	tags: Vec<String>,

	#[serde(alias = "ignoreUnknownInstances", default)]
	keep_unknowns: Option<bool>,
}

#[derive(Debug)]
pub struct DataSnapshot {
	pub class: Option<String>,
	pub properties: HashMap<String, Variant>,
	pub keep_unknowns: Option<bool>,
	pub path: PathBuf,
}

#[profiling::function]
pub fn snapshot_data(path: &Path, vfs: &Vfs) -> Result<DataSnapshot> {
	let data = vfs.read_to_string(path)?;
	let data: Data = serde_json::from_str(&data)?;

	let mut properties = HashMap::new();

	let class = {
		if let Some(class_name) = &data.class_name {
			class_name.to_owned()
		} else {
			let name = path.get_name();

			if util::is_service(name) {
				name.to_owned()
			} else {
				let parent_name = path.get_parent().get_name();

				if util::is_service(parent_name) {
					parent_name.to_owned()
				} else {
					String::from("Folder")
				}
			}
		}
	};

	// Resolve properties
	for (property, value) in data.properties {
		match value.resolve(&class, &property) {
			Ok(value) => {
				properties.insert(property, value);
			}
			Err(err) => {
				error!("Failed to parse property: {}", err);
			}
		}
	}

	// Resolve attributes
	if let Some(attributes) = data.attributes {
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
	if !data.tags.is_empty() {
		properties.insert(String::from("Tags"), Tags::from(data.tags).into());
	}

	Ok(DataSnapshot {
		class: data.class_name,
		properties,
		keep_unknowns: data.keep_unknowns,
		path: path.to_owned(),
	})
}

#[derive(Debug, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WritableData {
	#[serde(skip_serializing_if = "Option::is_none")]
	pub class_name: Option<String>,

	#[serde(skip_serializing_if = "HashMap::is_empty")]
	pub properties: HashMap<String, Variant>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub attributes: Option<Variant>,
	#[serde(skip_serializing_if = "Vec::is_empty")]
	pub tags: Vec<String>,

	#[serde(skip_serializing_if = "Option::is_none")]
	pub keep_unknowns: Option<bool>,
}
