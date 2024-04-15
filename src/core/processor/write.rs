use anyhow::{anyhow, Context as AnyhowContext, Result};
use log::{error, trace, warn};
use rbx_dom_weak::{types::Ref, Instance};
use std::{
	collections::HashMap,
	path::{Path, PathBuf},
};

use crate::{
	core::{
		meta::{Meta, SourceEntry, SourceKind},
		snapshot::{AddedSnapshot, UpdatedSnapshot},
		tree::Tree,
	},
	ext::PathExt,
	middleware::{data, Middleware},
	project::Project,
	resolution::UnresolvedValue,
	vfs::Vfs,
	Properties,
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

	let mut meta = tree.get_meta(snapshot.id).unwrap().clone();
	let instance = tree.get_instance_mut(snapshot.id).unwrap();

	fn update_non_project_properties(
		properties: Properties,
		instance: &mut Instance,
		meta: &mut Meta,
		path: &Path,
		vfs: &Vfs,
	) -> Result<()> {
		let properties = validate_properties(properties);

		if let Some(middleware) = Middleware::from_class(&instance.class) {
			let file_path = if let Some(entry) = meta.source.get_file() {
				Some(entry.path().to_owned())
			} else {
				let mut file_path = None;

				for sync_rule in meta.context.sync_rules_of_type(&middleware) {
					if let Some(path) = sync_rule.locate(path, &instance.name, vfs.is_dir(path)) {
						file_path = Some(path);
						break;
					}
				}

				if let Some(file_path) = &file_path {
					meta.source.add_file(file_path);
				}

				file_path
			};

			if let Some(file_path) = file_path {
				let properties = middleware.write(properties.clone(), &file_path, vfs)?;

				if let Some(data_path) = locate_instance_data(&instance.name, &file_path, meta, vfs) {
					let data_path = data::write_data(true, &instance.class, properties, &data_path, meta, vfs)?;
					meta.source.set_data(data_path)
				}
			} else {
				error!("Failed to locate file for path {:?}", path.display());
			}
		} else if let Some(data_path) = locate_instance_data(&instance.name, path, meta, vfs) {
			let data_path = data::write_data(false, &instance.class, properties.clone(), &data_path, meta, vfs)?;
			meta.source.set_data(data_path)
		}

		instance.properties = properties;

		Ok(())
	}

	match meta.source.get().clone() {
		SourceKind::Path(path) => {
			if let Some(name) = snapshot.name {
				let new_path = path.get_parent().join(path.get_name().replace(&instance.name, &name));
				*meta.source.get_mut() = SourceKind::Path(new_path.clone());

				for mut entry in meta.source.relevants_mut() {
					match &mut entry {
						SourceEntry::Project(_) => continue,
						SourceEntry::File(path) | SourceEntry::Folder(path) | SourceEntry::Data(path) => {
							let name = path.get_name().replace(&instance.name, &name);
							let new_path = path.get_parent().join(name);

							vfs.rename(path, &new_path)?;

							*path = new_path;
						}
					}
				}

				instance.name = name;
			}

			if let Some(properties) = snapshot.properties {
				update_non_project_properties(properties, instance, &mut meta, &path, vfs)?;
			}

			tree.update_meta(snapshot.id, meta);

			if let Some(_class) = snapshot.class {
				// You can't change the class of an instance inside Roblox Studio
				unreachable!()
			}

			if let Some(_meta) = snapshot.meta {
				// Currently Argon client does not update meta
				unreachable!()
			}
		}
		SourceKind::Project(name, path, node, node_path) => {
			let mut project = Project::load(&path)?;

			if let Some(properties) = snapshot.properties {
				if let Some(custom_path) = node.path {
					let path = path_clean::clean(path.get_parent().join(custom_path));

					update_non_project_properties(properties, instance, &mut meta, &path, vfs)?;

					let node = project
						.find_node_by_path(&node_path)
						.context(format!("Failed to find project node with path {:?}", node_path))?;

					node.properties = HashMap::new();
					node.attributes = None;
					node.tags = vec![];
					node.keep_unknowns = None;
				} else {
					let node = project
						.find_node_by_path(&node_path)
						.context(format!("Failed to find project node with path {:?}", node_path))?;

					let class = node.class_name.as_ref().unwrap_or(&name);
					let properties = validate_properties(properties);

					node.properties = properties
						.clone()
						.iter()
						.map(|(property, varaint)| {
							(
								property.to_owned(),
								UnresolvedValue::from_variant(varaint.clone(), class, property),
							)
						})
						.collect();

					node.tags = vec![];
					node.keep_unknowns = None;

					instance.properties = properties;
				}
			}

			if let Some(new_name) = snapshot.name {
				let parent_node = project.find_node_by_path(&node_path.parent()).with_context(|| {
					format!("Failed to find parent project node with path {:?}", node_path.parent())
				})?;

				let node = parent_node
					.tree
					.remove(&name)
					.context(format!("Failed to remove project node with path {:?}", node_path))?;

				parent_node.tree.insert(new_name.clone(), node.clone());

				let node_path = node_path.parent().join(&new_name);

				*meta.source.get_mut() = SourceKind::Project(new_name.clone(), path.clone(), node, node_path);

				instance.name = new_name;
			}

			tree.update_meta(snapshot.id, meta);
			project.save(&path)?;

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
			let node = project.find_node_by_path(&node_path.parent());

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

fn locate_instance_data(name: &str, path: &Path, meta: &Meta, vfs: &Vfs) -> Option<PathBuf> {
	let data_path = if let Some(entry) = meta.source.get_data() {
		Some(entry.path().to_owned())
	} else {
		let mut data_path = None;

		for sync_rule in meta.context.sync_rules_of_type(&Middleware::InstanceData) {
			if let Some(path) = sync_rule.locate(path, name, vfs.is_dir(path)) {
				data_path = Some(path);
				break;
			}
		}

		data_path
	};

	if data_path.is_none() {
		warn!("Failed to locate instance data for {}", path.display())
	}

	data_path
}

// Temporary solution for serde failing to deserialize empty HashMap
fn validate_properties(properties: Properties) -> Properties {
	if properties.contains_key("ArgonEmpty") {
		HashMap::new()
	} else {
		properties
	}
}
