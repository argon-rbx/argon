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

#[profiling::function]
pub fn process_changes(id: Ref, tree: &mut Tree, vfs: &Vfs) -> Changes {
	profiling::start_frame!();
	trace!("Processing changes for instance: {:?}", id);

	let mut changes = Changes::new();

	let meta = tree.get_meta(id).unwrap();
	let source = meta.source.get();

	let snapshot = match source {
		SourceKind::Project(name, path, node, node_path) => {
			match new_snapshot_node(name, path, node.clone(), node_path.clone(), &meta.context, vfs) {
				Ok(snapshot) => Some(snapshot),
				Err(err) => {
					error!("Failed to process changes: {}, source: {:?}", err, source);
					return changes;
				}
			}
		}
		SourceKind::Path(path) => match new_snapshot(path, &meta.context, vfs) {
			Ok(snapshot) => snapshot,
			Err(err) => {
				error!("Failed to process changes: {}, source: {:?}", err, source);
				return changes;
			}
		},
		SourceKind::None => panic!(
			"Fatal processing error: `SourceKind::None` should not be present in the tree! Id: {:#?}, meta: {:#?}",
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

	changes
}

fn process_child_changes(id: Ref, mut snapshot: Snapshot, changes: &mut Changes, tree: &mut Tree) {
	// Update meta if it's different
	match (snapshot.meta, tree.get_meta(id)) {
		(snapshot_meta, Some(meta)) => {
			if snapshot_meta != *meta {
				tree.update_meta(id, snapshot_meta);
			}
		}
		(snapshot_meta, None) => {
			tree.insert_meta(id, snapshot_meta);
		}
	}

	// Process instance changes

	let instance = tree.get_instance_mut(id).unwrap();

	let mut updated_snapshot = UpdatedSnapshot::new(id);

	updated_snapshot.name = if snapshot.name != instance.name {
		instance.name = snapshot.name.clone();
		Some(snapshot.name)
	} else {
		None
	};

	updated_snapshot.class = if snapshot.class != instance.class {
		instance.class = snapshot.class.clone();
		Some(snapshot.class)
	} else {
		None
	};

	updated_snapshot.properties = if snapshot.properties != instance.properties {
		instance.properties = snapshot.properties.clone();
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
	let referent = tree.insert_instance_non_recursive(snapshot.clone(), parent);

	snapshot.set_id(referent);

	for child in snapshot.children.iter_mut() {
		insert_children(child, referent, tree);
	}
}
