#![allow(clippy::unnecessary_to_owned)] // false positive detection

use multimap::MultiMap;
use rbx_dom_weak::{types::Ref, Instance, InstanceBuilder, WeakDom};
use std::{
	collections::HashMap,
	path::{Path, PathBuf},
};

use super::instance::Instance as ArgonInstance;
use crate::{argon_warn, project::Project, rbx_path::RbxPath};

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
				local_path: instance.path,
			},
		);

		dom_ref
	}

	pub fn remove(&mut self, rbx_path: &RbxPath) -> bool {
		if let Some(refs) = self.ref_map.remove(rbx_path) {
			self.inner.destroy(refs.dom_ref);

			let mut children = vec![];

			for (path, _) in self.ref_map.iter() {
				if path.starts_with(rbx_path) {
					children.push(path.clone());
				}
			}

			for path in children {
				self.ref_map.remove(&path);
			}

			return true;
		}

		false
	}

	pub fn append(&mut self, dom: &mut WeakDom, rbx_path: &RbxPath, path: &Path) -> MultiMap<RbxPath, &Instance> {
		fn insert_new_refs(
			children: &[Ref],
			dom: &WeakDom,
			ref_map: &mut HashMap<RbxPath, Refs>,
			rbx_path: &RbxPath,
			path: &Path,
		) {
			for child in children {
				let instance = dom.get_by_ref(*child).unwrap();
				let instance_path = rbx_path.join(&instance.name);

				ref_map.insert(
					instance_path,
					Refs {
						dom_ref: child.to_owned(),
						local_path: path.to_owned(),
					},
				);

				insert_new_refs(instance.children(), dom, ref_map, rbx_path, path);
			}
		}

		fn get_new_instances<'a>(
			instance: &'a Instance,
			new_instances: &mut MultiMap<RbxPath, &'a Instance>,
			rbx_path: &RbxPath,
			dom: &'a WeakDom,
		) {
			let rbx_path = rbx_path.join(&instance.name);

			new_instances.insert(rbx_path.clone(), instance);

			for child in instance.children() {
				let child = dom.get_by_ref(*child).unwrap();
				get_new_instances(child, new_instances, &rbx_path, dom);
			}
		}

		let mut new_instances = MultiMap::new();

		let mut parent = rbx_path.clone();
		parent.pop();
		let parent = self.get_ref(&parent).unwrap();

		if dom.root().children().is_empty() {
			argon_warn!("Tried to append empty DOM");
		} else if dom.root().children().len() == 1 {
			let child_ref = *dom.root().children().first().unwrap();

			dom.transfer(child_ref, &mut self.inner, parent);

			let child = self.get_by_ref_mut(child_ref).unwrap();
			child.name = dom.root().name.clone();

			self.ref_map.insert(
				rbx_path.to_owned(),
				Refs {
					dom_ref: child_ref,
					local_path: path.to_owned(),
				},
			);

			insert_new_refs(
				&self.get_by_ref(child_ref).unwrap().children().to_vec(),
				&self.inner,
				&mut self.ref_map,
				rbx_path,
				path,
			);

			get_new_instances(
				self.get_by_ref(child_ref).unwrap(),
				&mut new_instances,
				rbx_path,
				&self.inner,
			);
		} else {
			let instance = InstanceBuilder::new("Folder").with_name(dom.root().name.clone());
			let parent = self.inner.insert(parent, instance);

			for child in dom.root().children().to_vec() {
				dom.transfer(child, &mut self.inner, parent);
			}

			self.ref_map.insert(
				rbx_path.to_owned(),
				Refs {
					dom_ref: parent,
					local_path: path.to_owned(),
				},
			);

			insert_new_refs(
				&self.get_by_ref(parent).unwrap().children().to_vec(),
				&self.inner,
				&mut self.ref_map,
				rbx_path,
				path,
			);

			get_new_instances(
				self.get_by_ref(parent).unwrap(),
				&mut new_instances,
				rbx_path,
				&self.inner,
			);
		}

		new_instances
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

	pub fn get_rbx_path(&self, dom_ref: Ref) -> Option<&RbxPath> {
		self.ref_map
			.iter()
			.find_map(|(rbx_path, refs)| if refs.dom_ref == dom_ref { Some(rbx_path) } else { None })
	}

	pub fn root(&self) -> &Instance {
		self.inner.root()
	}

	pub fn root_ref(&self) -> Ref {
		self.inner.root_ref()
	}

	pub fn place_root_refs(&self) -> &[Ref] {
		self.inner.root().children()
	}

	pub fn inner(&self) -> &WeakDom {
		&self.inner
	}
}
