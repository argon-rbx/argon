use std::{
	path::PathBuf,
	sync::{Arc, Mutex},
};

use crate::{lock, project::Project};

pub struct Processor {
	project: Arc<Mutex<Project>>,
}

impl Processor {
	pub fn new(project: Arc<Mutex<Project>>) -> Self {
		Self { project }
	}

	pub fn create(&self, path: &PathBuf) {
		println!("create: {:?}", path);
	}

	pub fn delete(&self, path: &PathBuf) {
		println!("delete: {:?}", path);
	}

	pub fn write(&self, path: &PathBuf) {
		println!("{:?}", lock!(self.project).name);
		println!("write: {:?}", path);
	}
}
