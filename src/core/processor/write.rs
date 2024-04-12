use anyhow::Result;
use log::{trace, warn};
use rbx_dom_weak::types::Ref;

use crate::{
	core::{
		meta::{SourceEntry, SourceKind},
		snapshot::{AddedSnapshot, UpdatedSnapshot},
		tree::Tree,
	},
	ext::PathExt,
	vfs::Vfs,
};

pub fn apply_addition(snapshot: AddedSnapshot, _tree: &mut Tree, _vfs: &Vfs) {
	println!("Added {:#?}", snapshot);
}

pub fn apply_update(snapshot: UpdatedSnapshot, _tree: &mut Tree, _vfs: &Vfs) {
	println!("Updated {:#?}", snapshot);
}

pub fn apply_removal(id: Ref, tree: &mut Tree, vfs: &Vfs) -> Result<()> {
	trace!("Removing {:#?}", id);

	if !tree.exists(id) {
		warn!("Attempted to remove instance that doesn't exist: {:?}", id);
		return Ok(());
	}

	if let Some(meta) = tree.get_meta(id) {
		match meta.source.get() {
			SourceKind::Path(_) => {
				let mut path_len = None;

				for entry in meta.source.relevants() {
					println!("{:#?}", entry);
					match entry {
						SourceEntry::Project(_) => continue,
						SourceEntry::Folder(path) => {
							path_len = Some(path.len());
							vfs.remove(path)?
						}
						SourceEntry::File(path) | SourceEntry::Data(path) => {
							if let Some(len) = path_len {
								if path.len() == len {
									vfs.remove(path)?
								}
							} else {
								vfs.remove(path)?
							}
						}
					}
				}
			}
			SourceKind::Project(_, _, _) => {
				// TODO
			}
			SourceKind::None => panic!("Attempted to remove instance with no source: {:?}", id),
		}
	}

	tree.remove_instance(id);

	Ok(())
}
