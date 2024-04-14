use anyhow::Result;
use rbx_dom_weak::types::{Enum, Variant};
use std::{collections::HashMap, path::Path};

use super::FileType;
use crate::{
	core::{meta::Context, snapshot::Snapshot},
	vfs::Vfs,
};

#[derive(Debug, Clone, PartialEq)]
pub enum ScriptType {
	Server,
	Client,
	Module,
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

#[profiling::function]
pub fn snapshot_lua(path: &Path, context: &Context, vfs: &Vfs, script_type: ScriptType) -> Result<Snapshot> {
	let (class_name, run_context) = match (context.use_legacy_scripts(), &script_type) {
		(false, ScriptType::Server) => ("Script", Some(Variant::Enum(Enum::from_u32(1)))),
		(false, ScriptType::Client) => ("Script", Some(Variant::Enum(Enum::from_u32(2)))),
		(true, ScriptType::Server) => ("Script", Some(Variant::Enum(Enum::from_u32(0)))),
		(true, ScriptType::Client) => ("LocalScript", None),
		(_, ScriptType::Module) => ("ModuleScript", None),
	};

	let mut snapshot = Snapshot::new().with_class(class_name);
	let mut properties = HashMap::new();

	let source = vfs.read_to_string(path)?;

	if script_type != ScriptType::Module {
		let mut overridden = false;

		if let Some(line) = source.lines().next() {
			if line.contains("--disable") {
				properties.insert(String::from("Disabled"), Variant::Bool(true));
			}

			if script_type == ScriptType::Server {
				if line.contains("--server") {
					properties.insert(String::from("RunContext"), Variant::Enum(Enum::from_u32(1)));
					overridden = true;
				} else if line.contains("--client") {
					properties.insert(String::from("RunContext"), Variant::Enum(Enum::from_u32(2)));
					overridden = true;
				} else if line.contains("--plugin") {
					properties.insert(String::from("RunContext"), Variant::Enum(Enum::from_u32(3)));
					overridden = true;
				}
			}
		}

		if !overridden {
			if let Some(run_context) = run_context {
				properties.insert(String::from("RunContext"), run_context);
			}
		}
	}

	properties.insert(String::from("Source"), Variant::String(source));
	snapshot.set_properties(properties);

	Ok(snapshot)
}
