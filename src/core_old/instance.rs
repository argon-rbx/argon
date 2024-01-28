use rbx_dom_weak::types::Variant;
use std::{
	collections::HashMap,
	path::{Path, PathBuf},
};

use crate::rbx_path::RbxPath;

#[derive(Debug)]
pub struct Instance {
	pub name: String,
	pub class: String,
	pub properties: HashMap<String, Variant>,
	pub rbx_path: RbxPath,
	pub path: PathBuf,
}

impl Instance {
	pub fn new(name: &str) -> Self {
		Self {
			name: name.to_owned(),
			class: String::from("Folder"),
			properties: HashMap::new(),
			rbx_path: RbxPath::new(),
			path: PathBuf::new(),
		}
	}

	pub fn with_class(mut self, class: &str) -> Self {
		self.class = class.to_owned();
		self
	}

	pub fn with_properties(mut self, properties: HashMap<String, Variant>) -> Self {
		self.properties = properties;
		self
	}

	pub fn with_rbx_path(mut self, rbx_path: &RbxPath) -> Self {
		self.rbx_path = rbx_path.to_owned();
		self
	}

	pub fn with_path(mut self, path: &Path) -> Self {
		self.path = path.to_owned();
		self
	}
}
