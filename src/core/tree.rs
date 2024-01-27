use multimap::MultiMap;
use rbx_dom_weak::{types::Ref, InstanceBuilder, WeakDom};
use std::{
	collections::HashMap,
	path::{Path, PathBuf},
};

use super::{meta::Meta, snapshot::Snapshot};

#[derive(Debug)]
pub struct Tree {
	dom: WeakDom,
	ids_to_meta: HashMap<Ref, Meta>,
	path_to_ids: MultiMap<PathBuf, Ref>,
}

impl Tree {
	pub fn new(snapshot: Snapshot) -> Self {
		let builder = InstanceBuilder::new(snapshot.class)
			.with_name(snapshot.name)
			.with_properties(snapshot.properties);

		let mut tree = Self {
			dom: WeakDom::new(builder),
			ids_to_meta: HashMap::new(),
			path_to_ids: MultiMap::new(),
		};

		let root_ref = tree.dom.root_ref();

		tree.ids_to_meta.insert(root_ref, snapshot.meta.unwrap());
		tree.path_to_ids.insert(snapshot.path.unwrap(), root_ref);

		for child in snapshot.children {
			tree.insert(child, root_ref);
		}

		tree
	}

	pub fn insert(&mut self, snapshot: Snapshot, parent: Ref) -> Ref {
		let builder = InstanceBuilder::new(snapshot.class)
			.with_name(snapshot.name)
			.with_properties(snapshot.properties);

		let referent = self.dom.insert(parent, builder);

		if let Some(path) = snapshot.path {
			self.path_to_ids.insert(path, referent);
		}

		if let Some(meta) = snapshot.meta {
			self.ids_to_meta.insert(referent, meta);
		}

		for child in snapshot.children {
			self.insert(child, referent);
		}

		referent
	}

	pub fn remove(&mut self, id: Ref) {
		self.dom.destroy(id);
		self.ids_to_meta.remove(&id);

		let mut removed = vec![];

		for (path, ids) in self.path_to_ids.iter_all_mut() {
			ids.retain(|&referent| referent != id);

			if ids.is_empty() {
				removed.push(path.to_owned());
			}
		}

		for path in removed {
			self.path_to_ids.remove(&path);
		}
	}

	pub fn inner(&self) -> &WeakDom {
		&self.dom
	}

	pub fn root_ref(&self) -> Ref {
		self.dom.root_ref()
	}

	pub fn place_root_refs(&self) -> &[Ref] {
		self.dom.root().children()
	}

	pub fn get_ids(&self, path: &Path) -> Option<&Vec<Ref>> {
		self.path_to_ids.get_vec(path)
	}

	pub fn get_meta(&self, id: Ref) -> &Meta {
		if let Some(meta) = self.ids_to_meta.get(&id) {
			return meta;
		}

		self.get_meta(self.dom.get_by_ref(id).unwrap().parent())
	}
}
