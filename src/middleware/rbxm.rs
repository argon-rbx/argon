use anyhow::Result;
use std::path::Path;

use super::helpers;
use crate::{core::snapshot::Snapshot, vfs::Vfs};

#[profiling::function]
pub fn read_rbxm(path: &Path, vfs: &Vfs) -> Result<Snapshot> {
	let dom = rbx_binary::from_reader(vfs.read(path)?.as_slice())?;

	let snapshot = if dom.root().children().len() == 1 {
		let id = dom.root().children()[0];
		helpers::snapshot_from_dom(dom, id)
	} else {
		let id = dom.root_ref();
		helpers::snapshot_from_dom(dom, id).with_class("Folder")
	};

	Ok(snapshot)
}
