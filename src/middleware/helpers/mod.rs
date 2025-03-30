use rbx_dom_weak::{types::Ref, WeakDom};

use crate::{
	core::{helpers::apply_migrations, snapshot::Snapshot},
	Properties,
};

mod markdown;
mod mesh_part;
mod snapshot;

#[inline]
pub fn save_mesh(properties: &Properties) -> Option<String> {
	let mut properties = properties.clone();
	apply_migrations("MeshPart", &mut properties);

	mesh_part::save_mesh(&properties)
}

#[inline]
pub fn snapshot_from_dom(dom: WeakDom, id: Ref) -> Snapshot {
	snapshot::snapshot_from_dom(dom, id)
}

#[inline]
pub fn markdown_to_rich_text(text: &str) -> String {
	markdown::parse(text)
}
