use rbx_dom_weak::{types::Ref, WeakDom};
use std::{collections::HashMap, path::PathBuf};

use crate::project::Project;

pub struct Dom {
	inner: WeakDom,
	ref_map: HashMap<PathBuf, Ref>,
}

impl Dom {
	pub fn new() -> Self {
		Self {
			inner: WeakDom::default(),
			ref_map: HashMap::new(),
		}
	}

	pub fn from_project(project: &Project) -> Self {
		// TODO
		Self::new()
	}
}
