use log::{error, warn};
use multimap::MultiMap;
use rbx_dom_weak::{
	types::{Ref, Variant},
	Instance, InstanceBuilder, Ustr, WeakDom,
};
use std::{
	collections::HashMap,
	path::{Component, Path, PathBuf},
};

use super::{meta::Meta, snapshot::Snapshot};

#[derive(Debug)]
pub struct Tree {
	dom: WeakDom,
	path_to_ids: MultiMap<PathBuf, Ref>,
	id_to_meta: HashMap<Ref, Meta>,
}

impl Tree {
	pub fn new(snapshot: Snapshot) -> Self {
		let builder = InstanceBuilder::new(snapshot.class)
			.with_name(snapshot.name)
			.with_properties(snapshot.properties);

		let mut tree = Self {
			dom: WeakDom::new(builder),
			id_to_meta: HashMap::new(),
			path_to_ids: MultiMap::new(),
		};

		let root_ref = tree.dom.root_ref();

		tree.insert_meta(root_ref, snapshot.meta);

		for child in snapshot.children {
			tree.insert_instance_recursive(child, root_ref);
		}

		tree
	}

	pub fn insert_instance(&mut self, snapshot: Snapshot, parent: Ref) -> Ref {
		let builder = InstanceBuilder::new(snapshot.class)
			.with_name(snapshot.meta.original_name.as_ref().unwrap_or(&snapshot.name))
			.with_properties(snapshot.properties);

		let id = self.dom.insert(parent, builder);

		self.insert_meta(id, snapshot.meta);

		id
	}

	pub fn insert_instance_recursive(&mut self, mut snapshot: Snapshot, parent: Ref) -> Ref {
		let mut ref_map = HashMap::new();

		let id = self.insert_instance_recursive_inner(&mut snapshot, parent, &mut ref_map);

		if !ref_map.is_empty() {
			self.remap_inserted_refs(&snapshot, &ref_map);
		}

		id
	}

	fn insert_instance_recursive_inner(
		&mut self,
		snapshot: &mut Snapshot,
		parent: Ref,
		ref_map: &mut HashMap<Ref, Ref>,
	) -> Ref {
		let mut builder = InstanceBuilder::new(snapshot.class)
			.with_name(snapshot.meta.original_name.as_ref().unwrap_or(&snapshot.name))
			.with_properties(snapshot.properties.clone());

		if snapshot.id != Ref::none() {
			builder = builder.with_referent(snapshot.id);
		}

		let id = self.dom.insert(parent, builder);

		self.insert_meta(id, snapshot.meta.clone());
		snapshot.set_id(id);

		if snapshot.ref_id.is_some() {
			ref_map.insert(snapshot.ref_id, id);
		}

		for child in snapshot.children.iter_mut() {
			self.insert_instance_recursive_inner(child, id, ref_map);
		}

		id
	}

	fn remap_inserted_refs(&mut self, snapshot: &Snapshot, ref_map: &HashMap<Ref, Ref>) {
		if let Some(instance) = self.dom.get_by_ref_mut(snapshot.id) {
			for value in instance.properties.values_mut() {
				if let Variant::Ref(reference) = value {
					if let Some(&mapped) = ref_map.get(reference) {
						*value = Variant::Ref(mapped);
					}
				}
			}
		}

		for child in &snapshot.children {
			self.remap_inserted_refs(child, ref_map);
		}
	}

	pub fn insert_instance_with_ref(&mut self, snapshot: Snapshot, parent: Ref) {
		let builder = InstanceBuilder::new(snapshot.class)
			.with_name(snapshot.meta.original_name.as_ref().unwrap_or(&snapshot.name))
			.with_referent(snapshot.id)
			.with_properties(snapshot.properties);

		let id = self.dom.insert(parent, builder);

		self.insert_meta(id, snapshot.meta);
	}

	pub fn remove_instance(&mut self, id: Ref) {
		if id == self.root_ref() {
			error!("Attempted to remove root instance: {id:?}");
			return;
		}

		let mut to_remove = vec![id];

		fn walk(id: Ref, dom: &WeakDom, to_remove: &mut Vec<Ref>) {
			let instance = dom.get_by_ref(id).unwrap();

			for child in instance.children() {
				to_remove.push(*child);
				walk(*child, dom, to_remove);
			}
		}

		walk(id, &self.dom, &mut to_remove);

		for id in to_remove {
			self.remove_meta(id);
		}

		self.dom.destroy(id);
	}

	pub fn get_instance(&self, id: Ref) -> Option<&Instance> {
		self.dom.get_by_ref(id)
	}

	pub fn get_instance_mut(&mut self, id: Ref) -> Option<&mut Instance> {
		self.dom.get_by_ref_mut(id)
	}

	pub fn insert_meta(&mut self, id: Ref, meta: Meta) -> Option<Meta> {
		for path in meta.source.paths() {
			self.path_to_ids.insert(path.to_owned(), id);
		}

		self.id_to_meta.insert(id, meta)
	}

	pub fn update_meta(&mut self, id: Ref, meta: Meta) -> Option<Meta> {
		let old_meta = self.id_to_meta.remove(&id);

		if let Some(old_meta) = &old_meta {
			let removed: Vec<&Path> = old_meta
				.source
				.paths()
				.into_iter()
				.filter(|&path| !meta.source.paths().contains(&path))
				.collect();

			let added: Vec<&Path> = meta
				.source
				.paths()
				.into_iter()
				.filter(|&path| !old_meta.source.paths().contains(&path))
				.collect();

			for path in removed {
				self.path_to_ids.remove(path);
			}

			for path in added {
				self.path_to_ids.insert(path.to_owned(), id);
			}
		}

		self.id_to_meta.insert(id, meta);

		old_meta
	}

	pub fn remove_meta(&mut self, id: Ref) -> Option<Meta> {
		let meta = self.id_to_meta.remove(&id);

		if let Some(meta) = &meta {
			for path in meta.source.paths() {
				self.path_to_ids.remove(path);
			}
		}

		meta
	}

	pub fn get_meta(&self, id: Ref) -> Option<&Meta> {
		self.id_to_meta.get(&id)
	}

	pub fn get_meta_mut(&mut self, id: Ref) -> Option<&mut Meta> {
		self.id_to_meta.get_mut(&id)
	}

	pub fn get_ids(&self, path: &Path) -> Option<&Vec<Ref>> {
		self.path_to_ids.get_vec(path)
	}

	pub fn resolve_refs(&mut self) {
		let pending: Vec<(Ref, Ustr, String)> = self
			.id_to_meta
			.iter()
			.flat_map(|(&id, meta)| {
				meta.pending_refs
					.iter()
					.map(move |(&property, path)| (id, property, path.clone()))
			})
			.collect();

		for (id, property, relative_path) in pending {
			let Some(meta) = self.id_to_meta.get(&id) else {
				continue;
			};

			let Some(anchor) = meta.source.anchor_dir() else {
				warn!("Failed to resolve Ref property {property}: instance has no anchor directory");
				continue;
			};

			let target_path = normalize_path(&anchor.join(&relative_path));

			let target_id = self
				.path_to_ids
				.get_vec(&target_path)
				.and_then(|ids| ids.first())
				.copied();

			match target_id {
				Some(target_id) => {
					if let Some(instance) = self.dom.get_by_ref_mut(id) {
						instance.properties.insert(property, Variant::Ref(target_id));
					}
				}
				None => {
					warn!(
						"Failed to resolve Ref property {property}: target '{relative_path}' not found relative to {}",
						anchor.display()
					);
				}
			}
		}
	}

	pub fn exists(&self, id: Ref) -> bool {
		self.dom.get_by_ref(id).is_some()
	}

	pub fn inner(&self) -> &WeakDom {
		&self.dom
	}

	pub fn meta_map(&self) -> &HashMap<Ref, Meta> {
		&self.id_to_meta
	}

	pub fn id_map(&self) -> &MultiMap<PathBuf, Ref> {
		&self.path_to_ids
	}

	pub fn root(&self) -> &Instance {
		self.dom.root()
	}

	pub fn root_ref(&self) -> Ref {
		self.dom.root_ref()
	}

	pub fn place_root_refs(&self) -> &[Ref] {
		self.dom.root().children()
	}
}

fn normalize_path(path: &Path) -> PathBuf {
	let mut components = Vec::new();

	for component in path.components() {
		match component {
			Component::CurDir => {}
			Component::ParentDir => {
				if matches!(components.last(), Some(Component::Normal(_))) {
					components.pop();
				} else {
					components.push(component);
				}
			}
			other => components.push(other),
		}
	}

	components.iter().collect()
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::core::meta::Source;

	#[test]
	fn resolve_refs_resolves_pending_ref() {
		let root_path = Path::new("/project/src/Model");
		let hitbox_path = Path::new("/project/src/Model/Hitbox");

		let mut root_meta = Meta::new();
		root_meta.set_source(Source::directory(root_path));
		root_meta
			.pending_refs
			.insert(Ustr::from("PrimaryPart"), "Hitbox".to_owned());

		let mut hitbox_meta = Meta::new();
		hitbox_meta.set_source(Source::directory(hitbox_path));

		let snapshot = Snapshot::new()
			.with_class("Model")
			.with_name("Model")
			.with_meta(root_meta)
			.with_children(vec![Snapshot::new()
				.with_class("Folder")
				.with_name("Hitbox")
				.with_meta(hitbox_meta)]);

		let mut tree = Tree::new(snapshot);
		tree.resolve_refs();

		let hitbox_id = tree.root().children()[0];

		assert_eq!(
			tree.root().properties.get(&Ustr::from("PrimaryPart")),
			Some(&Variant::Ref(hitbox_id))
		);
	}
}
