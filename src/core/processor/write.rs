use anyhow::{anyhow, Context as AnyhowContext, Result};
use log::{trace, warn};
use rbx_dom_weak::types::{Ref, Variant};
use std::{
	collections::HashMap,
	path::{Path, PathBuf},
};

use crate::{
	core::{
		meta::{Context, SourceEntry, SourceKind},
		snapshot::{AddedSnapshot, UpdatedSnapshot},
		tree::Tree,
	},
	ext::PathExt,
	middleware::{data::WritableData, FileType},
	project::{self, Project},
	util,
	vfs::Vfs,
};

pub fn apply_addition(snapshot: AddedSnapshot, _tree: &mut Tree, _vfs: &Vfs) {
	println!("Added {:#?}", snapshot);
}

pub fn apply_update(snapshot: UpdatedSnapshot, tree: &mut Tree, vfs: &Vfs) -> Result<()> {
	trace!("Updating {:?}", snapshot.id);

	if !tree.exists(snapshot.id) {
		warn!("Attempted to update instance that doesn't exist: {:?}", snapshot.id);
		return Ok(());
	}

	let meta = tree.get_meta(snapshot.id).unwrap().clone();
	let instance = tree.get_instance_mut(snapshot.id).unwrap();

	let update_paths = |new_name: &str| -> Result<()> {
		for entry in meta.source.relevants() {
			match entry {
				SourceEntry::Project(_) => continue,
				_ => {
					let path = entry.path();
					let name = path.get_name().replace(&instance.name, new_name);

					vfs.rename(path, &path.get_parent().join(name))?;
				}
			}
		}

		Ok(())
	};

	match meta.source.get() {
		SourceKind::Path(path) => {
			if let Some(name) = snapshot.name {
				update_paths(&name)?;
				instance.name = name;
			}

			if let Some(properties) = snapshot.properties {
				// Temporary solution for serde failing to decode empty HashMap
				let properties = if properties.contains_key("ArgonEmpty") {
					HashMap::new()
				} else {
					properties
				};

				if util::is_script(&instance.class) {
					let source = properties_to_source(&properties);
					vfs.write(path, source.as_bytes())?;
				} else {
					let data_path = if let Some(path) = meta.source.get_data() {
						Some(path.path().to_owned())
					} else {
						locate_instance_data(&instance.name, path, &meta.context, vfs)
					};

					if let Some(data_path) = data_path {
						if !properties.is_empty() {
							let data = WritableData {
								class_name: Some(instance.class.clone()),
								properties: properties.clone(),
								..WritableData::default()
							};

							vfs.write(&data_path, &serde_json::to_vec_pretty(&data)?)?;
						} else if vfs.exists(&data_path) {
							vfs.remove(&data_path)?;
						}
					// TODO: Update tree meta
					} else {
						warn!("Failed to locate instance data for {:?}", snapshot.id);
					}
				}

				instance.properties = properties;
			}

			if let Some(_class) = snapshot.class {
				// You can't change the class of an instance inside Roblox Studio
				unreachable!()
			}

			if let Some(_meta) = snapshot.meta {
				// Currently Argon client does not update meta
				unreachable!()
			}
		}
		SourceKind::Project(name, path, _node, node_path) => {
			let mut project = Project::load(path)?;

			if let Some(new_name) = snapshot.name {
				let parent_node = project::find_node_by_path(&mut project, &node_path.parent())
					.with_context(|| format!("Failed to find project node with path {:?}", node_path.parent()))?;

				let node = parent_node
					.tree
					.remove(name)
					.context(format!("Failed to remove project node with path {:?}", node_path))?;

				parent_node.tree.insert(new_name.clone(), node);
				instance.name = new_name;

				// TODO: Update tree meta
			}

			project.save(path)?;
			if let Some(_class) = snapshot.class {
				// You can't change the class of an instance inside Roblox Studio
				unreachable!()
			}

			if let Some(_meta) = snapshot.meta {
				// Currently Argon client does not update meta
				unreachable!()
			}
		}
		SourceKind::None => panic!("Attempted to update instance with no source: {:?}", snapshot.id),
	}

	Ok(())
}

pub fn apply_removal(id: Ref, tree: &mut Tree, vfs: &Vfs) -> Result<()> {
	trace!("Removing {:?}", id);

	if !tree.exists(id) {
		warn!("Attempted to remove instance that doesn't exist: {:?}", id);
		return Ok(());
	}

	let meta = tree.get_meta(id).unwrap();

	match meta.source.get() {
		SourceKind::Path(_) => {
			let mut path_len = None;

			for entry in meta.source.relevants() {
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
		SourceKind::Project(name, path, _node, node_path) => {
			let mut project = Project::load(path)?;
			let node = project::find_node_by_path(&mut project, &node_path.parent());

			node.and_then(|node| node.tree.remove(name)).ok_or(anyhow!(
				"Failed to remove instance {:?} from project: {:?}",
				id,
				project
			))?;

			project.save(path)?;
		}
		SourceKind::None => panic!("Attempted to remove instance with no source: {:?}", id),
	}

	tree.remove_instance(id);

	Ok(())
}

fn properties_to_source(properties: &HashMap<String, Variant>) -> String {
	let (mut header, source) = if let Some(Variant::String(source)) = properties.get("Source") {
		if let Some(new_line) = source.find('\n') {
			let (header, source) = source.split_at(new_line);
			(header.to_string(), source.to_string())
		} else {
			(String::new(), source.to_owned())
		}
	} else {
		(String::new(), String::new())
	};

	let mut new_header = String::new();

	if properties.get("Disabled").is_some() {
		new_header += "--disable ";
	}

	new_header.pop();

	if let Some(Variant::Enum(run_context)) = properties.get("RunContext") {
		match run_context.to_u32() {
			1 => new_header += "--server ",
			2 => new_header += "--client ",
			3 => new_header += "--plugin ",
			_ => {}
		}
	}

	header = header.replace("--disable", "");
	header = header.replace("--server", "");
	header = header.replace("--client", "");
	header = header.replace("--plugin", "");

	if header.len() == header.match_indices(' ').count() {
		header.clear();
	}

	new_header += &header;

	if !new_header.is_empty() && !source.starts_with('\n') {
		new_header += "\n";
	}

	new_header + &source
}

fn locate_instance_data(name: &str, path: &Path, context: &Context, vfs: &Vfs) -> Option<PathBuf> {
	for sync_rule in context.sync_rules_of_type(&FileType::InstanceData) {
		if let Some(data_path) = sync_rule.locate_data(path, name, vfs.is_dir(path)) {
			return Some(data_path);
		}
	}

	None
}
