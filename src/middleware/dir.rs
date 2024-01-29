use anyhow::Result;
use std::path::Path;

use crate::{
	core::{meta::Meta, snapshot::Snapshot},
	util,
	vfs::Vfs,
};

use super::new_snapshot;

#[profiling::function]
pub fn snapshot_dir(path: &Path, meta: &Meta, vfs: &Vfs) -> Result<Option<Snapshot>> {
	let name = util::get_file_name(path);
	let mut snapshot = Snapshot::new(name).with_path(path);

	for path in vfs.read_dir(path)? {
		if let Some(child_snapshot) = new_snapshot(&path, meta, vfs)? {
			snapshot.children.push(child_snapshot);
		}
	}

	Ok(Some(snapshot))
}
