use anyhow::Result;
use std::path::Path;

use super::new_snapshot;
use crate::{
	core::{meta::Meta, snapshot::Snapshot},
	ext::PathExt,
	vfs::Vfs,
};

#[profiling::function]
pub fn snapshot_dir(path: &Path, meta: &Meta, vfs: &Vfs) -> Result<Snapshot> {
	let name = path.get_file_name();
	let mut snapshot = Snapshot::new().with_name(name).with_path(path);

	for path in vfs.read_dir(path)? {
		if let Some(child_snapshot) = new_snapshot(&path, meta, vfs)? {
			snapshot.add_child(child_snapshot);
		}
	}

	Ok(snapshot)
}
