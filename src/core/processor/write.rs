use anyhow::{anyhow, Context as AnyhowContext, Result};
use log::{error, trace, warn};
use path_clean::PathClean;
use rbx_dom_weak::{types::Ref, HashMapExt, Instance, Ustr, UstrMap};
use std::path::{Path, PathBuf};

use crate::{
	config::Config,
	core::{
		helpers::syncback::{rename_path, serialize_properties, validate_properties, verify_name, verify_path},
		meta::{Meta, NodePath, Source, SourceEntry, SourceKind},
		snapshot::{AddedSnapshot, Snapshot, UpdatedSnapshot},
		tree::Tree,
	},
	ext::PathExt,
	middleware::{
		data::{self, write_original_name},
		dir, Middleware,
	},
	project::{Project, ProjectNode},
	vfs::Vfs,
	Properties,
};

macro_rules! filter_warn {
	($id:expr) => {
		warn!("Instance {} does not pass syncback filter! Skipping..", $id);
	};
	($id:expr, $path:expr) => {
		warn!(
			"Path: {} (source of instance: {}) does not pass syncback filter! Skipping..",
			$path.display(),
			$id
		);
	};
}

pub fn apply_addition(snapshot: AddedSnapshot, tree: &mut Tree, vfs: &Vfs) -> Result<()> {
	trace!("Adding {:?} with parent {:?}", snapshot.id, snapshot.parent);

	if !tree.exists(snapshot.parent) {
		warn!(
			"Attempted to add instance: {:?} whose parent doesn't exist: {:?}",
			snapshot.id, snapshot.parent
		);
		return Ok(());
	}

	let parent_id = snapshot.parent;
	let mut snapshot = Snapshot::from(snapshot);
	let mut parent_meta = tree.get_meta(parent_id).unwrap().clone();
	let filter = parent_meta.context.syncback_filter();

	if filter.matches_name(&snapshot.name) || filter.matches_class(&snapshot.class) {
		filter_warn!(snapshot.id);
		return Ok(());
	}

	snapshot.properties = validate_properties(snapshot.properties, filter);

	fn locate_instance_data(is_dir: bool, path: &Path, snapshot: &Snapshot, parent_meta: &Meta) -> Result<PathBuf> {
		parent_meta
			.context
			.sync_rules_of_type(&Middleware::InstanceData, true)
			.iter()
			.find_map(|rule| rule.locate(path, &snapshot.name, is_dir))
			.with_context(|| format!("Failed to locate data path for parent: {}", path.display()))
	}

	fn write_instance(
		has_children: bool,
		path: &mut PathBuf,
		snapshot: &mut Snapshot,
		parent_meta: &Meta,
		vfs: &Vfs,
	) -> Result<Option<Meta>> {
		let mut meta = snapshot.meta.clone().with_context(&parent_meta.context);
		let filter = parent_meta.context.syncback_filter();
		let mut properties = snapshot.properties.clone();

		if let Some(middleware) = Middleware::from_class(
			&snapshot.class,
			if !parent_meta.context.use_legacy_scripts() {
				Some(&mut properties)
			} else {
				None
			},
		) {
			let mut file_path = parent_meta
				.context
				.sync_rules_of_type(&middleware, true)
				.iter()
				.find_map(|rule| rule.locate(path, &snapshot.name, has_children))
				.with_context(|| format!("Failed to locate file path for parent: {}", path.display()))?;

			if has_children {
				if filter.matches_path(path) {
					filter_warn!(snapshot.id, path);
					return Ok(None);
				}

				if !verify_path(path, &mut snapshot.name, &mut meta, vfs) {
					return Ok(None);
				}

				dir::write_dir(path, vfs)?;

				meta.set_source(Source::child_file(path, &file_path));
			} else {
				if !verify_path(&mut file_path, &mut snapshot.name, &mut meta, vfs) {
					return Ok(None);
				}

				meta.set_source(Source::file(&file_path));
			}

			if filter.matches_path(&file_path) {
				filter_warn!(snapshot.id, &file_path);
				return Ok(None);
			}

			let properties = middleware.write(properties, &file_path, vfs)?;
			let data_path = locate_instance_data(has_children, path, snapshot, parent_meta)?;

			if filter.matches_path(&data_path) {
				filter_warn!(snapshot.id, &data_path);
			} else {
				let data_path = data::write_data(true, &snapshot.class, properties, &data_path, &meta, vfs)?;
				meta.source.set_data(data_path);
			}
		} else {
			if filter.matches_path(path) {
				filter_warn!(snapshot.id, path);
				return Ok(None);
			}

			if !verify_path(path, &mut snapshot.name, &mut meta, vfs) {
				return Ok(None);
			}

			dir::write_dir(path, vfs)?;

			meta.set_source(Source::directory(path));

			let data_path = locate_instance_data(true, path, snapshot, parent_meta)?;

			if filter.matches_path(&data_path) {
				filter_warn!(snapshot.id, &data_path);
			} else {
				let data_path = data::write_data(false, &snapshot.class, properties, &data_path, &meta, vfs)?;
				meta.source.set_data(data_path);
			}
		}

		Ok(Some(meta))
	}

	fn add_non_project_instances(
		parent_id: Ref,
		parent_path: &Path,
		mut snapshot: Snapshot,
		parent_meta: &mut Meta,
		tree: &mut Tree,
		vfs: &Vfs,
	) -> Result<Source> {
		let config = Config::new();

		let mut parent_path = parent_path.to_owned();

		// Transform parent instance source from file to folder
		let parent_source = if vfs.is_file(&parent_path) {
			let sync_rule = parent_meta
				.context
				.sync_rules()
				.iter()
				.filter(|rule| {
					if let Some(pattern) = rule.child_pattern.as_ref() {
						!((pattern.as_str().starts_with(".src") || pattern.as_str().ends_with(".data.json"))
							&& config.rojo_mode)
					} else {
						true
					}
				})
				.find(|rule| rule.matches(&parent_path))
				.with_context(|| format!("Failed to find sync rule for path: {}", parent_path.display()))?
				.clone();

			let name = sync_rule.get_name(&parent_path);
			let mut folder_path = parent_path.with_file_name(&name);

			if !verify_path(&mut folder_path, &mut snapshot.name, parent_meta, vfs) {
				return Ok(parent_meta.source.clone());
			}

			let file_path = sync_rule
				.locate(&folder_path, &name, true)
				.with_context(|| format!("Failed to locate file path for parent: {}", folder_path.display()))?;

			let data_paths = if let Some(data) = parent_meta.source.get_data() {
				let new_path = parent_meta
					.context
					.sync_rules_of_type(&Middleware::InstanceData, true)
					.iter()
					.find_map(|rule| rule.locate(&folder_path, &name, true))
					.with_context(|| format!("Failed to locate data path for parent: {}", folder_path.display()))?;

				Some((data.path().to_owned(), new_path))
			} else {
				None
			};

			let mut source = Source::child_file(&folder_path, &file_path);

			dir::write_dir(&folder_path, vfs)?;
			vfs.rename(&parent_path, &file_path)?;

			if let Some(data_paths) = data_paths {
				source.add_data(&data_paths.1);
				vfs.rename(&data_paths.0, &data_paths.1)?;
			}

			parent_path = folder_path;

			source
		} else {
			parent_meta.source.clone()
		};

		if !verify_name(&mut snapshot.name, &mut snapshot.meta) {
			return Ok(parent_source);
		}

		let mut path = parent_path.join(&snapshot.name);

		if snapshot.children.is_empty() {
			if let Some(meta) = write_instance(false, &mut path, &mut snapshot, parent_meta, vfs)? {
				let snapshot = snapshot.with_meta(meta);

				tree.insert_instance_with_ref(snapshot, parent_id);
			}
		} else if let Some(mut meta) = write_instance(true, &mut path, &mut snapshot, parent_meta, vfs)? {
			let snapshot = snapshot.with_meta(meta.clone());

			tree.insert_instance_with_ref(snapshot.clone(), parent_id);

			for mut child in snapshot.children {
				child.properties = validate_properties(child.properties.clone(), meta.context.syncback_filter());
				add_non_project_instances(snapshot.id, &path, child, &mut meta, tree, vfs)?;
			}
		}

		Ok(parent_source)
	}

	fn add_project_instances(
		parent_id: Ref,
		path: &Path,
		node_path: NodePath,
		mut snapshot: Snapshot,
		parent_node: &mut ProjectNode,
		parent_meta: &Meta,
		tree: &mut Tree,
	) {
		let mut node = ProjectNode {
			class_name: Some(snapshot.class),
			properties: serialize_properties(&snapshot.class, snapshot.properties.clone()),
			..ProjectNode::default()
		};

		if snapshot.meta.keep_unknowns {
			node.keep_unknowns = Some(true);
		}

		let node_path = node_path.join(&snapshot.name);
		let source = Source::project(&snapshot.name, path, node.clone(), node_path.clone());
		let meta = snapshot
			.meta
			.clone()
			.with_context(&parent_meta.context)
			.with_source(source);

		snapshot.meta = meta;
		tree.insert_instance_with_ref(snapshot.clone(), parent_id);

		let filter = snapshot.meta.context.syncback_filter();

		for mut child in snapshot.children {
			child.properties = validate_properties(child.properties, filter);
			add_project_instances(parent_id, path, node_path.clone(), child, &mut node, parent_meta, tree);
		}

		parent_node.tree.insert(snapshot.name, node);
	}

	match parent_meta.source.get().clone() {
		SourceKind::Path(path) => {
			let parent_source = add_non_project_instances(parent_id, &path, snapshot, &mut parent_meta, tree, vfs)?;

			parent_meta.set_source(parent_source);
			tree.update_meta(parent_id, parent_meta);
		}
		SourceKind::Project(name, path, node, node_path) => {
			if let Some(custom_path) = &node.path {
				let custom_path = path.with_file_name(custom_path.path()).clean();

				let parent_source =
					add_non_project_instances(parent_id, &custom_path, snapshot, &mut parent_meta, tree, vfs)?;

				let parent_source = Source::project(&name, &path, *node, node_path.clone())
					.with_relevant(parent_source.relevant().to_owned());

				parent_meta.set_source(parent_source);
				tree.update_meta(parent_id, parent_meta);
			} else {
				let mut project = Project::load(&path)?;

				let node = project
					.find_node_by_path(&node_path)
					.context(format!("Failed to find project node with path {:?}", node_path))?;

				add_project_instances(parent_id, &path, node_path.clone(), snapshot, node, &parent_meta, tree);

				project.save(&path)?;
			}
		}
		SourceKind::None => panic!(
			"Attempted to add instance whose parent has no source: {:?}",
			snapshot.id
		),
	}

	Ok(())
}

pub fn apply_update(snapshot: UpdatedSnapshot, tree: &mut Tree, vfs: &Vfs) -> Result<()> {
	trace!("Updating {:?}", snapshot.id);

	if let Some(instance) = tree.get_instance(snapshot.id) {
		let filter = tree.get_meta(snapshot.id).unwrap().context.syncback_filter();

		if filter.matches_name(&instance.name) || filter.matches_class(&instance.class) {
			filter_warn!(snapshot.id);
			return Ok(());
		}

		if snapshot.name.as_ref().is_some_and(|name| filter.matches_name(name)) {
			filter_warn!(snapshot.id);
			return Ok(());
		}

		if snapshot.class.as_ref().is_some_and(|class| filter.matches_class(class)) {
			filter_warn!(snapshot.id);
			return Ok(());
		}
	} else {
		warn!("Attempted to update instance that doesn't exist: {:?}", snapshot.id);
		return Ok(());
	}

	let mut meta = tree.get_meta(snapshot.id).unwrap().clone();
	let instance = tree.get_instance_mut(snapshot.id).unwrap();

	fn locate_instance_data(name: &str, path: &Path, meta: &Meta, vfs: &Vfs) -> Option<PathBuf> {
		let data_path = if let Some(data) = meta.source.get_data() {
			Some(data.path().to_owned())
		} else {
			meta.context
				.sync_rules_of_type(&Middleware::InstanceData, true)
				.iter()
				.find_map(|rule| rule.locate(path, name, vfs.is_dir(path)))
		};

		if data_path.is_none() {
			warn!("Failed to locate instance data for {}", path.display())
		}

		data_path
	}

	fn update_non_project_properties(
		path: &Path,
		properties: Properties,
		instance: &mut Instance,
		meta: &mut Meta,
		vfs: &Vfs,
	) -> Result<()> {
		let filter = meta.context.syncback_filter();

		if filter.matches_path(path) {
			filter_warn!(instance.referent(), path);
			return Ok(());
		}

		let mut properties = validate_properties(properties, filter);

		if let Some(middleware) = Middleware::from_class(
			&instance.class,
			if !meta.context.use_legacy_scripts() {
				Some(&mut properties)
			} else {
				None
			},
		) {
			let new_path = {
				let mut paths = meta
					.context
					.sync_rules_of_type(&middleware, true)
					.iter()
					.filter_map(|rule| rule.locate(path, &instance.name, vfs.is_dir(path)))
					.collect::<Vec<PathBuf>>();

				paths.sort_by_key(|path| !path.exists());
				paths.first().map(|path| path.to_owned())
			};

			let file_path = if let Some(SourceEntry::File(path)) = meta.source.get_file_mut() {
				let mut current_path = path.to_owned();

				if let Some(new_path) = new_path {
					if current_path != new_path {
						vfs.rename(&current_path, &new_path)?;

						*path = new_path.clone();
						current_path = new_path;
					}
				}

				Some(current_path)
			} else {
				if let Some(new_path) = &new_path {
					meta.source.add_file(new_path);
				}

				new_path
			};

			if let Some(file_path) = file_path {
				let properties = middleware.write(properties.clone(), &file_path, vfs)?;

				if let Some(data_path) = locate_instance_data(&instance.name, path, meta, vfs) {
					if filter.matches_path(&data_path) {
						filter_warn!(instance.referent(), &data_path);
					} else {
						let data_path = data::write_data(true, &instance.class, properties, &data_path, meta, vfs)?;
						meta.source.set_data(data_path)
					}
				}
			} else {
				error!("Failed to locate file for path {:?}", path.display());
			}
		} else if let Some(data_path) = locate_instance_data(&instance.name, path, meta, vfs) {
			if filter.matches_path(&data_path) {
				filter_warn!(instance.referent(), &data_path);
			} else {
				let data_path = data::write_data(false, &instance.class, properties.clone(), &data_path, meta, vfs)?;
				meta.source.set_data(data_path)
			}
		}

		instance.properties = properties;

		Ok(())
	}

	match meta.source.get().clone() {
		SourceKind::Path(mut path) => {
			if let Some(mut name) = snapshot.name {
				let original_name = meta.original_name.clone();

				if !verify_name(&mut name, &mut meta) {
					return Ok(());
				}

				path = rename_path(&path, &instance.name, &name);

				if !verify_path(&mut path, &mut name, &mut meta, vfs) {
					return Ok(());
				}

				*meta.source.get_mut() = SourceKind::Path(path.clone());

				let filter = meta.context.syncback_filter();

				if let Some(SourceEntry::Folder(path)) = meta.source.get_folder_mut() {
					let new_path = path.with_file_name(&name);

					if filter.matches_path(path) && filter.matches_path(&new_path) {
						filter_warn!(snapshot.id, path);
					} else {
						vfs.rename(path, &new_path)?;
						*path = new_path.clone();

						for mut entry in meta.source.relevant_mut() {
							match &mut entry {
								SourceEntry::File(path) | SourceEntry::Data(path) => {
									*path = new_path.join(path.get_name());
								}
								_ => continue,
							}
						}
					}
				} else {
					for mut entry in meta.source.relevant_mut() {
						match &mut entry {
							SourceEntry::File(path) | SourceEntry::Data(path) => {
								let new_path = rename_path(path, &instance.name, &name);

								if filter.matches_path(path) && filter.matches_path(&new_path) {
									filter_warn!(snapshot.id, path);
									continue;
								}

								vfs.rename(path, &new_path)?;
								*path = new_path;
							}
							_ => continue,
						}
					}
				}

				if original_name != meta.original_name && snapshot.properties.is_none() {
					if let Some(data_path) = locate_instance_data(&name, &path, &meta, vfs) {
						if filter.matches_path(&data_path) {
							filter_warn!(instance.referent(), &data_path);
						} else {
							write_original_name(&data_path, &meta, vfs)?;
						}
					}
				}

				instance.name = meta.original_name.clone().unwrap_or(name);
			}

			if let Some(properties) = snapshot.properties {
				update_non_project_properties(&path, properties, instance, &mut meta, vfs)?;
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
					let custom_path = path.with_file_name(custom_path.path()).clean();

					update_non_project_properties(&custom_path, properties, instance, &mut meta, vfs)?;

					let node = project
						.find_node_by_path(&node_path)
						.context(format!("Failed to find project node with path {:?}", node_path))?;

					node.properties = UstrMap::new();
					node.attributes = None;
					node.tags = Vec::new();
					node.keep_unknowns = None;
				} else {
					let node = project
						.find_node_by_path(&node_path)
						.context(format!("Failed to find project node with path {:?}", node_path))?;

					let class = node.class_name.unwrap_or(Ustr::from(&name));
					let properties = validate_properties(properties, meta.context.syncback_filter());

					node.properties = serialize_properties(&class, properties.clone());
					node.tags = Vec::new();
					node.keep_unknowns = None;

					instance.properties = properties;
				}
			}

			// It has to be done after updating properties as it may change the node path
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

				*meta.source.get_mut() = SourceKind::Project(new_name.clone(), path.clone(), Box::new(node), node_path);

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

	if let Some(instance) = tree.get_instance(id) {
		let filter = tree.get_meta(id).unwrap().context.syncback_filter();

		if filter.matches_name(&instance.name) || filter.matches_class(&instance.class) {
			filter_warn!(id);
			return Ok(());
		}
	} else {
		warn!("Attempted to remove instance that doesn't exist: {:?}", id);
		return Ok(());
	}

	let meta = tree.get_meta(id).unwrap().clone();

	fn remove_non_project_instances(id: Ref, meta: &Meta, tree: &mut Tree, vfs: &Vfs) -> Result<()> {
		let filter = meta.context.syncback_filter();

		for entry in meta.source.relevant() {
			match entry {
				SourceEntry::Project(_) => continue,
				_ => {
					let path = entry.path();

					if vfs.exists(path) {
						if filter.matches_path(path) {
							filter_warn!(id, path);
						} else {
							vfs.remove(path)?
						}
					}
				}
			}
		}

		// Transform parent instance source from folder to file
		// if it no longer has any children

		let parent = tree
			.get_instance(id)
			.and_then(|instance| tree.get_instance(instance.parent()))
			.expect("Instance has no parent or parent does not have associated meta");

		if parent.children().len() != 1 {
			return Ok(());
		}

		let meta = tree.get_meta_mut(parent.referent()).unwrap();

		if let SourceKind::Path(folder_path) = meta.source.get() {
			let name = folder_path.get_name();

			if let Some(file) = meta.source.get_file() {
				let file_path = meta
					.context
					.sync_rules()
					.iter()
					.find(|rule| rule.matches_child(file.path()))
					.and_then(|rule| rule.locate(folder_path, name, false));

				if let Some(new_path) = file_path {
					vfs.rename(file.path(), &new_path)?;
					let mut source = Source::file(&new_path);

					if let Some(data) = meta.source.get_data() {
						let data_path = meta
							.context
							.sync_rules_of_type(&Middleware::InstanceData, true)
							.iter()
							.find_map(|rule| rule.locate(folder_path, name, false));

						if let Some(new_path) = data_path {
							vfs.rename(data.path(), &new_path)?;
							source.add_data(&new_path);
						}
					}

					vfs.remove(folder_path)?;
					meta.set_source(source);
				}
			}
		}

		Ok(())
	}

	match meta.source.get() {
		SourceKind::Path(_) => remove_non_project_instances(id, &meta, tree, vfs)?,
		SourceKind::Project(name, path, node, node_path) => {
			let mut project = Project::load(path)?;
			let parent_node = project.find_node_by_path(&node_path.parent());

			parent_node.and_then(|node| node.tree.remove(name)).ok_or(anyhow!(
				"Failed to remove instance {:?} from project: {:?}",
				id,
				project
			))?;

			if node.path.is_some() {
				remove_non_project_instances(id, &meta, tree, vfs)?;
			}

			project.save(path)?;
		}
		SourceKind::None => panic!("Attempted to remove instance with no source: {:?}", id),
	}

	tree.remove_instance(id);

	Ok(())
}
