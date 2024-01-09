use anyhow::{bail, Result};
use log::error;
use rbx_dom_weak::types::Variant;
use std::{
	collections::HashMap,
	fs::{self, File},
	io::BufReader,
	path::Path,
};

use crate::{resolution::UnresolvedValue, util};

pub fn read_module(path: &Path) -> Result<String> {
	let mut module = String::from("return ");

	let json = fs::read_to_string(path)?;
	let lua = json2lua::parse(&json)?;

	module.push_str(&lua);

	Ok(module)
}

pub fn read_properties(path: &Path) -> Result<HashMap<String, Variant>> {
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

			let name = util::get_file_name(path);
			let is_service = util::is_service(name);

			if !is_service {
				bail!("No ClassName property found");
			};

			name.to_owned()
		}
	};

	for (property, value) in data {
		match value.resolve(&class, &property) {
			Ok(value) => {
				properties.insert(property, value);
			}
			Err(err) => {
				error!("{err}");
			}
		}
	}

	Ok(properties)
}
