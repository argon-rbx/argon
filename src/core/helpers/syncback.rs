use colored::Colorize;
use rbx_dom_weak::{
	types::{Ref, Variant},
	ustr, HashMapExt, Ustr, UstrMap,
};
use std::path::{Path, PathBuf};
use uuid::Uuid;

use crate::{
	argon_error, argon_warn,
	config::Config,
	core::{
		meta::{Meta, SyncbackFilter},
		tree::Tree,
	},
	ext::PathExt,
	resolution::{is_ref_property, UnresolvedValue},
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
	let (messages, renamed) = {
		let mut messages = Vec::new();
		let mut name = name.clone();

		if name.len() > 255 {
			messages.push("file name cannot be longer than 255 characters".into());
			name = name[..255].to_owned();
		}

		{
			let mut forbidden_chars = Vec::new();

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

				messages.push(message);

				for char in forbidden_chars {
					name = name.replace(char, "");
				}
			}
		}

		#[cfg(windows)]
		if name.ends_with('.') || name.ends_with(' ') {
			messages.push("file name cannot end with a period or space".into());

			while name.ends_with('.') || name.ends_with(' ') {
				name = name[..name.len() - 1].to_owned();
			}
		}

		if name.is_empty() {
			messages.push("file name cannot be empty".into());
			name = "EmptyName".into();
		} else {
			#[cfg(windows)]
			for file_name in FORBIDDEN_FILE_NAMES {
				if name == file_name {
					messages.push(format!("file cannot be named {}", file_name.bold()));
					name = format!("{}{}", name, name.chars().last().unwrap());
				}
			}
		}

		(messages, name)
	};

	if !messages.is_empty() {
		if Config::new().rename_instances {
			argon_warn!(
				"Instance with name: {} got renamed to: {}, because: {}!",
				name.bold(),
				renamed.bold(),
				messages.iter().map(|m| m.as_str()).collect::<Vec<&str>>().join(" & ")
			);

			meta.set_original_name(Some(name.to_owned()));
			*name = renamed;

			return true;
		} else {
			argon_error!(
				"Instance with name: {} is corrupted: {}! Skipping..",
				name.bold(),
				messages.iter().map(|m| m.as_str()).collect::<Vec<&str>>().join(" & ")
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

	if Config::new().keep_duplicates {
		let suffix = path.get_name().strip_prefix(name.as_str()).unwrap_or_default();

		let renamed = format!("{}_{}", name, Uuid::new_v4());
		let renamed_path = path.with_file_name(format!("{renamed}{suffix}"));

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
	// Temporary solution for empty Luau maps being serialized as arrays
	if properties.contains_key(&ustr("ArgonEmpty")) {
		UstrMap::new()
	} else {
		properties
			.into_iter()
			.filter(|(property, _)| !filter.matches_property(property))
			.collect()
	}
}

pub fn resolve_ref_properties(properties: &mut Properties, class: &str, anchor_dir: &Path, tree: &Tree) {
	let ref_properties: Vec<Ustr> = properties
		.keys()
		.copied()
		.filter(|property| is_ref_property(class, property))
		.collect();

	for property in ref_properties {
		let target = match properties.get(&property) {
			Some(Variant::Ref(target)) => *target,
			_ => continue,
		};

		if target == Ref::none() {
			properties.remove(&property);
			continue;
		}

		match tree.get_meta(target).and_then(|meta| meta.source.anchor_dir()) {
			Some(target_dir) => {
				let relative = relative_path(anchor_dir, target_dir);
				properties.insert(property, Variant::String(path_to_ref_string(&relative)));
			}
			None => {
				argon_warn!(
					"Failed to serialize Ref property {}.{}: target instance is outside of the synced tree",
					class.bold(),
					property.bold()
				);

				properties.remove(&property);
			}
		}
	}
}

fn relative_path(from: &Path, to: &Path) -> PathBuf {
	let from: Vec<_> = from.components().collect();
	let to: Vec<_> = to.components().collect();

	let common = from.iter().zip(to.iter()).take_while(|(a, b)| a == b).count();

	let mut result = PathBuf::new();

	for _ in common..from.len() {
		result.push("..");
	}

	for component in &to[common..] {
		result.push(component.as_os_str());
	}

	if result.as_os_str().is_empty() {
		result.push(".");
	}

	result
}

fn path_to_ref_string(path: &Path) -> String {
	path.components()
		.map(|component| component.as_os_str().to_string_lossy())
		.collect::<Vec<_>>()
		.join("/")
}

pub fn serialize_properties(class: &str, properties: Properties) -> UstrMap<UnresolvedValue> {
	properties
		.iter()
		.map(|(property, variant)| {
			(
				*property,
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

#[cfg(test)]
mod tests {
	use super::*;
	use crate::core::{
		meta::{Meta, Source},
		snapshot::Snapshot,
		tree::Tree,
	};

	#[test]
	fn resolve_ref_properties_computes_relative_path() {
		let root_path = Path::new("/project/src/Model");
		let part_a_path = root_path.join("PartA");
		let part_b_path = root_path.join("PartB");

		let mut root_meta = Meta::new();
		root_meta.set_source(Source::directory(root_path));

		let mut part_a_meta = Meta::new();
		part_a_meta.set_source(Source::directory(&part_a_path));

		let mut part_b_meta = Meta::new();
		part_b_meta.set_source(Source::directory(&part_b_path));

		let snapshot = Snapshot::new()
			.with_class("Model")
			.with_name("Model")
			.with_meta(root_meta)
			.with_children(vec![
				Snapshot::new()
					.with_class("Weld")
					.with_name("PartA")
					.with_meta(part_a_meta),
				Snapshot::new()
					.with_class("Folder")
					.with_name("PartB")
					.with_meta(part_b_meta),
			]);

		let tree = Tree::new(snapshot);

		let part_b_id = tree.root().children()[1];

		let mut properties = Properties::default();
		properties.insert(ustr("Part0"), Variant::Ref(part_b_id));

		resolve_ref_properties(&mut properties, "Weld", &part_a_path, &tree);

		assert_eq!(
			properties.get(&ustr("Part0")),
			Some(&Variant::String("../PartB".to_owned()))
		);
	}

	#[test]
	fn resolve_ref_properties_removes_unset_ref() {
		let mut properties = Properties::default();
		properties.insert(ustr("Part0"), Variant::Ref(Ref::none()));

		let snapshot = Snapshot::new().with_class("Model").with_name("Model");
		let tree = Tree::new(snapshot);

		resolve_ref_properties(&mut properties, "Weld", Path::new("/project/src/Model"), &tree);

		assert!(!properties.contains_key(&ustr("Part0")));
	}

	#[test]
	fn relative_path_siblings() {
		let from = Path::new("/project/src/Hitbox");
		let to = Path::new("/project/src/Welds");

		assert_eq!(relative_path(from, to), Path::new("../Welds"));
	}

	#[test]
	fn relative_path_descendant() {
		let from = Path::new("/project/src/Model");
		let to = Path::new("/project/src/Model/Hitbox");

		assert_eq!(relative_path(from, to), Path::new("Hitbox"));
	}

	#[test]
	fn relative_path_same_dir() {
		let path = Path::new("/project/src/Model");

		assert_eq!(relative_path(path, path), Path::new("."));
	}

	#[test]
	fn path_to_ref_string_uses_forward_slashes() {
		let relative = Path::new("..").join("Welds").join("Weld");

		assert_eq!(path_to_ref_string(&relative), "../Welds/Weld");
	}
}
