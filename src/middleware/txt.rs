use anyhow::Result;
use rbx_dom_weak::{types::Variant, ustr, HashMapExt, UstrMap};
use std::path::Path;

use crate::{core::snapshot::Snapshot, vfs::Vfs, Properties};

#[profiling::function]
pub fn read_txt(path: &Path, vfs: &Vfs) -> Result<Snapshot> {
	let value = vfs.read_to_string(path)?;

	let mut properties = UstrMap::new();
	properties.insert(ustr("Value"), Variant::String(value));

	Ok(Snapshot::new().with_class("StringValue").with_properties(properties))
}

#[profiling::function]
pub fn write_txt(mut properties: Properties, path: &Path, vfs: &Vfs) -> Result<Properties> {
	let value = if let Some(Variant::String(value)) = properties.remove(&ustr("Value")) {
		value
	} else {
		String::new()
	};

	vfs.write(path, value.as_bytes())?;

	Ok(properties)
}
