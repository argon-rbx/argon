use anyhow::Result;
use rbx_dom_weak::types::Variant;
use std::{collections::HashMap, path::Path};

use crate::{core::snapshot::Snapshot, vfs::Vfs};

#[profiling::function]
pub fn snapshot_txt(path: &Path, vfs: &Vfs) -> Result<Snapshot> {
	let value = vfs.read(path)?;

	let mut properties = HashMap::new();
	properties.insert(String::from("Value"), Variant::String(value));

	Ok(Snapshot::new().with_class("StringValue").with_properties(properties))
}
