use anyhow::Result;
use json_formatter::JsonFormatter;
use log::error;
use rbx_dom_weak::types::{Tags, Variant};
use serde::{Deserialize, Serialize};
use std::{
	collections::{BTreeMap, HashMap},
	path::{Path, PathBuf},
};

use crate::{
	core::meta::Meta, ext::PathExt, middleware::helpers, resolution::UnresolvedValue, util, vfs::Vfs, Properties,
};

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

#[derive(Debug, Default)]
pub struct DataSnapshot {
	pub path: PathBuf,
	pub class: Option<String>,
	pub properties: Properties,
	pub keep_unknowns: Option<bool>,
	pub mesh_source: Option<String>,
}

#[profiling::function]
pub fn read_data(path: &Path, vfs: &Vfs) -> Result<DataSnapshot> {
	let data = vfs.read_to_string(path)?;

	if data.is_empty() {
		return Ok(DataSnapshot::default());
	}

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

	let mesh_source = if class == "MeshPart" {
		helpers::save_mesh(&mut properties)
	} else {
		None
	};

	Ok(DataSnapshot {
		path: path.to_owned(),
		class: data.class_name,
		properties,
		keep_unknowns: data.keep_unknowns,
		mesh_source,
	})
}

#[derive(Debug, Default, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
struct WritableData {
	#[serde(skip_serializing_if = "Option::is_none")]
	pub class_name: Option<String>,

	#[serde(skip_serializing_if = "BTreeMap::is_empty")]
	pub properties: BTreeMap<String, UnresolvedValue>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub attributes: Option<Variant>,
	#[serde(skip_serializing_if = "Vec::is_empty")]
	pub tags: Vec<String>,

	#[serde(skip_serializing_if = "Option::is_none")]
	pub keep_unknowns: Option<bool>,
}

#[profiling::function]
pub fn write_data<'a>(
	has_file: bool,
	class: &str,
	properties: Properties,
	path: &'a Path,
	meta: &Meta,
	vfs: &Vfs,
) -> Result<Option<&'a Path>> {
	let class_name = if !has_file && class != "Folder" {
		Some(class.to_owned())
	} else {
		None
	};

	let properties = properties
		.iter()
		.map(|(property, varaint)| {
			(
				property.to_owned(),
				UnresolvedValue::from_variant(varaint.clone(), class, property),
			)
		})
		.collect();

	let mut data = WritableData {
		class_name,
		properties,
		..WritableData::default()
	};

	if meta.keep_unknowns {
		data.keep_unknowns = Some(true);
	}

	if data == WritableData::default() {
		if vfs.exists(path) {
			vfs.remove(path)?;
		}

		return Ok(None);
	}

	let formatter = JsonFormatter::with_array_breaks(false);

	let mut writer = Vec::new();
	let mut serializer = serde_json::Serializer::with_formatter(&mut writer, formatter);

	data.serialize(&mut serializer)?;
	writer.push(b'\n');

	vfs.write(path, &writer)?;

	Ok(Some(path))
}
