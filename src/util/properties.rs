use anyhow::{bail, Result};
use log::error;
use rbx_dom_weak::types::{Attributes, Tags, Variant};
use std::{collections::HashMap, path::Path};

use crate::project::ProjectNode;

pub fn from_json(path: &Path) -> Result<HashMap<String, Variant>> {
	if super::get_file_stem(path) == ".data" {
		super::json::read_data(path)
	} else {
		super::json::read_meta(path)
	}
}

pub fn from_node(node: ProjectNode, name: &str) -> Result<HashMap<String, Variant>> {
	let mut properties = HashMap::new();

	let class = {
		if let Some(class) = node.class_name {
			class.to_owned()
		} else {
			if !super::is_service(name) {
				bail!("No ClassName property found");
			};

			name.to_owned()
		}
	};

	properties.insert(String::from("ClassName"), Variant::String(class.clone()));

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

	Ok(properties)
}
