use anyhow::Result;
use log::error;
use rbx_dom_weak::{types::Tags, ustr, HashMapExt, Ustr, UstrMap};
use serde::{Deserialize, Serialize};
use serde_json::Serializer;
use std::{
	collections::{BTreeMap, HashMap},
	path::{Path, PathBuf},
};

use crate::{
	core::meta::Meta,
	ext::PathExt,
	middleware::helpers,
	resolution::UnresolvedValue,
	util::{self, get_json_formatter},
	vfs::Vfs,
	Properties,
};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Data {
	class_name: Option<Ustr>,

	#[serde(default)]
	properties: HashMap<Ustr, UnresolvedValue>,
	attributes: Option<UnresolvedValue>,
	#[serde(default)]
	tags: Vec<String>,

	#[serde(alias = "ignoreUnknownInstances", default)]
	keep_unknowns: Option<bool>,
	#[serde(default)]
	original_name: Option<String>,
}

#[derive(Debug, Default)]
pub struct DataSnapshot {
	pub path: PathBuf,
	pub class: Option<Ustr>,
	pub properties: Properties,
	pub keep_unknowns: Option<bool>,
	pub original_name: Option<String>,
	pub mesh_source: Option<String>,
}

#[profiling::function]
pub fn read_data(path: &Path, class: Option<&str>, vfs: &Vfs) -> Result<DataSnapshot> {
	let data = vfs.read_to_string(path)?;

	if data.is_empty() {
		return Ok(DataSnapshot::default());
	}

	let data: Data = serde_json::from_str(&data)?;

	let mut properties = UstrMap::new();

	let class = if let Some(class) = class.or(data.class_name.as_deref()) {
		class.to_owned()
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
	};

	// Resolve properties
	for (property, value) in data.properties {
		match value.resolve(&class, &property) {
			Ok(value) => {
				properties.insert(property, value);
			}
			Err(err) => {
				error!("Failed to parse property: {} at {}", err, path.display());
			}
		}
	}

	// Resolve attributes
	if let Some(attributes) = data.attributes {
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
	if !data.tags.is_empty() {
		properties.insert(ustr("Tags"), Tags::from(data.tags).into());
	}

	let mesh_source = if class == "MeshPart" {
		helpers::save_mesh(&properties)
	} else {
		None
	};

	Ok(DataSnapshot {
		path: path.to_owned(),
		class: data.class_name,
		properties,
		keep_unknowns: data.keep_unknowns,
		original_name: data.original_name,
		mesh_source,
	})
}

#[derive(Debug, Default, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
struct WritableData {
	#[serde(skip_serializing_if = "Option::is_none")]
	pub class_name: Option<Ustr>,
	#[serde(skip_serializing_if = "BTreeMap::is_empty")]
	pub properties: BTreeMap<Ustr, UnresolvedValue>,

	#[serde(skip_serializing_if = "Option::is_none")]
	pub keep_unknowns: Option<bool>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub original_name: Option<String>,
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
		Some(Ustr::from(class))
	} else {
		None
	};

	let properties = properties
		.iter()
		.map(|(property, variant)| {
			(
				*property,
				UnresolvedValue::from_variant(variant.clone(), class, property),
			)
		})
		.collect();

	let mut data = WritableData {
		class_name,
		properties,
		original_name: meta.original_name.clone(),
		..WritableData::default()
	};

	if meta.keep_unknowns {
		data.keep_unknowns = Some(true);
	}

	if let Some(original_name) = meta.original_name.as_ref() {
		data.original_name = Some(original_name.to_owned());
	}

	if data == WritableData::default() {
		if vfs.exists(path) {
			vfs.remove(path)?;
		}

		return Ok(None);
	}

	let mut writer = Vec::new();
	let mut serializer = Serializer::with_formatter(&mut writer, get_json_formatter());

	data.serialize(&mut serializer)?;
	vfs.write(path, &writer)?;

	Ok(Some(path))
}

#[profiling::function]
pub fn write_original_name(path: &Path, meta: &Meta, vfs: &Vfs) -> Result<()> {
	let data = if vfs.exists(path) {
		let data = vfs.read_to_string(path)?;

		if data.is_empty() {
			return Ok(());
		}

		let data: Data = serde_json::from_str(&data)?;

		if data.original_name == meta.original_name {
			return Ok(());
		}

		let data = WritableData {
			class_name: data.class_name,
			properties: data.properties.into_iter().collect(),
			keep_unknowns: data.keep_unknowns,
			original_name: meta.original_name.clone(),
		};

		if data == WritableData::default() {
			vfs.remove(path)?;
			return Ok(());
		}

		data
	} else {
		let data = WritableData {
			original_name: meta.original_name.clone(),
			..WritableData::default()
		};

		if data == WritableData::default() {
			return Ok(());
		}

		data
	};

	let mut writer = Vec::new();
	let mut serializer = Serializer::with_formatter(&mut writer, get_json_formatter());

	data.serialize(&mut serializer)?;
	vfs.write(path, &writer)?;

	Ok(())
}
