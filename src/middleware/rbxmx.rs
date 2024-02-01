use anyhow::Result;
use std::path::Path;

use crate::{core::snapshot::Snapshot, vfs::Vfs};

#[profiling::function]
pub fn snapshot_rbxmx(path: &Path, vfs: &Vfs) -> Result<Snapshot> {
	let reader = vfs.reader(path)?;
	let dom = rbx_xml::from_reader_default(reader)?;

	let snapshot = if dom.root().children().len() == 1 {
		let id = dom.root().children()[0];
		Snapshot::from_dom(dom, id)
	} else {
		let id = dom.root_ref();
		Snapshot::from_dom(dom, id).with_class("Folder")
	};

	Ok(snapshot)
}
