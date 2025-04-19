use anyhow::Result;
use rbx_dom_weak::{types::Variant, HashMapExt, Ustr, UstrMap};
use std::path::Path;

use crate::{core::snapshot::Snapshot, vfs::Vfs, Properties};

#[profiling::function]
pub fn read_txt(path: &Path, vfs: &Vfs) -> Result<Snapshot> {
	let value = vfs.read_to_string(path)?;

	let mut properties = UstrMap::new();
	properties.insert(Ustr::from("Value"), Variant::String(value));

	Ok(Snapshot::new().with_class("StringValue").with_properties(properties))
}

#[profiling::function]
pub fn write_txt(mut properties: Properties, path: &Path, vfs: &Vfs) -> Result<Properties> {
	let mut contents = String::new();

	if let Some(Variant::String(value)) = properties.remove(&Ustr::from("Value")) {
		contents = value;
	}

	vfs.write(path, contents.as_bytes())?;

	Ok(properties)
}
