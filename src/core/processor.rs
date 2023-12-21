use anyhow::{bail, Result};
use log::{info, warn};
use pathsub::sub_paths;
use rbx_dom_weak::types::Ref;
use rbx_reflection::ClassTag;
use serde_json::{from_reader, Value};
use std::{
	collections::HashMap,
	fs::{self, File},
	io::BufReader,
	path::Path,
	sync::{Arc, Mutex},
};

use crate::{
	config::Config,
	lock,
	messages::{Message, Sync, SyncAction},
	project::Project,
	rbx_path::RbxPath,
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
	Dir,               // dir
}

pub struct Processor {
	dom: Arc<Mutex<Dom>>,
	queue: Arc<Mutex<Queue>>,
	project: Arc<Mutex<Project>>,
	config: Arc<Config>,
	data_file: String,
}

impl Processor {
	pub fn new(
		dom: Arc<Mutex<Dom>>,
		queue: Arc<Mutex<Queue>>,
		project: Arc<Mutex<Project>>,
		config: Arc<Config>,
	) -> Self {
		let mut data_file = config.data.clone();
		data_file.push_str(".json");

		Self {
			dom,
			queue,
			project,
			config,
			data_file,
		}
	}

	fn is_place(&self) -> bool {
		lock!(self.project).root_class == "DataModel"
	}

	fn is_service(&self, class: &str) -> bool {
		let descriptor = rbx_reflection_database::get().classes.get(class);

		let has_tag = if let Some(descriptor) = descriptor {
			descriptor.tags.contains(&ClassTag::Service)
		} else {
			false
		};

		has_tag || class == "StarterPlayerScripts" || class == "StarterCharacterScripts"
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

	fn get_rbx_paths(&self, path: &Path, name: &str, ext: &str) -> Result<Vec<RbxPath>> {
		let project = lock!(self.project);

		for local_path in project.get_paths() {
			if let Some(diff) = sub_paths(path, &local_path) {
				let mut rbx_paths = project.path_map.get_vec(&local_path).unwrap().clone();
				let mut parent = diff.clone();

				parent.pop();

				for rbx_path in rbx_paths.iter_mut() {
					for comp in parent.iter() {
						let comp = utils::from_os_str(comp);
						rbx_path.push(comp);
					}

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
				}

				return Ok(rbx_paths);
			}
		}

		bail!("{:?} does not exists in the project file", path)
	}

	fn get_file_kind(&self, name: &str, ext: &str, is_dir: bool) -> Result<FileKind> {
		if is_dir {
			return Ok(FileKind::Dir);
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
		#[allow(unused_assignments)]
		let mut temp = String::new();

		let class = match kind {
			FileKind::ServerScript => "Script",
			FileKind::ClientScript => "LocalScript",
			FileKind::ModuleScript => "ModuleScript",
			FileKind::JsonModule => "ModuleScript",
			FileKind::LocalizationTable => "LocalizationTable",
			FileKind::StringValue => "StringValue",
			FileKind::Dir => {
				if let Some(path) = path {
					let data_file = path.join(&self.data_file);

					if data_file.exists() {
						let result = || -> Result<&str> {
							let data_file = File::open(data_file)?;
							let reader = BufReader::new(data_file);
							let data: HashMap<String, Value> = from_reader(reader)?;

							if data.contains_key("ClassName") && data["ClassName"].is_string() {
								// Sketchy solution to get around borrow checker
								temp = data["ClassName"].as_str().unwrap().to_owned();
								Ok(&temp)
							} else {
								Ok("Folder")
							}
						};

						if let Ok(class) = result() {
							class
						} else {
							"Folder"
						}
					} else {
						"Folder"
					}
				} else if let Some(rbx_path) = rbx_path {
					if self.is_place() {
						let len = rbx_path.len();

						if len == 2 && self.is_service(&rbx_path[1]) {
							&rbx_path[1]
						} else if len == 3 && self.is_service(&rbx_path[1]) && self.is_service(&rbx_path[2]) {
							&rbx_path[2]
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
		let rbx_paths = project.path_map.get_vec(path).unwrap().clone();

		drop(project);

		for rbx_path in rbx_paths {
			let mut dom = lock!(self.dom);
			let mut cur_rbx_path = RbxPath::new();
			let mut last_ref = Ref::new();

			for (index, comp) in rbx_path.iter().enumerate() {
				cur_rbx_path.push(comp);

				if dom.contains(&cur_rbx_path) {
					last_ref = dom.get_ref(&cur_rbx_path).unwrap();
				} else if index != rbx_path.len() - 1 {
					let dom_ref = dom.insert(last_ref, comp, path, cur_rbx_path.clone());
					let class = self.get_class(&FileKind::Dir, None, Some(&cur_rbx_path))?;

					dom.set_class(dom_ref, &class);
					last_ref = dom_ref;
				} else {
					let kind = self.get_file_kind(comp, ext, is_dir)?;
					let class = self.get_class(&kind, None, Some(&cur_rbx_path))?;

					let dom_ref = dom.insert(last_ref, comp, path, cur_rbx_path.clone());
					dom.set_class(dom_ref, &class);
				}
			}
		}

		Ok(())
	}

	pub fn create(&self, path: &Path, dom_only: bool) -> Result<()> {
		let ext = utils::get_file_extension(path);
		let is_dir = path.is_dir();

		if !self.is_valid(path, ext, is_dir) {
			return Ok(());
		}

		let name = utils::get_file_name(path);
		let rbx_paths = self.get_rbx_paths(path, name, ext)?;
		let file_kind = self.get_file_kind(name, ext, is_dir)?;

		let mut dom = lock!(self.dom);

		for rbx_path in rbx_paths {
			let parent = self.get_parent(&rbx_path);
			let parent = dom.get_ref(&parent).unwrap();

			match file_kind {
				FileKind::ServerScript
				| FileKind::ClientScript
				| FileKind::ModuleScript
				| FileKind::JsonModule
				| FileKind::StringValue
				| FileKind::LocalizationTable
				| FileKind::Dir => {
					let class = self.get_class(&file_kind, Some(path), None)?;
					let dom_ref = dom.insert(parent, name, path, rbx_path);
					dom.set_class(dom_ref, &class);
				}
				_ => bail!("Unimplemented!"),
			}

			if !dom_only {
				println!("{:?}", "message");
			}
		}

		drop(dom);

		if path.is_dir() {
			for entry in fs::read_dir(path)? {
				self.create(&entry?.path(), dom_only)?;
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
		let rbx_path = self.get_rbx_paths(path, name, ext);
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

		let rbx_path = self.get_rbx_paths(path, name, ext);
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
