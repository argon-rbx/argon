use pathsub::sub_paths;
use std::{
	path::Path,
	sync::{Arc, Mutex},
};

use crate::{config::Config, lock, project::Project, types::RobloxPath, utils};

use super::queue::Queue;

const FILE_EXTENSIONS: [&str; 3] = ["lua", "luau", "json"];

pub struct Processor {
	queue: Arc<Mutex<Queue>>,
	project: Arc<Mutex<Project>>,
	config: Arc<Config>,
}

impl Processor {
	pub fn new(queue: Arc<Mutex<Queue>>, project: Arc<Mutex<Project>>, config: Arc<Config>) -> Self {
		Self { queue, project, config }
	}

	fn is_valid(&self, path: &Path, ext: &str, is_dir: bool) -> bool {
		if !FILE_EXTENSIONS.contains(&ext) && !is_dir {
			return false;
		}

		let path = path.to_str().unwrap_or_default();

		if let Some(ignore_globs) = &lock!(self.project).ignore_globs {
			for glob in ignore_globs {
				if glob.matches(path) {
					return false;
				}
			}
		};

		true
	}

	fn get_roblox_path(&self, path: &Path, name: &str, ext: &str) -> Option<RobloxPath> {
		let project = lock!(self.project);

		for (index, local_path) in project.local_paths.iter().enumerate() {
			if let Some(path) = sub_paths(path, local_path) {
				let absolute = &project.roblox_paths[index];

				let mut roblox_path = absolute.clone();
				let mut parent = path.clone();

				parent.pop();
				roblox_path.push(parent.to_str().unwrap());

				match ext {
					"lua" | "luau" => {
						if !name.starts_with(&self.config.src) {
							roblox_path.push(name);
						}
					}
					"json" => {
						if !name.starts_with(&self.config.data) {
							roblox_path.push(name);
						}
					}
					_ => {
						roblox_path.push(name);
					}
				}

				return Some(roblox_path);
			}
		}

		None
	}

	pub fn create(&self, path: &Path) {
		let ext = utils::get_file_extension(path);
		let is_dir = path.is_dir();

		if !self.is_valid(path, ext, is_dir) {
			return;
		}

		let queue = lock!(self.queue);

		let name = utils::get_file_name(path);
		let extension = utils::get_file_extension(path);

		println!("{:?}", self.get_roblox_path(path, name, extension));
	}

	pub fn delete(&self, path: &Path) {
		// println!("delete: {:?}", path);
	}

	pub fn write(&self, path: &Path) {
		let ext = utils::get_file_extension(path);

		if !self.is_valid(path, ext, false) {
			return;
		}

		let name = utils::get_file_name(path);

		let queue = lock!(self.queue);

		println!("{:?}", self.get_roblox_path(path, name, ext));
	}
}
