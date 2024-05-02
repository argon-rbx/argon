use anyhow::Result;
use std::path::Path;

use super::new_snapshot;
use crate::{
	core::{
		meta::{Context, Meta, Source},
		snapshot::Snapshot,
	},
	ext::PathExt,
	vfs::Vfs,
};

#[profiling::function]
pub fn read_dir(path: &Path, context: &Context, vfs: &Vfs) -> Result<Snapshot> {
	let name = path.get_name();

	let mut snapshot = Snapshot::new()
		.with_name(name)
		.with_meta(Meta::new().with_context(context).with_source(Source::directory(path)));

	for path in vfs.read_dir(path)? {
		if let Some(child_snapshot) = new_snapshot(&path, context, vfs)? {
			snapshot.add_child(child_snapshot);
		}
	}

	Ok(snapshot)
}

#[profiling::function]
pub fn write_dir(path: &Path, vfs: &Vfs) -> Result<()> {
	vfs.create_dir(path)?;

	Ok(())
}
