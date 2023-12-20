use anyhow::Result;
use log::{info, warn};
use pathsub::sub_paths;
use rbx_dom_weak::types::Ref;
use std::{
	fs,
	path::{Path, PathBuf},
	sync::{Arc, Mutex},
};

use crate::{
	config::Config,
	lock,
	messages::{Message, Sync, SyncAction},
	project::Project,
	types::{RbxKind, RbxPath},
	utils,
};

use super::{dom::Dom, queue::Queue};

const FILE_EXTENSIONS: [&str; 3] = ["lua", "luau", "json"];

#[derive(Debug, Clone, PartialEq)]
pub enum FileKind {
	ServerScript,
	ClientScript,
	ModuleScript,
	Properties,
	Other,
}

impl From<FileKind> for RbxKind {
	fn from(kind: FileKind) -> Self {
		match kind {
			FileKind::ServerScript => RbxKind::ServerScript,
			FileKind::ClientScript => RbxKind::ClientScript,
			FileKind::ModuleScript => RbxKind::ModuleScript,
			FileKind::Other => RbxKind::Other,
			_ => panic!("Cannot convert FileKind to RbxKind"),
		}
	}
}

pub struct Processor {
	dom: Arc<Mutex<Dom>>,
	queue: Arc<Mutex<Queue>>,
	project: Arc<Mutex<Project>>,
	config: Arc<Config>,
}

impl Processor {
	pub fn new(
		dom: Arc<Mutex<Dom>>,
		queue: Arc<Mutex<Queue>>,
		project: Arc<Mutex<Project>>,
		config: Arc<Config>,
	) -> Self {
		Self {
			dom,
			queue,
			project,
			config,
		}
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

	fn get_rbx_path(&self, path: &Path, name: &str, ext: &str) -> Option<RbxPath> {
		let project = lock!(self.project);

		for (index, local_path) in project.local_paths.iter().enumerate() {
			if let Some(path) = sub_paths(path, local_path) {
				let mut rbx_path = project.rbx_paths[index].clone();
				let mut parent = path.clone();

				parent.pop();
				rbx_path.push(parent.to_str().unwrap());

				match ext {
					"lua" | "luau" => {
						if !name.starts_with(&self.config.src) {
							rbx_path.push(name);
						}
					}
					"json" => {
						if !name.starts_with(&self.config.data) {
							rbx_path.push(name);
						}
					}
					_ => {
						rbx_path.push(name);
					}
				}

				return Some(rbx_path);
			}
		}

		None
	}

	fn get_file_kind(&self, name: &str, ext: &str, is_dir: bool) -> Option<FileKind> {
		if is_dir {
			return Some(FileKind::Other);
		}

		if ext == "lua" || ext == "luau" {
			if name.ends_with(".server") {
				return Some(FileKind::ServerScript);
			} else if name.ends_with(".client") {
				return Some(FileKind::ClientScript);
			} else {
				return Some(FileKind::ModuleScript);
			}
		}

		if ext == "json" {
			if name == self.config.data {
				return Some(FileKind::Properties);
			} else {
				return None;
			}
		}

		Some(FileKind::Other)
	}

	// fn get_parent(&self, path: &Path) -> PathBuf {
	// 	let mut parent = path.to_owned();
	// 	parent.pop();

	// 	parent
	// }

	pub fn create(&self, path: &Path) -> Result<()> {
		let ext = utils::get_file_extension(path);
		let is_dir = path.is_dir();

		if !self.is_valid(path, ext, is_dir) {
			return Ok(());
		}

		let name = utils::get_file_name(path);

		let rbx_path = self.get_rbx_path(path, name, ext).unwrap();
		// let file_kind = self.get_file_kind(name, ext, is_dir);

		let mut dom = lock!(self.dom);

		let mut cur_rbx_path = RbxPath::new();
		let mut last_ref = Ref::new();

		for comp in rbx_path.iter() {
			cur_rbx_path.push(comp);

			if dom.contains(&cur_rbx_path) {
				last_ref = dom.get_ref(&cur_rbx_path).unwrap();
			} else {
				dom.insert(last_ref, comp, path.to_owned(), cur_rbx_path.clone());
			}
		}

		// if let Some(file_kind) = file_kind {
		// 	let mut queue = lock!(self.queue);
		// 	let content = fs::read_to_string(path)?;

		// 	if file_kind != FileKind::Properties {
		// 		queue.push(Message::Sync(Sync {
		// 			action: SyncAction::Create,
		// 			path: rbx_path.unwrap(),
		// 			kind: Some(file_kind.into()),
		// 			data: Some(content),
		// 		}));
		// 	} else {
		// 		queue.push(Message::Sync(Sync {
		// 			action: SyncAction::Update,
		// 			path: rbx_path.unwrap(),
		// 			kind: None,
		// 			data: Some(content),
		// 		}));
		// 	}

		// 	info!("Create: {:?}", path);
		// } else {
		// 	warn!("Unknown file kind: {:?}", path);
		// };

		Ok(())
	}

	pub fn delete(&self, path: &Path) -> Result<()> {
		let ext = utils::get_file_extension(path);

		if !self.is_valid(path, ext, false) {
			return Ok(());
		}

		let name = utils::get_file_name(path);
		let rbx_path = self.get_rbx_path(path, name, ext);
		let mut queue = lock!(self.queue);

		queue.push(Message::Sync(Sync {
			action: SyncAction::Delete,
			path: rbx_path.unwrap(),
			kind: None,
			data: None,
		}));

		Ok(())
	}

	pub fn write(&self, path: &Path) -> Result<()> {
		let ext = utils::get_file_extension(path);

		if !self.is_valid(path, ext, false) {
			return Ok(());
		}

		let name = utils::get_file_name(path);
		let is_dir = path.is_dir();

		let rbx_path = self.get_rbx_path(path, name, ext);
		let file_kind = self.get_file_kind(name, ext, is_dir);

		if let Some(file_kind) = file_kind {
			let mut queue = lock!(self.queue);
			let content = fs::read_to_string(path)?;

			if file_kind != FileKind::Properties {
				queue.push(Message::Sync(Sync {
					action: SyncAction::Write,
					path: rbx_path.unwrap(),
					kind: None,
					data: Some(content),
				}));
			} else {
				queue.push(Message::Sync(Sync {
					action: SyncAction::Update,
					path: rbx_path.unwrap(),
					kind: None,
					data: Some(content),
				}));
			}

			info!("Write: {:?}", path);
		} else {
			warn!("Unknown file kind: {:?}", path);
		};

		Ok(())
	}
}
