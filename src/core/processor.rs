use std::{
	path::{Path, PathBuf},
	sync::{Arc, Mutex},
};

use crate::{glob::Glob, lock};

use super::queue::Queue;

const FILE_EXTENSIONS: [&str; 2] = ["lua", "luau"];

pub struct Processor {
	queue: Arc<Mutex<Queue>>,
	ignore_globs: Vec<Glob>,
}

impl Processor {
	pub fn new(queue: Arc<Mutex<Queue>>, ignore_globs: Option<Vec<Glob>>) -> Self {
		Self {
			queue,
			ignore_globs: ignore_globs.unwrap_or_default(),
		}
	}

	pub fn set_ignore_globs(&mut self, ignore_globs: Option<Vec<Glob>>) {
		self.ignore_globs = ignore_globs.unwrap_or_default();
	}

	fn is_valid(&self, path: &Path) -> bool {
		let extension = path.extension().unwrap_or_default();
		let path = path.to_str().unwrap_or_default();

		if !FILE_EXTENSIONS.contains(&extension.to_str().unwrap()) {
			return false;
		}

		for glob in &self.ignore_globs {
			if glob.matches(path) {
				return false;
			}
		}

		true
	}

	pub fn create(&self, path: &Path) {
		let queue = lock!(self.queue);

		if path.is_dir() {
			return; // TEMP!
		}

		if !self.is_valid(path) {
			return;
		}

		println!("{:?}", path.file_name());
	}

	pub fn delete(&self, path: &PathBuf) {
		println!("delete: {:?}", path);
	}

	pub fn write(&self, path: &PathBuf) {
		println!("write: {:?}", path);
	}
}
