use rbx_dom_weak::{types::Ref, Instance, InstanceBuilder, WeakDom};
use std::{collections::HashMap, path::PathBuf};

use super::instance::Instance as ArgonInstance;
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

	pub fn insert(&mut self, instance: ArgonInstance, parent: &RbxPath) -> Ref {
		let dom_ref = if self.contains(&instance.rbx_path) {
			let old_instance = self.get_mut(&instance.rbx_path).unwrap();

			old_instance.name = instance.name;
			old_instance.class = instance.class;
			old_instance.properties = instance.properties;

			old_instance.referent()
		} else {
			let builder = InstanceBuilder::new(instance.class)
				.with_name(instance.name)
				.with_properties(instance.properties);

			let parent_ref = self.get_ref(parent).unwrap();

			self.inner.insert(parent_ref, builder)
		};

		self.ref_map.insert(
			instance.rbx_path,
			Refs {
				dom_ref,
				local_path: instance.path.to_path_buf(),
			},
		);

		dom_ref
	}

	pub fn get(&self, rbx_path: &RbxPath) -> Option<&Instance> {
		self.ref_map
			.get(rbx_path)
			.and_then(|refs| self.inner.get_by_ref(refs.dom_ref))
	}

	pub fn get_mut(&mut self, rbx_path: &RbxPath) -> Option<&mut Instance> {
		self.ref_map
			.get(rbx_path)
			.and_then(|refs| self.inner.get_by_ref_mut(refs.dom_ref))
	}

	pub fn get_by_ref(&self, dom_ref: Ref) -> Option<&Instance> {
		self.inner.get_by_ref(dom_ref)
	}

	pub fn get_by_ref_mut(&mut self, dom_ref: Ref) -> Option<&mut Instance> {
		self.inner.get_by_ref_mut(dom_ref)
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
		self.inner.root().children()
	}

	pub fn inner(&self) -> &WeakDom {
		&self.inner
	}
}
