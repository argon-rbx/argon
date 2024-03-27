use anyhow::Result;
use rbx_dom_weak::types::Variant;
use std::{collections::HashMap, path::Path};

use crate::{core::snapshot::Snapshot, vfs::Vfs};

#[profiling::function]
pub fn snapshot_toml(path: &Path, vfs: &Vfs) -> Result<Snapshot> {
	let toml = vfs.read_to_string(path)?;
	let lua = toml2lua::parse(&toml)?;

	let source = format!("return {}", lua);

	let mut properties = HashMap::new();
	properties.insert(String::from("Source"), Variant::String(source));

	Ok(Snapshot::new().with_class("ModuleScript").with_properties(properties))
}
