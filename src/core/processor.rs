use std::{
	path::PathBuf,
	sync::{Arc, Mutex},
};

use crate::lock;

use super::queue::Queue;

const FILE_EXTENSIONS: [&str; 2] = ["lua", "luau"];

pub struct Processor {
	queue: Arc<Mutex<Queue>>,
	ignore_globs: Vec<String>,
}

impl Processor {
	pub fn new(queue: Arc<Mutex<Queue>>, ignore_globs: Option<Vec<String>>) -> Self {
		Self {
			queue,
			ignore_globs: ignore_globs.unwrap_or_default(),
		}
	}

	pub fn set_ignore_globs(&mut self, ignore_globs: Option<Vec<String>>) {
		self.ignore_globs = ignore_globs.unwrap_or_default();
	}

	pub fn create(&self, path: &PathBuf) {
		let queue = lock!(self.queue);

		if path.is_dir() {
			return;
		}

		let extension = path.extension().unwrap_or_default();

		if !FILE_EXTENSIONS.contains(&extension.to_str().unwrap()) {
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
