use rbx_dom_weak::{
	types::{Ref, Variant},
	Instance, WeakDom,
};
use std::{
	collections::HashMap,
	fmt::{self, Debug, Formatter},
	path::{Path, PathBuf},
};

use super::meta::Meta;
use crate::middleware::FileType;

#[derive(Clone)]
pub struct Snapshot {
	// For middleware & change processing
	pub id: Option<Ref>,
	pub meta: Option<Meta>,
	pub paths: Vec<PathBuf>,
	pub file_type: Option<FileType>,

	// Roblox related
	pub name: String,
	pub class: String,
	pub properties: HashMap<String, Variant>,
	pub children: Vec<Snapshot>,
}

impl Snapshot {
	// Creating new snapshot

	pub fn new() -> Self {
		Self {
			id: None,
			meta: None,
			paths: Vec::new(),
			file_type: None,
			name: String::from(""),
			class: String::from("Folder"),
			properties: HashMap::new(),
			children: Vec::new(),
		}
	}

	pub fn with_id(mut self, id: Ref) -> Self {
		self.id = Some(id);
		self
	}

	pub fn with_meta(mut self, meta: Meta) -> Self {
		self.meta = Some(meta);
		self
	}

	pub fn with_path(mut self, path: &Path) -> Self {
		self.paths.push(path.to_owned());
		self
	}

	pub fn with_paths(mut self, paths: Vec<PathBuf>) -> Self {
		// Projects should have their original paths kept
		if self.is_project() {
			self.extend_paths(paths);
			return self;
		}

		self.paths = paths;
		self
	}

	pub fn with_file_type(mut self, file_type: FileType) -> Self {
		self.file_type = Some(file_type);
		self
	}

	pub fn with_name(mut self, name: &str) -> Self {
		// Projects should not have their name overwritten
		if self.is_project() {
			return self;
		}

		self.name = name.to_owned();
		self
	}

	pub fn with_class(mut self, class: &str) -> Self {
		self.class = class.to_owned();
		self
	}

	pub fn with_properties(mut self, properties: HashMap<String, Variant>) -> Self {
		self.properties = properties;
		self
	}

	pub fn with_children(mut self, children: Vec<Snapshot>) -> Self {
		self.children = children;
		self
	}

	pub fn with_data(mut self, data: Self) -> Self {
		if self.class == "Folder" {
			self.class = data.class;
		}

		self.extend_properties(data.properties);

		self
	}

	// Overwriting snapshot fields

	pub fn set_id(&mut self, id: Ref) {
		self.id = Some(id);
	}

	pub fn set_meta(&mut self, meta: Meta) {
		self.meta = Some(meta);
	}

	pub fn set_paths(&mut self, paths: Vec<PathBuf>) {
		self.paths = paths;
	}

	pub fn set_file_type(&mut self, file_type: FileType) {
		self.file_type = Some(file_type);
	}

	pub fn set_name(&mut self, name: &str) {
		self.name = name.to_owned();
	}

	pub fn set_class(&mut self, class: &str) {
		self.class = class.to_owned();
	}

	pub fn set_properties(&mut self, properties: HashMap<String, Variant>) {
		self.properties = properties;
	}

	pub fn set_children(&mut self, children: Vec<Snapshot>) {
		self.children = children;
	}

	// Adding to snapshot fields

	pub fn add_path(&mut self, path: &Path) {
		self.paths.push(path.to_owned());
	}

	pub fn add_property(&mut self, name: &str, value: Variant) {
		self.properties.insert(name.to_owned(), value);
	}

	pub fn add_child(&mut self, child: Snapshot) {
		self.children.push(child);
	}

	// Joining snapshot fields

	pub fn extend_paths(&mut self, paths: Vec<PathBuf>) {
		self.paths.extend(paths);
	}

	pub fn extend_properties(&mut self, properties: HashMap<String, Variant>) {
		self.properties.extend(properties);
	}

	pub fn extend_children(&mut self, children: Vec<Snapshot>) {
		self.children.extend(children);
	}

	pub fn extend_meta(&mut self, meta: Meta) {
		if let Some(snapshot_meta) = &mut self.meta {
			snapshot_meta.extend(meta);
		} else {
			self.meta = Some(meta);
		}
	}

	// Misc

	pub fn apply_project_data(mut self, meta: &Meta, path: &Path) -> Self {
		if let Some(project_data) = &meta.project_data {
			if path != project_data.affects {
				return self;
			// Check if project containing this data still exists
			} else if !project_data.source.exists() {
				let mut meta = meta.clone();
				meta.project_data = None;

				self.set_meta(meta);
				return self;
			}

			self.set_name(&project_data.name);

			if let Some(class) = &project_data.class {
				self.set_class(class);
			}

			if let Some(properties) = &project_data.properties {
				self.extend_properties(properties.clone());
			}
		}

		self
	}

	// Based on Rojo's InstanceSnapshot::from_tree (https://github.com/rojo-rbx/rojo/blob/master/src/snapshot/instance_snapshot.rs#L105)
	pub fn from_dom(dom: WeakDom, id: Ref) -> Self {
		let (_, mut raw_dom) = dom.into_raw();

		fn walk(id: Ref, raw_dom: &mut HashMap<Ref, Instance>) -> Snapshot {
			let instance = raw_dom
				.remove(&id)
				.expect("Provided ID does not exist in the current DOM");

			let children = instance
				.children()
				.iter()
				.map(|&child_id| walk(child_id, raw_dom))
				.collect();

			Snapshot::new()
				.with_name(&instance.name)
				.with_class(&instance.class)
				.with_properties(instance.properties)
				.with_children(children)
		}

		walk(id, &mut raw_dom)
	}

	fn is_project(&self) -> bool {
		if let Some(file_type) = &self.file_type {
			if *file_type == FileType::Project {
				return true;
			}
		}

		false
	}
}

impl Debug for Snapshot {
	fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
		let mut debug = f.debug_struct("Snapshot");

		debug.field("name", &self.name);
		debug.field("class", &self.class);

		if !self.paths.is_empty() {
			debug.field("paths", &self.paths);
		}

		if let Some(id) = &self.id {
			debug.field("id", id);
		}

		if let Some(meta) = &self.meta {
			debug.field("meta", meta);
		}

		if !self.properties.is_empty() {
			let mut properties = self.properties.clone();

			if let Some(property) = properties.get_mut("Source") {
				if let Variant::String(source) = property {
					let lines = source.lines().count();

					if lines > 1 {
						*property = Variant::String(format!("Truncated... ({} lines)", lines));
					}
				}
			}

			debug.field("properties", &properties);
		}

		if !self.children.is_empty() {
			debug.field("children", &self.children);
		}

		debug.finish()
	}
}
