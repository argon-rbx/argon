use std::{collections::HashMap, path::PathBuf};

use rbx_dom_weak::types::{Ref, Variant};

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
	pub fn new() -> Self {
		Self {
			id: Ref::none(),
			path: PathBuf::new(),
			name: String::new(),
			class: String::new(),
			properties: HashMap::new(),
			children: Vec::new(),
		}
	}
}
