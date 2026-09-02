use anyhow::Result;
use rbx_dom_weak::{types::Variant, ustr, HashMapExt, UstrMap};
use std::path::Path;

use crate::{core::snapshot::Snapshot, middleware::helpers::markdown_to_rich_text, vfs::Vfs};

#[profiling::function]
pub fn read_md(path: &Path, vfs: &Vfs) -> Result<Snapshot> {
	let markdown = vfs.read_to_string(path)?;
	let rich_text = markdown_to_rich_text(&markdown);

	let mut properties = UstrMap::new();
	properties.insert(ustr("Value"), Variant::String(rich_text));

	Ok(Snapshot::new().with_class("StringValue").with_properties(properties))
}
