use anyhow::Result;
use std::path::Path;

use super::new_snapshot;
use crate::{
	core::{meta::Meta, snapshot::Snapshot},
	util,
	vfs::Vfs,
};

#[profiling::function]
pub fn snapshot_dir(path: &Path, meta: &Meta, vfs: &Vfs) -> Result<Snapshot> {
	let name = util::get_file_name(path);
	let mut snapshot = Snapshot::new().with_name(name).with_path(path);

	let mut meta = meta.clone();

	for path in vfs.read_dir(path)? {
		if let Some(child_snapshot) = new_snapshot(&path, &meta, vfs)? {
			if let Some(snapshot_meta) = &child_snapshot.meta {
				meta.extend(snapshot_meta.clone());
			}

			snapshot.add_child(child_snapshot);
		}
	}

	Ok(snapshot)
}
