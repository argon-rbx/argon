#![allow(clippy::unnecessary_to_owned)]
#![allow(dead_code)]

use colored::Colorize;
use log::{error, warn};
use multimap::MultiMap;
use rbx_dom_weak::{
	types::{Ref, Variant},
	Instance, InstanceBuilder, WeakDom,
};
use std::{
	collections::HashMap,
	fmt::Debug,
	mem,
	path::{Path, PathBuf},
};

use super::instance::Instance as ArgonInstance;
use crate::{project::Project, rbx_path::RbxPath};

#[derive(Debug)]
struct Refs {
	dom_ref: Ref,
	local_path: PathBuf,
}

#[derive(Debug)]
pub struct Tree {
	inner: WeakDom,
	ref_map: HashMap<RbxPath, Refs>,
}

impl Tree {
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

	pub fn init(&mut self, rbx_path: &RbxPath, properties: HashMap<String, Variant>) {
		if self.contains(rbx_path) {
			self.get_mut(rbx_path).unwrap().properties = properties;
		} else if let Some(Variant::String(class)) = properties.get("ClassName") {
			let name = rbx_path.last().unwrap();
			let parent = self.get_ref(&rbx_path.parent().unwrap()).unwrap();

			let builder = InstanceBuilder::new(class).with_name(name).with_properties(properties);

			self.inner.insert(parent, builder);
		} else {
			error!(
				"Failed to create instance {}: ClassName does not exist or is not a string!",
				rbx_path.to_string().bold()
			)
		}
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
		fn walk<'a>(
			children: &[Ref],
			dom: &'a WeakDom,
			ref_map: &mut HashMap<RbxPath, Refs>,
			new_instances: &mut MultiMap<RbxPath, &'a Instance>,
			rbx_path: &RbxPath,
			path: &Path,
		) {
			for child in children {
				let instance = dom.get_by_ref(*child).unwrap();
				let instance_path = rbx_path.join(&instance.name);

				ref_map.insert(
					instance_path.clone(),
					Refs {
						dom_ref: child.to_owned(),
						local_path: path.to_owned(),
					},
				);

				new_instances.insert(instance_path, instance);

				walk(instance.children(), dom, ref_map, new_instances, rbx_path, path);
			}
		}

		let mut new_instances = MultiMap::new();

		let parent = rbx_path.parent().unwrap();
		let parent = self.get_ref(&parent).unwrap();

		if dom.root().children().is_empty() {
			warn!("Tried to append empty DOM");
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

			walk(
				&self.get_by_ref(parent).unwrap().children().to_vec(),
				&self.inner,
				&mut self.ref_map,
				&mut new_instances,
				rbx_path,
				path,
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

			walk(
				&self.get_by_ref(parent).unwrap().children().to_vec(),
				&self.inner,
				&mut self.ref_map,
				&mut new_instances,
				rbx_path,
				path,
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

	pub fn get_local_path(&self, rbx_path: &RbxPath) -> Option<&PathBuf> {
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

	pub fn reload(&mut self, project: &Project) {
		let new = Self::new(project);

		drop(mem::replace(self, new));
	}
}
