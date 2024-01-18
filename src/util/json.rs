use anyhow::{bail, Result};
use log::error;
use rbx_dom_weak::types::{Tags, Variant};
use serde::{Deserialize, Serialize};
use std::{
	collections::HashMap,
	fs::{self, File},
	io::BufReader,
	path::Path,
};

use crate::resolution::UnresolvedValue;

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
struct MetaFile {
	pub class_name: Option<String>,

	pub properties: Option<HashMap<String, UnresolvedValue>>,
	pub attributes: Option<UnresolvedValue>,
	pub tags: Option<Vec<String>>,

	// This field is not actually used by Argon
	#[allow(dead_code)]
	#[serde(skip_serializing)]
	pub ignore_unknown_instances: Option<bool>,
}

pub fn read_module(path: &Path) -> Result<String> {
	let mut module = String::from("return ");

	let json = fs::read_to_string(path)?;
	let lua = json2lua::parse(&json)?;

	module.push_str(&lua);

	Ok(module)
}

pub fn read_data(path: &Path) -> Result<HashMap<String, Variant>> {
	let reader = BufReader::new(File::open(path)?);
	let data: HashMap<String, UnresolvedValue> = serde_json::from_reader(reader)?;
	let mut properties = HashMap::new();

	if data.is_empty() {
		return Ok(properties);
	}

	let class = {
		if let Some(class) = data.get("ClassName") {
			let class = class.as_str();

			if class.is_none() {
				bail!("ClassName property is not a string");
			}

			class.unwrap().to_owned()
		} else {
			let path = path.parent().unwrap();

			let name = super::get_file_name(path);
			let is_service = super::is_service(name);

			if !is_service {
				bail!("No ClassName property found");
			};

			name.to_owned()
		}
	};

	for (property, value) in data {
		if property == "ClassName" {
			continue;
		}

		match value.resolve(&class, &property) {
			Ok(value) => {
				properties.insert(property, value);
			}
			Err(err) => {
				error!("Failed to parse property: {}", err);
			}
		}
	}

	Ok(properties)
}

pub fn read_meta(path: &Path) -> Result<HashMap<String, Variant>> {
	let reader = BufReader::new(File::open(path)?);
	let meta: MetaFile = serde_json::from_reader(reader)?;
	let mut properties = HashMap::new();

	let class = {
		if let Some(class) = meta.class_name {
			class.to_owned()
		} else {
			let path = path.parent().unwrap();

			let name = super::get_file_name(path);
			let is_service = super::is_service(name);

			if !is_service {
				bail!("No ClassName property found");
			};

			name.to_owned()
		}
	};

	if let Some(meta_properties) = meta.properties {
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

	if let Some(attributes) = meta.attributes {
		match attributes.resolve(&class, "Attributes") {
			Ok(value) => {
				properties.insert(String::from("Attributes"), value);
			}
			Err(err) => {
				error!("Failed to parse attributes: {}", err);
			}
		}
	}

	if let Some(tags) = meta.tags {
		properties.insert(String::from("Tags"), Tags::from(tags).into());
	}

	Ok(properties)
}
