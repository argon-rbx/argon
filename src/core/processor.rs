use std::path::PathBuf;

pub struct Processor {
	ignore_globs: Vec<String>,
}

impl Processor {
	pub fn new(ignore_globs: Option<Vec<String>>) -> Self {
		Self {
			ignore_globs: ignore_globs.unwrap_or_default(),
		}
	}

	pub fn create(&self, path: &PathBuf) {
		println!("create: {:?}", path);
	}

	pub fn delete(&self, path: &PathBuf) {
		println!("delete: {:?}", path);
	}

	pub fn write(&self, path: &PathBuf) {
		println!("{:?}", self.ignore_globs);
		println!("write: {:?}", path);
	}

	pub fn set_ignore_globs(&mut self, ignore_globs: Option<Vec<String>>) {
		self.ignore_globs = ignore_globs.unwrap_or_default();
	}
}
