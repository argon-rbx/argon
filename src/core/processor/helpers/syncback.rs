use colored::Colorize;
use std::{
	collections::HashMap,
	path::{Path, PathBuf},
};
use uuid::Uuid;

use crate::{
	argon_error, argon_warn,
	config::Config,
	core::meta::{Meta, SyncbackFilter},
	ext::PathExt,
	resolution::UnresolvedValue,
	vfs::Vfs,
	Properties,
};

#[cfg(not(windows))]
const FORBIDDEN_CHARACTERS: [char; 1] = ['/'];

#[cfg(windows)]
const FORBIDDEN_CHARACTERS: [char; 9] = ['<', '>', ':', '"', '/', '\\', '|', '?', '*'];

#[cfg(windows)]
const FORBIDDEN_FILE_NAMES: [&str; 22] = [
	"CON", "PRN", "AUX", "NUL", "COM1", "COM2", "COM3", "COM4", "COM5", "COM6", "COM7", "COM8", "COM9", "LPT1", "LPT2",
	"LPT3", "LPT4", "LPT5", "LPT6", "LPT7", "LPT8", "LPT9",
];

pub fn verify_name(name: &mut String, meta: &mut Meta) -> bool {
	let verify = || -> Option<(String, String)> {
		if name.is_empty() {
			return Some(("file name cannot be empty".into(), "EmptyName".into()));
		}

		if name.len() > 255 {
			return Some((
				"file name cannot be longer than 255 characters".into(),
				name[..255].into(),
			));
		}

		#[cfg(windows)]
		if name.ends_with('.') {
			return Some((
				"file name cannot end with a period".into(),
				name[..name.len() - 1].into(),
			));
		}

		#[cfg(windows)]
		if name.ends_with(' ') {
			return Some((
				"file name cannot end with a space".into(),
				name[..name.len() - 1].into(),
			));
		}

		{
			let mut forbidden_chars = vec![];

			for char in name.chars() {
				if FORBIDDEN_CHARACTERS.contains(&char) && !forbidden_chars.contains(&char) {
					forbidden_chars.push(char);
				}

				#[cfg(windows)]
				if char.is_control() && !forbidden_chars.contains(&char) {
					forbidden_chars.push(char);
				}
			}

			if !forbidden_chars.is_empty() {
				let message = if forbidden_chars.len() == 1 {
					format!(
						"file name cannot contain {} character",
						if forbidden_chars[0].is_control() {
							"ASCII control".bold()
						} else {
							forbidden_chars[0].to_string().bold()
						}
					)
				} else {
					format!(
						"file name cannot contain {} characters",
						forbidden_chars
							.iter()
							.map(|char| if char.is_control() {
								"ASCII control".bold().to_string()
							} else {
								char.to_string().bold().to_string()
							})
							.collect::<Vec<String>>()
							.join(", ")
					)
				};

				let mut name = name.to_owned();

				for char in forbidden_chars {
					name = name.replace(char, "");
				}

				if name.is_empty() {
					name = "EmptyName".into();
				}

				return Some((message, name));
			}
		}

		#[cfg(windows)]
		for file_name in FORBIDDEN_FILE_NAMES {
			if name == file_name {
				return Some((
					format!("file cannot be named {}", file_name.bold()),
					format!("{}{}", name, name.chars().last().unwrap()),
				));
			}
		}

		None
	};

	if let Some((message, renamed)) = verify() {
		if Config::new().rename_instances {
			argon_warn!(
				"Instance with name: {} got renamed to: {}, because: {}!",
				name.bold(),
				renamed.bold(),
				message
			);

			meta.set_original_name(Some(name.to_owned()));
			*name = renamed;

			return true;
		} else {
			argon_error!(
				"Instance with name: {} is corrupted: {}! Skipping..",
				name.bold(),
				message
			);

			return false;
		}
	} else if meta.original_name.is_some() {
		meta.set_original_name(None);
	}

	true
}

pub fn verify_path(path: &mut PathBuf, name: &mut String, meta: &mut Meta, vfs: &Vfs) -> bool {
	if !vfs.exists(path) || meta.source.get().path().is_some_and(|p| p == path) {
		return true;
	}

	if Config::new().rename_instances {
		let suffix = path.get_name().strip_prefix(name.as_str()).unwrap_or_default();

		let renamed = format!("{}_{}", name, Uuid::new_v4());
		let renamed_path = path.with_file_name(&format!("{}{}", renamed, suffix));

		argon_warn!(
			"Instance with path: {} got renamed to: {}, because it already exists!",
			path.to_string().bold(),
			renamed_path.to_string().bold()
		);

		meta.set_original_name(Some(name.to_owned()));

		*path = renamed_path;
		*name = renamed;

		true
	} else {
		argon_error!(
			"Instance with path: {} already exists! Skipping..",
			path.to_string().bold()
		);

		false
	}
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

pub fn rename_path(path: &Path, from: &str, to: &str) -> PathBuf {
	path.with_file_name(format!(
		"{}{}",
		to,
		path.get_name().strip_prefix(from).unwrap_or_default()
	))
}
