use anyhow::Result;
use rbx_dom_weak::types::Variant;
use std::{collections::HashMap, path::Path};

use crate::{core::snapshot::Snapshot, vfs::Vfs};

#[profiling::function]
pub fn snapshot_json(path: &Path, vfs: &Vfs) -> Result<Snapshot> {
	let json = vfs.read(path)?;
	let lua = json2lua::parse(&json)?;

	let source = format!("return {}", lua);

	let mut properties = HashMap::new();
	properties.insert(String::from("Source"), Variant::String(source));

	Ok(Snapshot::new().with_class("ModuleScript").with_properties(properties))
}
