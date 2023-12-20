use anyhow::{bail, Result};
use log::{info, warn};
use pathsub::sub_paths;
use rbx_dom_weak::types::Ref;
use rbx_reflection::ClassTag;
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

const FILE_EXTENSIONS: [&str; 7] = ["lua", "luau", "json", "csv", "txt", "rbxm", "rbxmx"];

#[derive(Debug, Clone, PartialEq)]
pub enum FileKind {
	ServerScript,      // .server.lua
	ClientScript,      // .client.lua
	ModuleScript,      // .lua
	InstanceData,      // .data.json
	JsonModule,        // .json
	LocalizationTable, // .csv
	StringValue,       // .txt
	BinaryModel,       // .rbxm
	XmlModel,          // .rbxmx
	Other,             // dir
}

// This will be removed in the future
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

	fn is_place(&self) -> bool {
		lock!(self.project).root_class == "DataModel"
	}

	fn is_service(&self, class: &str) -> bool {
		let class = rbx_reflection_database::get().classes.get(class);

		if let Some(class) = class {
			class.tags.contains(&ClassTag::Service)
		} else {
			false
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

	fn get_rbx_path(&self, path: &Path, name: &str, ext: &str) -> Result<RbxPath> {
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

				return Ok(rbx_path);
			}
		}

		bail!("{:?} does not exists in the project file", path)
	}

	fn get_file_kind(&self, name: &str, ext: &str, is_dir: bool) -> Result<FileKind> {
		if is_dir {
			return Ok(FileKind::Other);
		}

		if ext == "lua" || ext == "luau" {
			if name.ends_with(".server") {
				return Ok(FileKind::ServerScript);
			} else if name.ends_with(".client") {
				return Ok(FileKind::ClientScript);
			} else {
				return Ok(FileKind::ModuleScript);
			}
		} else if ext == "json" {
			if name == self.config.data {
				return Ok(FileKind::InstanceData);
			} else {
				return Ok(FileKind::JsonModule);
			}
		} else if ext == "csv" {
			return Ok(FileKind::LocalizationTable);
		} else if ext == "txt" {
			return Ok(FileKind::StringValue);
		} else if ext == "rbxm" {
			return Ok(FileKind::BinaryModel);
		} else if ext == "rbxmx" {
			return Ok(FileKind::XmlModel);
		}

		bail!(".{} extension is not supported", ext)
	}

	pub fn get_class(&self, kind: &FileKind, path: Option<&Path>, rbx_path: Option<&RbxPath>) -> Result<String> {
		let class = match kind {
			FileKind::ServerScript => "Script",
			FileKind::ClientScript => "LocalScript",
			FileKind::ModuleScript => "ModuleScript",
			FileKind::JsonModule => "ModuleScript",
			FileKind::LocalizationTable => "LocalizationTable",
			FileKind::StringValue => "StringValue",
			FileKind::Other => {
				if let Some(path) = path {
					println!("{:?}", path);
					"temp"
				} else if let Some(rbx_path) = rbx_path {
					if self.is_place() {
						if rbx_path.len() == 2 && self.is_service(&rbx_path[1]) {
							&rbx_path[1]
						} else {
							"Folder"
						}
					} else {
						"Folder"
					}
				} else {
					"Folder"
				}
			}
			_ => bail!("Cannot get class of {:?} file kind", kind),
		};

		Ok(String::from(class))
	}

	fn get_parent(&self, rbx_path: &RbxPath) -> RbxPath {
		let mut parent = rbx_path.to_owned();
		parent.pop();

		parent
	}

	pub fn init(&self, path: &Path) -> Result<()> {
		let ext = utils::get_file_extension(path);
		let is_dir = ext.is_empty();

		if !self.is_valid(path, ext, is_dir) {
			return Ok(());
		}

		let project = lock!(self.project);
		let rbx_path = project.rbx_paths[utils::get_index(&project.local_paths, &path.to_path_buf()).unwrap()].clone();

		drop(project);

		let name = utils::get_file_name(path);
		let kind = self.get_file_kind(name, ext, is_dir)?;
		let class = self.get_class(&kind, None, Some(&rbx_path))?;

		Ok(())
	}

	pub fn create(&self, path: &Path, dom_only: bool) -> Result<()> {
		let ext = utils::get_file_extension(path);
		let is_dir = path.is_dir();

		if !self.is_valid(path, ext, is_dir) {
			return Ok(());
		}

		bail!("temp");

		let name = utils::get_file_name(path);
		let rbx_path = self.get_rbx_path(path, name, ext)?;
		let file_kind = self.get_file_kind(name, ext, is_dir)?;
		let parent = self.get_parent(&rbx_path);

		let mut dom = lock!(self.dom);

		// let dom_ref = dom.get_ref(&parent).unwrap();

		// println!("{:?}", dom_ref);

		if !dom_only {
			println!("{:?}", "message");
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

		// queue.push(Message::Sync(Sync {
		// 	action: SyncAction::Delete,
		// 	path: rbx_path.unwrap(),
		// 	kind: None,
		// 	data: None,
		// }));

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

		// if let Some(file_kind) = file_kind {
		// 	let mut queue = lock!(self.queue);
		// 	let content = fs::read_to_string(path)?;

		// 	if file_kind != FileKind::InstanceData {
		// 		queue.push(Message::Sync(Sync {
		// 			action: SyncAction::Write,
		// 			path: rbx_path.unwrap(),
		// 			kind: None,
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

		// 	info!("Write: {:?}", path);
		// } else {
		// 	warn!("Unknown file kind: {:?}", path);
		// };

		Ok(())
	}
}
