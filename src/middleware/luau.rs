use anyhow::Result;
use rbx_dom_weak::{
	types::{Enum, Variant},
	ustr, HashMapExt, UstrMap,
};
use std::path::Path;

use super::Middleware;
use crate::{
	core::{meta::Context, snapshot::Snapshot},
	vfs::Vfs,
	Properties,
};

#[derive(Debug, Clone, PartialEq)]
pub enum ScriptType {
	Server,
	Client,
	Module,
}

impl From<Middleware> for ScriptType {
	fn from(middleware: Middleware) -> Self {
		match middleware {
			Middleware::ServerScript => ScriptType::Server,
			Middleware::ClientScript => ScriptType::Client,
			Middleware::ModuleScript => ScriptType::Module,
			_ => panic!("Cannot convert {:?} to ScriptType", middleware),
		}
	}
}

#[profiling::function]
pub fn read_luau(path: &Path, context: &Context, vfs: &Vfs, script_type: ScriptType) -> Result<Snapshot> {
	let (class_name, run_context) = match (context.use_legacy_scripts(), &script_type) {
		(false, ScriptType::Server) => ("Script", Some(Variant::Enum(Enum::from_u32(1)))),
		(false, ScriptType::Client) => ("Script", Some(Variant::Enum(Enum::from_u32(2)))),
		(true, ScriptType::Server) => ("Script", Some(Variant::Enum(Enum::from_u32(0)))),
		(true, ScriptType::Client) => ("LocalScript", None),
		(_, ScriptType::Module) => ("ModuleScript", None),
	};

	let mut snapshot = Snapshot::new().with_class(class_name);
	let mut properties = UstrMap::new();

	let source = vfs.read_to_string(path)?;

	if script_type != ScriptType::Module {
		if let Some(run_context) = run_context {
			properties.insert(ustr("RunContext"), run_context);
		}
	}

	properties.insert(ustr("Source"), Variant::String(source));
	snapshot.set_properties(properties);

	Ok(snapshot)
}

#[profiling::function]
pub fn write_luau(mut properties: Properties, path: &Path, vfs: &Vfs) -> Result<Properties> {
	let source = if let Some(Variant::String(source)) = properties.remove(&ustr("Source")) {
		source
	} else {
		String::new()
	};

	vfs.write(path, source.as_bytes())?;

	Ok(properties)
}
