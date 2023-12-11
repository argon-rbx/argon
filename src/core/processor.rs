use pathsub::sub_paths;
use std::{
	path::{Path, PathBuf},
	sync::{Arc, Mutex},
};

use crate::{config::Config, lock, project::Project, utils, ROBLOX_SEPARATOR};

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

	fn is_valid(&self, path: &Path) -> bool {
		let extension = utils::get_file_extension(path);
		let path = path.to_str().unwrap_or_default();

		if !FILE_EXTENSIONS.contains(&extension) {
			return false;
		}

		if let Some(ignore_globs) = &lock!(self.project).ignore_globs {
			for glob in ignore_globs {
				if glob.matches(path) {
					return false;
				}
			}
		};

		true
	}

	fn get_roblox_path(&self, path: &Path) -> Option<String> {
		let project = lock!(self.project);

		for (index, local_path) in project.local_paths.iter().enumerate() {
			if let Some(path) = sub_paths(path, local_path) {
				let absolute = &project.roblox_paths[index];
				let extension = utils::get_file_extension(&path);
				let name = utils::get_file_name(&path);

				match extension {
					"lua" | "luau" => {
						let mut roblox_path = absolute.to_owned();
						let mut parent = path.clone();

						parent.pop();

						if parent != PathBuf::from("") {
							roblox_path.push(ROBLOX_SEPARATOR);
							roblox_path.push_str(parent.to_str().unwrap());
						}

						if !name.starts_with(&self.config.src) {
							roblox_path.push(ROBLOX_SEPARATOR);
							roblox_path.push_str(name);
						}

						return Some(roblox_path);
					}
					_ => {
						// TODO
					}
				}

				break;
			}
		}

		None
	}

	pub fn create(&self, path: &Path) {
		let queue = lock!(self.queue);

		if path.is_dir() {
			return; // TEMP!
		}

		if !self.is_valid(path) {
			return;
		}

		let file_name = path.file_stem().unwrap().to_str().unwrap();

		println!("{:?}", file_name);
	}

	pub fn delete(&self, path: &PathBuf) {
		println!("delete: {:?}", path);
	}

	pub fn write(&self, path: &Path) {
		if !self.is_valid(path) {
			return;
		}

		let queue = lock!(self.queue);

		println!("{:?}", self.get_roblox_path(path));
	}
}
