use std::path::Path;

use log::{error, trace};
use rbx_dom_weak::types::Ref;

use crate::{
	core::{
		changes::Changes,
		meta::SourceKind,
		snapshot::{Snapshot, UpdatedSnapshot},
		tree::Tree,
	},
	middleware::{new_snapshot, project::new_snapshot_node},
	stats, util,
	vfs::Vfs,
};

pub fn process_changes(id: Ref, tree: &mut Tree, vfs: &Vfs) -> Option<Changes> {
	trace!("Processing changes for instance: {:?}", id);

	let mut changes = Changes::new();

	let meta = tree.get_meta(id)?;
	let source = meta.source.get();

	let process_path = |path: &Path| -> Option<Option<Snapshot>> {
		match new_snapshot(path, &meta.context, vfs) {
			Ok(snapshot) => Some(snapshot),
			Err(err) => {
				error!("Failed to process changes: {}, source: {:?}", err, source);
				None
			}
		}
	};

	let snapshot = match source {
		SourceKind::Project(name, path, node, node_path) => {
			if node_path.is_root() {
				process_path(path)?
			} else {
				match new_snapshot_node(name, path, *node.clone(), node_path.clone(), &meta.context, vfs) {
					Ok(snapshot) => Some(snapshot),
					Err(err) => {
						error!("Failed to process changes: {}, source: {:?}", err, source);
						return Some(changes);
					}
				}
			}
		}
		SourceKind::Path(path) => process_path(path)?,
		SourceKind::None => panic!(
			"Fatal processing error: `SourceKind::None` should not be present in the tree! Id: {:?}, meta: {:#?}",
			id, meta
		),
	};

	// Handle additions, modifications and child removals
	if let Some(snapshot) = snapshot {
		process_child_changes(id, snapshot, &mut changes, tree);
	// Handle regular removals
	} else {
		tree.remove_instance(id);
		changes.remove(id);
	}

	Some(changes)
}

fn process_child_changes(id: Ref, mut snapshot: Snapshot, changes: &mut Changes, tree: &mut Tree) {
	// Process instance changes
	let mut updated_snapshot = UpdatedSnapshot::new(id);

	updated_snapshot.meta = if snapshot.meta != *tree.get_meta(id).expect("Instance meta not found") {
		tree.update_meta(id, snapshot.meta.clone());
		Some(snapshot.meta)
	} else {
		None
	};

	let instance = tree.get_instance_mut(id).unwrap();

	updated_snapshot.name = if snapshot.name != instance.name {
		instance.name.clone_from(&snapshot.name);
		Some(snapshot.name)
	} else {
		None
	};

	updated_snapshot.class = if snapshot.class != instance.class {
		instance.class.clone_from(&snapshot.class);
		Some(snapshot.class)
	} else {
		None
	};

	updated_snapshot.properties = if snapshot.properties != instance.properties {
		instance.properties.clone_from(&snapshot.properties);
		Some(snapshot.properties)
	} else {
		None
	};

	if !updated_snapshot.is_empty() {
		// Track `lines_synced` stat
		if let Some(properties) = &updated_snapshot.properties {
			let loc = util::count_loc_from_properties(properties);
			stats::lines_synced(loc as u32);
		}

		changes.update(updated_snapshot);
	}

	let mut hydrated = vec![false; snapshot.children.len()];

	// Pair instances and find removed children
	#[allow(clippy::unnecessary_to_owned)]
	for child_id in instance.children().to_owned() {
		let instance = tree.get_instance(child_id).unwrap();

		let snapshot = snapshot.children.iter_mut().enumerate().find(|(index, child)| {
			if hydrated[*index] {
				return false;
			}

			if child.name == instance.name && child.class == instance.class {
				hydrated[*index] = true;
				return true;
			}

			false
		});

		if let Some((_, child)) = snapshot {
			child.set_id(child_id);
		} else {
			tree.remove_instance(child_id);
			changes.remove(child_id);
		}
	}

	// Process child changes and find new children
	for child in snapshot.children {
		if child.id.is_some() {
			process_child_changes(child.id, child, changes, tree);
		} else {
			let mut child = child;

			insert_children(&mut child, id, tree);

			// Track `lines_synced` stat
			let loc = util::count_loc_from_properties(&child.properties);
			stats::lines_synced(loc as u32);

			changes.add(child, id);
		}
	}
}

fn insert_children(snapshot: &mut Snapshot, parent: Ref, tree: &mut Tree) {
	let id = tree.insert_instance(snapshot.clone(), parent);

	snapshot.set_id(id);

	for child in snapshot.children.iter_mut() {
		insert_children(child, id, tree);
	}
}
