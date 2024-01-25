use multimap::MultiMap;
use rbx_dom_weak::{types::Ref, InstanceBuilder, WeakDom};
use std::{
	collections::HashMap,
	path::{Path, PathBuf},
};

use super::meta::Meta;
use crate::project::Project;

pub struct Tree {
	dom: WeakDom,
	ids_to_meta: HashMap<Ref, Meta>,
	path_to_ids: MultiMap<PathBuf, Ref>,
}

impl Tree {
	pub fn new(project: &Project) -> Self {
		let builder = InstanceBuilder::new(project.root_class.clone()).with_name(project.name.clone());

		Self {
			dom: WeakDom::new(builder),
			ids_to_meta: HashMap::new(),
			path_to_ids: MultiMap::new(),
		}
	}

	pub fn get_ids(&self, path: &Path) -> Option<&Vec<Ref>> {
		self.path_to_ids.get_vec(path)
	}

	pub fn get_meta(&self, id: Ref) -> &Meta {
		if let Some(meta) = self.ids_to_meta.get(&id) {
			return meta;
		}

		self.get_meta(self.dom.get_by_ref(id).unwrap().referent())
	}
}
