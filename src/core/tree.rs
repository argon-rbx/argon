use multimap::MultiMap;
use rbx_dom_weak::{types::Ref, Instance, InstanceBuilder, WeakDom};
use std::{
	collections::{HashMap, VecDeque},
	path::{Path, PathBuf},
};

use super::{meta::Meta, snapshot::Snapshot};
use crate::util::PathExt;

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
		let meta = snapshot.meta.unwrap();

		tree.ids_to_meta.insert(root_ref, meta.clone());
		tree.path_to_ids.insert(snapshot.path.unwrap(), root_ref);

		// If the root `Ref` has `$path` assigned we need
		// to insert it as well.
		for path in meta.child_sources {
			let path = path.get_parent();

			if !tree.path_to_ids.contains_key(path) {
				tree.path_to_ids.insert(path.to_owned(), root_ref);
			}
		}

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
			if !meta.is_empty() {
				for path in &meta.child_sources {
					let path = path.get_parent();

					// We need to insert `$path` of the root `Ref` if there is one.
					// Only child projects require this.
					if !self.path_to_ids.contains_key(path)
						|| !self.path_to_ids.get_vec(path).unwrap().contains(&referent)
					{
						self.path_to_ids.insert(path.to_owned(), referent);
					}
				}

				self.ids_to_meta.insert(referent, meta);
			}
		}

		for child in snapshot.children {
			self.insert(child, referent);
		}

		referent
	}

	pub fn remove(&mut self, id: Ref) {
		self.dom.destroy(id);
		self.ids_to_meta.remove(&id);

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
			self.path_to_ids.retain(|path, referent| {
				let matches = path.starts_with(removed_path) && path != removed_path;

				if matches {
					self.ids_to_meta.retain(|id, _| id != referent);
				}

				!matches
			})
		}
	}

	pub fn insert_meta(&mut self, id: Ref, meta: Meta) -> Option<Meta> {
		self.ids_to_meta.insert(id, meta)
	}

	pub fn get_instance(&self, id: Ref) -> Option<&Instance> {
		self.dom.get_by_ref(id)
	}

	pub fn get_instance_mut(&mut self, id: Ref) -> Option<&mut Instance> {
		self.dom.get_by_ref_mut(id)
	}

	pub fn get_ids(&self, path: &Path) -> Option<&Vec<Ref>> {
		self.path_to_ids.get_vec(path)
	}

	pub fn get_meta(&self, id: Ref) -> Option<&Meta> {
		self.ids_to_meta.get(&id)
	}

	pub fn get_meta_mut(&mut self, id: Ref) -> Option<&mut Meta> {
		self.ids_to_meta.get_mut(&id)
	}

	/// Get all meta associated with the given `Ref` in order from root to leaf
	pub fn get_meta_all(&self, id: Ref) -> VecDeque<&Meta> {
		let mut metas = VecDeque::new();
		let mut id = id;

		loop {
			if let Some(meta) = self.ids_to_meta.get(&id) {
				metas.push_front(meta);
			}

			if id == self.dom.root_ref() {
				break metas;
			}

			id = self.dom.get_by_ref(id).unwrap().parent();
		}
	}

	pub fn get_path(&self, id: Ref) -> Option<&PathBuf> {
		let mut paths = vec![];

		for (path, ids) in &self.path_to_ids {
			if ids.contains(&id) {
				paths.push(path);
			}
		}

		// Only scenario where there are multiple paths for a single `id`
		// is when said `id` is the root `Ref` of the `WeakDom` and it's
		// not a `DataModel` instance - has `$path` assigned.
		if paths.is_empty() {
			None
		} else {
			let path = paths
				.iter()
				.fold(paths[0], |acc, path| if path.len() < acc.len() { path } else { acc });

			Some(path)
		}
	}

	pub fn inner(&self) -> &WeakDom {
		&self.dom
	}

	pub fn meta_map(&self) -> &HashMap<Ref, Meta> {
		&self.ids_to_meta
	}

	pub fn id_map(&self) -> &MultiMap<PathBuf, Ref> {
		&self.path_to_ids
	}

	pub fn root_ref(&self) -> Ref {
		self.dom.root_ref()
	}

	pub fn place_root_refs(&self) -> &[Ref] {
		self.dom.root().children()
	}
}
