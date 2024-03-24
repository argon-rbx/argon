use multimap::MultiMap;
use rbx_dom_weak::{types::Ref, Instance, InstanceBuilder, WeakDom};
use std::{
	collections::HashMap,
	path::{Path, PathBuf},
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

		if let Some(path) = snapshot.meta.source.path() {
			tree.path_to_ids.insert(path.to_owned(), root_ref);
		}

		tree.id_to_meta.insert(root_ref, snapshot.meta);

		for child in snapshot.children {
			tree.insert_instance(child, root_ref);
		}

		tree
	}

	pub fn insert_instance(&mut self, snapshot: Snapshot, parent: Ref) -> Ref {
		let builder = InstanceBuilder::new(snapshot.class)
			.with_name(snapshot.name)
			.with_properties(snapshot.properties);

		let referent = self.dom.insert(parent, builder);

		if let Some(path) = snapshot.meta.source.path() {
			self.path_to_ids.insert(path.to_owned(), referent);
		}

		self.id_to_meta.insert(referent, snapshot.meta);

		for child in snapshot.children {
			self.insert_instance(child, referent);
		}

		referent
	}

	pub fn insert_instance_non_recursive(&mut self, snapshot: Snapshot, parent: Ref) -> Ref {
		let builder = InstanceBuilder::new(snapshot.class)
			.with_name(snapshot.name)
			.with_properties(snapshot.properties);

		let referent = self.dom.insert(parent, builder);

		if let Some(path) = snapshot.meta.source.path() {
			self.path_to_ids.insert(path.to_owned(), referent);
		}

		self.id_to_meta.insert(referent, snapshot.meta);

		referent
	}

	pub fn remove_instance(&mut self, id: Ref) {
		self.dom.destroy(id);
		self.id_to_meta.remove(&id);

		let mut removed_paths = vec![];

		self.path_to_ids.retain(|path, &referent| {
			let matches = referent == id;

			if matches {
				removed_paths.push(path.to_owned());
			}

			!matches
		});

		// Remove all descendant references
		for removed_path in &removed_paths {
			self.path_to_ids.retain(|path, id| {
				let matches = path.starts_with(removed_path) && path != removed_path;

				if matches {
					self.id_to_meta.remove(id);
				}

				!matches
			})
		}
	}

	pub fn get_instance(&self, id: Ref) -> Option<&Instance> {
		self.dom.get_by_ref(id)
	}

	pub fn get_instance_mut(&mut self, id: Ref) -> Option<&mut Instance> {
		self.dom.get_by_ref_mut(id)
	}

	pub fn insert_meta(&mut self, id: Ref, meta: Meta) -> Option<Meta> {
		self.id_to_meta.insert(id, meta)
	}

	pub fn remove_meta(&mut self, id: Ref) -> Option<Meta> {
		self.id_to_meta.remove(&id)
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
