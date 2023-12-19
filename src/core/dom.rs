use rbx_dom_weak::{types::Ref, InstanceBuilder, WeakDom};
use std::{collections::HashMap, path::PathBuf};

#[derive(Debug)]
pub struct Dom {
	inner: WeakDom,
	ref_map: HashMap<PathBuf, Ref>,
}

impl Dom {
	pub fn new(root_type: &str) -> Self {
		Self {
			inner: WeakDom::new(InstanceBuilder::new(root_type)),
			ref_map: HashMap::new(),
		}
	}
}
