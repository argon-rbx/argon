use rbx_dom_weak::types::{Ref, Variant};
use std::{
	collections::HashMap,
	fmt::Debug,
	path::{Path, PathBuf},
};

use super::meta::Meta;

pub struct Snapshot {
	pub id: Option<Ref>,
	pub meta: Option<Meta>,
	pub path: Option<PathBuf>,

	pub name: String,
	pub class: String,
	pub properties: HashMap<String, Variant>,
	pub children: Vec<Snapshot>,
}

impl Snapshot {
	pub fn new(name: &str) -> Self {
		Self {
			id: None,
			meta: None,
			path: None,
			name: name.to_string(),
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
		self.path = Some(path.into());
		self
	}

	pub fn with_name(mut self, name: &str) -> Self {
		self.name = name.into();
		self
	}

	pub fn with_class(mut self, class: &str) -> Self {
		self.class = class.into();
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
}

impl Debug for Snapshot {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		let mut debug = f.debug_struct("Snapshot");

		debug.field("name", &self.name);
		debug.field("class", &self.class);

		if let Some(path) = &self.path {
			debug.field("path", &path);
		}

		if let Some(id) = self.id {
			debug.field("id", &id);
		}

		if let Some(meta) = &self.meta {
			debug.field("meta", &meta);
		}

		if !self.properties.is_empty() {
			let mut properties = self.properties.clone();

			if let Some(property) = properties.get_mut("Source") {
				if let Variant::String(source) = property {
					*property = Variant::String(format!("Truncated... ({} lines)", source.lines().count()));
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
