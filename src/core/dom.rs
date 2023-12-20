use rbx_dom_weak::{
	types::{Ref, Variant},
	Instance, InstanceBuilder, WeakDom,
};
use std::{
	collections::HashMap,
	path::{Path, PathBuf},
};

use crate::{project::Project, rbx_path::RbxPath};

#[derive(Debug)]
struct Refs {
	dom_ref: Ref,
	local_path: PathBuf,
}

#[derive(Debug)]
pub struct Dom {
	inner: WeakDom,
	ref_map: HashMap<RbxPath, Refs>,
}

impl Dom {
	pub fn new(project: &Project) -> Self {
		let builder = InstanceBuilder::new(&project.root_class).with_name(&project.name);
		let dom = WeakDom::new(builder);

		let mut ref_map = HashMap::new();

		let root_path = if let Some(path) = &project.root_dir {
			path.to_owned()
		} else {
			project.project_path.clone()
		};

		ref_map.insert(
			RbxPath::from(&project.name),
			Refs {
				dom_ref: dom.root_ref(),
				local_path: root_path,
			},
		);

		Self { inner: dom, ref_map }
	}

	pub fn insert(&mut self, parent: Ref, name: &str, path: &Path, rbx_path: RbxPath) -> Ref {
		let builder = InstanceBuilder::new("Folder").with_name(name);
		let dom_ref = self.inner.insert(parent, builder);

		self.ref_map.insert(
			rbx_path,
			Refs {
				dom_ref,
				local_path: path.to_path_buf(),
			},
		);

		dom_ref
	}

	pub fn get(&self, dom_ref: Ref) -> Option<&Instance> {
		self.inner.get_by_ref(dom_ref)
	}

	pub fn set_class(&mut self, dom_ref: Ref, class: &str) {
		let instance = self.inner.get_by_ref_mut(dom_ref).unwrap();
		instance.class = class.to_owned();
	}

	pub fn set_properties(&mut self, dom_ref: Ref, properties: HashMap<String, Variant>) {
		let instance = self.inner.get_by_ref_mut(dom_ref).unwrap();
		instance.properties = properties;
	}

	pub fn contains(&self, rbx_path: &RbxPath) -> bool {
		self.ref_map.contains_key(rbx_path)
	}

	pub fn get_ref(&self, rbx_path: &RbxPath) -> Option<Ref> {
		self.ref_map.get(rbx_path).map(|refs| refs.dom_ref)
	}

	pub fn get_local_paths(&self, rbx_path: &RbxPath) -> Option<&PathBuf> {
		self.ref_map.get(rbx_path).map(|refs| &refs.local_path)
	}

	pub fn root(&self) -> Ref {
		self.inner.root_ref()
	}

	pub fn place_roots(&self) -> &[Ref] {
		self.inner.get_by_ref(self.root()).unwrap().children()
	}

	pub fn inner(&self) -> &WeakDom {
		&self.inner
	}
}
