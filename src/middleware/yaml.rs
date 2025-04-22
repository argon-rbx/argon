use anyhow::Result;
use rbx_dom_weak::{types::Variant, ustr, HashMapExt, UstrMap};
use std::path::Path;

use crate::{core::snapshot::Snapshot, vfs::Vfs};

#[profiling::function]
pub fn read_yaml(path: &Path, vfs: &Vfs) -> Result<Snapshot> {
	let yaml = vfs.read_to_string(path)?;
	let lua = yaml2lua::parse(&yaml)?;

	let source = format!("return {}", lua);

	let mut properties = UstrMap::new();
	properties.insert(ustr("Source"), Variant::String(source));

	Ok(Snapshot::new().with_class("ModuleScript").with_properties(properties))
}
