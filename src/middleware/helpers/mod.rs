use rbx_dom_weak::{types::Ref, WeakDom};

use crate::{core::snapshot::Snapshot, Properties};

mod mesh_part;
mod snapshot;

#[inline]
pub fn save_mesh(properties: &mut Properties) -> Option<String> {
	mesh_part::save_mesh(properties)
}

#[inline]
pub fn snapshot_from_dom(dom: WeakDom, id: Ref) -> Snapshot {
	snapshot::snapshot_from_dom(dom, id)
}
