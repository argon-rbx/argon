use anyhow::Result;
use rbx_dom_weak::types::{Enum, Variant};
use std::{collections::HashMap, path::Path};

use super::FileType;
use crate::{core::snapshot::Snapshot, vfs::Vfs};

#[derive(Debug, Clone, PartialEq)]
pub enum ScriptType {
	Server,
	Client,
	Module,
}

impl ScriptType {
	fn as_class(&self) -> &'static str {
		match self {
			ScriptType::Server => "Script",
			ScriptType::Client => "LocalScript",
			ScriptType::Module => "ModuleScript",
		}
	}
}

impl From<FileType> for ScriptType {
	fn from(file_type: FileType) -> Self {
		match file_type {
			FileType::ServerScript => ScriptType::Server,
			FileType::ClientScript => ScriptType::Client,
			FileType::ModuleScript => ScriptType::Module,
			_ => panic!("Cannot convert {:?} to ScriptType", file_type),
		}
	}
}

pub fn snapshot_lua(name: &str, path: &Path, vfs: &Vfs, script_type: ScriptType) -> Result<Snapshot> {
	let mut snapshot = Snapshot::new(name).with_class(script_type.as_class()).with_path(path);
	let mut properties = HashMap::new();

	let source = vfs.read(path)?;

	if script_type != ScriptType::Module {
		if let Some(line) = source.lines().next() {
			if line.contains("--disable") {
				properties.insert(String::from("Disabled"), Variant::Bool(true));
			}

			if script_type == ScriptType::Server {
				if line.contains("--server") {
					properties.insert(String::from("RunContext"), Variant::Enum(Enum::from_u32(1)));
				} else if line.contains("--client") {
					properties.insert(String::from("RunContext"), Variant::Enum(Enum::from_u32(2)));
				} else if line.contains("--plugin") {
					properties.insert(String::from("RunContext"), Variant::Enum(Enum::from_u32(3)));
				}
			}
		}
	}

	properties.insert(String::from("Source"), Variant::String(source));

	snapshot.properties = properties;

	Ok(snapshot)
}
