use crate::{core::meta::SyncbackFilter, resolution::UnresolvedValue, Properties};
use anyhow::{bail, Result};
use colored::Colorize;
use std::collections::HashMap;

#[cfg(not(windows))]
const FORBIDDEN_CHARACTERS: [char; 1] = ['/'];

#[cfg(windows)]
const FORBIDDEN_CHARACTERS: [char; 9] = ['<', '>', ':', '"', '/', '\\', '|', '?', '*'];

#[cfg(windows)]
const FORBIDDEN_FILE_NAMES: [&str; 22] = [
	"CON", "PRN", "AUX", "NUL", "COM1", "COM2", "COM3", "COM4", "COM5", "COM6", "COM7", "COM8", "COM9", "LPT1", "LPT2",
	"LPT3", "LPT4", "LPT5", "LPT6", "LPT7", "LPT8", "LPT9",
];

pub fn verify_name(name: &str) -> Result<()> {
	if name.is_empty() {
		bail!("file name cannot be empty");
	}

	if name.len() > 255 {
		bail!("file name cannot be longer than 255 characters");
	}

	#[cfg(windows)]
	if name.ends_with('.') {
		bail!("file name cannot end with a period");
	}

	#[cfg(windows)]
	if name.ends_with(' ') {
		bail!("file name cannot end with a space");
	}

	for char in name.chars() {
		if FORBIDDEN_CHARACTERS.contains(&char) {
			bail!("file name cannot contain {} character", char.to_string().bold());
		}

		#[cfg(windows)]
		if char.is_control() {
			bail!("file name cannot contain ASCII control characters");
		}
	}

	#[cfg(windows)]
	for file_name in FORBIDDEN_FILE_NAMES {
		if name == file_name {
			bail!("file cannot be named {}", file_name.bold());
		}
	}

	Ok(())
}

pub fn validate_properties(properties: Properties, filter: &SyncbackFilter) -> Properties {
	// Temporary solution for serde failing to deserialize empty HashMap
	if properties.contains_key("ArgonEmpty") {
		HashMap::new()
	} else {
		properties
			.into_iter()
			.filter(|(property, _)| !filter.matches_property(property))
			.collect()
	}
}

pub fn serialize_properties(class: &str, properties: Properties) -> HashMap<String, UnresolvedValue> {
	properties
		.iter()
		.map(|(property, variant)| {
			(
				property.to_owned(),
				UnresolvedValue::from_variant(variant.clone(), class, property),
			)
		})
		.collect()
}
