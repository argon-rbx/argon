use anyhow::{bail, Result};
use pathsub::sub_paths;
use rbx_dom_weak::types::{Enum, Variant};
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

use super::{dom::Dom, instance::Instance, queue::Queue};

const FILE_EXTENSIONS: [&str; 7] = ["lua", "luau", "json", "csv", "txt", "rbxm", "rbxmx"];

#[derive(Debug, Clone, PartialEq)]
pub enum ScriptKind {
	Server,
	Client,
	Module,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ModelKind {
	Binary,
	Xml,
}

#[derive(Debug, Clone, PartialEq)]
pub enum FileKind {
	Script(ScriptKind),      // *.lua(u)
	ChildScript(ScriptKind), // .src.lua(u)
	InstanceData,            // .data.json
	JsonModule,              // .json
	LocalizationTable,       // .csv
	StringValue,             // .txt
	Model(ModelKind),        // .rbxm, .rbxmx
	Dir,                     // dir
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
			let kind = if name.ends_with(".server") {
				ScriptKind::Server
			} else if name.ends_with(".client") {
				ScriptKind::Client
			} else {
				ScriptKind::Module
			};

			if name.starts_with(&self.config.src) {
				return Ok(FileKind::ChildScript(kind));
			} else {
				return Ok(FileKind::Script(kind));
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
			return Ok(FileKind::Model(ModelKind::Binary));
		} else if ext == "rbxmx" {
			return Ok(FileKind::Model(ModelKind::Xml));
		}

		bail!(".{} extension is not supported", ext)
	}

	pub fn get_class(&self, kind: &FileKind, path: Option<&Path>, rbx_path: Option<&RbxPath>) -> Result<String> {
		#[allow(unused_assignments)]
		let mut temp = String::new();

		let class = match kind {
			FileKind::Script(kind) | FileKind::ChildScript(kind) => match kind {
				ScriptKind::Server => "Script",
				ScriptKind::Client => "LocalScript",
				ScriptKind::Module => "ModuleScript",
			},
			FileKind::JsonModule => "ModuleScript",
			FileKind::LocalizationTable => "LocalizationTable",
			FileKind::StringValue => "StringValue",
			FileKind::Dir => {
				if let Some(path) = path {
					let data_file = path.join(&self.data_file);

					if data_file.exists() {
						|| -> Result<&str> {
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
						}()
						.unwrap_or("Folder")
					} else {
						"Folder"
					}
				} else if let Some(rbx_path) = rbx_path {
					if lock!(self.project).is_place() {
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

	fn get_name(&self, kind: &FileKind, rbx_path: &RbxPath, name: &str) -> String {
		match kind {
			FileKind::Script(ref kind) => {
				if *kind != ScriptKind::Module {
					let pos = if *kind == ScriptKind::Server {
						name.rfind(".server").unwrap()
					} else {
						name.rfind(".client").unwrap()
					};

					name[..pos].to_owned()
				} else {
					name.to_owned()
				}
			}
			FileKind::ChildScript(_) => rbx_path.last().unwrap().clone(),
			_ => name.to_owned(),
		}
	}

	fn get_parent(&self, rbx_path: &RbxPath) -> RbxPath {
		let mut parent = rbx_path.to_owned();
		parent.pop();

		parent
	}

	fn get_properties(&self, kind: &FileKind, path: &Path) -> Result<HashMap<String, Variant>> {
		let mut properties = HashMap::new();

		match kind {
			FileKind::Script(kind) | FileKind::ChildScript(kind) => {
				let source = fs::read_to_string(path)?;

				if *kind != ScriptKind::Module {
					if let Some(line) = source.lines().next() {
						if line.contains("--disable") {
							properties.insert(String::from("Disabled"), Variant::Bool(true));
						}

						if line.contains("--server") {
							properties.insert(String::from("RunContext"), Variant::Enum(Enum::from_u32(1)));
						} else if line.contains("--client") {
							properties.insert(String::from("RunContext"), Variant::Enum(Enum::from_u32(2)));
						} else if line.contains("--plugin") {
							properties.insert(String::from("RunContext"), Variant::Enum(Enum::from_u32(3)));
						}
					}
				}

				properties.insert(String::from("Source"), Variant::String(source));
			}
			FileKind::JsonModule => {
				let mut source = String::from("return ");

				let json = fs::read_to_string(path)?;
				let lua = json2lua::parse(&json)?;

				source.push_str(&lua);

				properties.insert(String::from("Source"), Variant::String(source));
			}
			FileKind::LocalizationTable => {
				// TODO: Implement
			}
			FileKind::StringValue => {
				let value = fs::read_to_string(path)?;

				properties.insert(String::from("Value"), Variant::String(value));
			}
			FileKind::Dir => {
				let data_file = path.join(&self.data_file);

				if data_file.exists() {
					let reader = BufReader::new(File::open(data_file)?);
					let data: HashMap<String, Value> = from_reader(reader)?;

					for (_key, _value) in data {
						// TODO: Implement
					}
				}
			}
			_ => bail!("Cannot get properties of {:?} file kind", kind),
		}

		Ok(properties)
	}

	pub fn init(&self, path: &Path) -> Result<()> {
		let ext = utils::get_file_ext(path);
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

			for (index, comp) in rbx_path.iter().enumerate() {
				cur_rbx_path.push(comp);

				if dom.contains(&cur_rbx_path) {
					continue;
				} else if index != rbx_path.len() - 1 {
					let class = self.get_class(&FileKind::Dir, None, Some(&cur_rbx_path))?;
					let parent = self.get_parent(&cur_rbx_path);

					let instance = Instance::new(comp)
						.with_class(&class)
						.with_rbx_path(&cur_rbx_path)
						.with_path(path);

					dom.insert(instance, &parent);
				} else {
					let kind = self.get_file_kind(comp, ext, is_dir)?;
					let class = self.get_class(&kind, None, Some(&cur_rbx_path))?;
					let parent = self.get_parent(&cur_rbx_path);

					let instance = Instance::new(comp)
						.with_class(&class)
						.with_rbx_path(&cur_rbx_path)
						.with_path(path);

					dom.insert(instance, &parent);
				}
			}
		}

		Ok(())
	}

	pub fn create(&self, path: &Path, dom_only: bool) -> Result<()> {
		let ext = utils::get_file_ext(path);
		let is_dir = path.is_dir();

		if !self.is_valid(path, ext, is_dir) {
			return Ok(());
		}

		let name = utils::get_file_stem(path);
		let rbx_paths = self.get_rbx_paths(path, name, ext)?;
		let file_kind = self.get_file_kind(name, ext, is_dir)?;

		let mut dom = lock!(self.dom);

		for rbx_path in rbx_paths {
			let parent = self.get_parent(&rbx_path);

			match file_kind {
				FileKind::ChildScript(_)
				| FileKind::Script(_)
				| FileKind::JsonModule
				| FileKind::StringValue
				| FileKind::LocalizationTable
				| FileKind::Dir => {
					let class = self.get_class(&file_kind, Some(path), None)?;
					let properties = self.get_properties(&file_kind, path)?;
					let name = self.get_name(&file_kind, &rbx_path, name);

					let instance = Instance::new(&name)
						.with_class(&class)
						.with_properties(properties)
						.with_rbx_path(&rbx_path)
						.with_path(path);

					dom.insert(instance, &parent);
				}
				_ => bail!("Unimplemented!"),
			}

			if !dom_only {
				// println!("{:?}", "message, rbx_path");
			}
		}

		drop(dom);

		if path.is_dir() {
			for entry in fs::read_dir(path)? {
				self.create(&entry?.path(), dom_only)?;
			}
		}

		Ok(())
	}

	pub fn delete(&self, path: &Path) -> Result<()> {
		let ext = utils::get_file_ext(path);

		if !self.is_valid(path, ext, true) {
			return Ok(());
		}

		let name = utils::get_file_stem(path);
		let rbx_paths = self.get_rbx_paths(path, name, ext)?;

		let mut dom = lock!(self.dom);

		for rbx_path in rbx_paths {
			if dom.remove(&rbx_path) {
				// println!("{:?}", "message, rbx_path");
			}
		}

		Ok(())
	}

	pub fn write(&self, path: &Path) -> Result<()> {
		let ext = utils::get_file_ext(path);

		if !self.is_valid(path, ext, false) {
			return Ok(());
		}

		let name = utils::get_file_stem(path);
		let rbx_paths = self.get_rbx_paths(path, name, ext)?;
		let file_kind = self.get_file_kind(name, ext, false)?;

		let mut dom = lock!(self.dom);

		for rbx_path in rbx_paths {
			let instance = dom.get_mut(&rbx_path).unwrap();
			let properties = self.get_properties(&file_kind, path)?;

			instance.properties = properties;

			// println!("{:?}", "message, rbx_path");
		}

		Ok(())
	}
}
