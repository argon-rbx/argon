use anyhow::Result;
use rbx_dom_weak::{types::Variant, ustr, HashMapExt, UstrMap};
use std::path::Path;

use crate::{core::snapshot::Snapshot, vfs::Vfs};

#[profiling::function]
pub fn read_toml(path: &Path, vfs: &Vfs) -> Result<Snapshot> {
	let toml = vfs.read_to_string(path)?;
	let lua = toml2lua::parse(&toml)?;

	let source = format!("return {}", lua);

	let mut properties = UstrMap::new();
	properties.insert(ustr("Source"), Variant::String(source));

	Ok(Snapshot::new().with_class("ModuleScript").with_properties(properties))
}
