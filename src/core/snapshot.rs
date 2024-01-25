use rbx_dom_weak::types::{Ref, Variant};
use std::{
	collections::HashMap,
	path::{Path, PathBuf},
};

#[derive(Debug)]
pub struct Snapshot {
	pub id: Ref,
	pub path: PathBuf,

	pub name: String,
	pub class: String,
	pub properties: HashMap<String, Variant>,
	pub children: Vec<Snapshot>,
}

impl Snapshot {
	pub fn new(name: &str) -> Self {
		Self {
			id: Ref::none(),
			path: PathBuf::new(),
			name: name.to_string(),
			class: String::new(),
			properties: HashMap::new(),
			children: Vec::new(),
		}
	}

	pub fn with_id(mut self, id: Ref) -> Self {
		self.id = id;
		self
	}

	pub fn with_path(mut self, path: &Path) -> Self {
		self.path = path.into();
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
