use anyhow::{bail, Result};
use pathsub::sub_paths;
use rbx_dom_weak::types::{Enum, Variant};
use rbx_xml::DecodeOptions;
use serde_json::Value;
use std::{
	collections::HashMap,
	fs::{self, File},
	io::BufReader,
	path::Path,
	sync::{Arc, Mutex},
};

use super::{dom::Dom, instance::Instance, queue::Queue};
use crate::{
	config::Config,
	lock,
	messages::{Create, Delete, Message, Update},
	project::Project,
	rbx_path::RbxPath,
	util,
};

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
		let data_file = format!("{}.json", config.data());

		Self {
			dom,
			queue,
			project,
			config,
			data_file,
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

	fn get_rbx_paths(&self, path: &Path, file_name: &str, ext: &str) -> Result<Vec<RbxPath>> {
		let project = lock!(self.project);

		for local_path in project.get_paths() {
			if let Some(diff) = sub_paths(path, &local_path) {
				let mut rbx_paths = project.path_map.get_vec(&local_path).unwrap().clone();
				let parent = diff.parent().unwrap();

				for rbx_path in rbx_paths.iter_mut() {
					for comp in parent.iter() {
						let comp = util::from_os_str(comp);
						rbx_path.push(comp);
					}

					match ext {
						"lua" | "luau" => {
							if !file_name.starts_with(self.config.src()) {
								let name = if file_name.ends_with(".server") || file_name.ends_with(".client") {
									&file_name[..file_name.len() - 7]
								} else {
									file_name
								};

								rbx_path.push(name);
							}
						}
						"json" => {
							if !file_name.starts_with(self.config.data()) {
								rbx_path.push(file_name);
							}
						}
						_ => {
							rbx_path.push(file_name);
						}
					}
				}

				return Ok(rbx_paths);
			}
		}

		bail!("{:?} does not exists in the project file", path)
	}

	fn get_file_kind(&self, file_name: &str, ext: &str, is_dir: bool) -> Result<FileKind> {
		if is_dir {
			return Ok(FileKind::Dir);
		}

		if ext == "lua" || ext == "luau" {
			let kind = if file_name.ends_with(".server") {
				ScriptKind::Server
			} else if file_name.ends_with(".client") {
				ScriptKind::Client
			} else {
				ScriptKind::Module
			};

			if file_name.starts_with(self.config.src()) {
				return Ok(FileKind::ChildScript(kind));
			} else {
				return Ok(FileKind::Script(kind));
			}
		} else if ext == "json" {
			if file_name == self.config.data() {
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
							let data: HashMap<String, Value> = serde_json::from_reader(reader)?;

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

						if len == 2 && util::is_service(&rbx_path[1]) {
							&rbx_path[1]
						} else if len == 3 && util::is_service(&rbx_path[1]) && util::is_service(&rbx_path[2]) {
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

	fn get_name(&self, kind: &FileKind, rbx_path: &RbxPath, file_name: &str) -> String {
		match kind {
			FileKind::Script(ref kind) => {
				if *kind != ScriptKind::Module {
					let pos = if *kind == ScriptKind::Server {
						file_name.rfind(".server").unwrap()
					} else {
						file_name.rfind(".client").unwrap()
					};

					file_name[..pos].to_owned()
				} else {
					file_name.to_owned()
				}
			}
			FileKind::ChildScript(_) => rbx_path.last().unwrap().clone(),
			_ => file_name.to_owned(),
		}
	}

	fn get_parent(&self, rbx_path: &RbxPath) -> RbxPath {
		rbx_path.parent().unwrap()
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
				let source = util::json::read_module(path)?;

				properties.insert(String::from("Source"), Variant::String(source));
			}
			FileKind::LocalizationTable => {
				let contents = util::csv::read_localization_table(path)?;

				properties.insert(String::from("Contents"), Variant::String(contents));
			}
			FileKind::StringValue => {
				let value = fs::read_to_string(path)?;

				properties.insert(String::from("Value"), Variant::String(value));
			}
			FileKind::Dir | FileKind::InstanceData => {
				let data_file = if *kind == FileKind::Dir {
					path.join(&self.data_file)
				} else {
					path.to_owned()
				};

				if data_file.exists() {
					properties.extend(util::json::read_properties(&data_file)?);
				}
			}
			_ => bail!("Cannot get properties of {:?} file kind", kind),
		}

		Ok(properties)
	}

	pub fn init(&self, path: &Path) -> Result<()> {
		let ext = util::get_file_ext(path);
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

					let properties = if path.exists() {
						self.get_properties(&FileKind::Dir, path)?
					} else {
						HashMap::new()
					};

					let instance = Instance::new(comp)
						.with_class(&class)
						.with_properties(properties)
						.with_rbx_path(&cur_rbx_path)
						.with_path(path);

					dom.insert(instance, &parent);
				} else {
					let kind = self.get_file_kind(comp, ext, is_dir)?;
					let class = self.get_class(&kind, None, Some(&cur_rbx_path))?;
					let parent = self.get_parent(&cur_rbx_path);

					let properties = if path.is_file() {
						self.get_properties(&kind, path)?
					} else {
						HashMap::new()
					};

					let instance = Instance::new(comp)
						.with_class(&class)
						.with_properties(properties)
						.with_rbx_path(&cur_rbx_path)
						.with_path(path);

					dom.insert(instance, &parent);
				}
			}
		}

		Ok(())
	}

	pub fn create(&self, path: &Path, dom_only: bool) -> Result<()> {
		let ext = util::get_file_ext(path);
		let is_dir = path.is_dir();

		if !self.is_valid(path, ext, is_dir) {
			return Ok(());
		}

		let file_name = util::get_file_stem(path);
		let rbx_paths = self.get_rbx_paths(path, file_name, ext)?;
		let file_kind = self.get_file_kind(file_name, ext, is_dir)?;

		let mut dom = lock!(self.dom);
		let mut queue = lock!(self.queue);

		for rbx_path in rbx_paths {
			match file_kind {
				FileKind::ChildScript(_)
				| FileKind::Script(_)
				| FileKind::JsonModule
				| FileKind::StringValue
				| FileKind::LocalizationTable
				| FileKind::Dir => {
					let class = self.get_class(&file_kind, Some(path), None)?;
					let properties = self.get_properties(&file_kind, path)?;
					let name = self.get_name(&file_kind, &rbx_path, file_name);
					let parent = self.get_parent(&rbx_path);

					let instance = Instance::new(&name)
						.with_class(&class)
						.with_properties(properties.clone())
						.with_rbx_path(&rbx_path)
						.with_path(path);

					dom.insert(instance, &parent);

					if !dom_only {
						queue.push(
							Message::Create(Create {
								class,
								path: rbx_path,
								properties,
							}),
							None,
						);
					}
				}
				FileKind::InstanceData => {
					let instance = dom.get_mut(&rbx_path).unwrap();
					let properties = self.get_properties(&file_kind, path)?;

					instance.properties = properties.clone();

					if !dom_only {
						queue.push(
							Message::Update(Update {
								path: rbx_path,
								properties,
							}),
							None,
						);
					}
				}
				FileKind::Model(ref kind) => {
					let reader = BufReader::new(File::open(path)?);
					let mut model = if *kind == ModelKind::Binary {
						rbx_binary::from_reader(reader)?
					} else {
						rbx_xml::from_reader(reader, DecodeOptions::default())?
					};

					model.root_mut().name = file_name.to_owned();

					let new_instances = dom.append(&mut model, &rbx_path, path);

					if !dom_only {
						for (path, instances) in new_instances {
							for instance in instances {
								queue.push(
									Message::Create(Create {
										class: instance.class.clone(),
										path: path.to_owned(),
										properties: instance.properties.clone(),
									}),
									None,
								);
							}
						}
					}
				}
			}
		}

		drop(dom);
		drop(queue);

		if path.is_dir() {
			for entry in fs::read_dir(path)? {
				self.create(&entry?.path(), dom_only)?;
			}
		}

		Ok(())
	}

	pub fn delete(&self, path: &Path) -> Result<()> {
		let ext = util::get_file_ext(path);

		if !self.is_valid(path, ext, true) {
			return Ok(());
		}

		let file_name = util::get_file_stem(path);
		let rbx_paths = self.get_rbx_paths(path, file_name, ext)?;

		let mut dom = lock!(self.dom);
		let mut queue = lock!(self.queue);

		for rbx_path in rbx_paths {
			if file_name == self.config.data() && ext == "json" {
				let instance = dom.get_mut(&rbx_path).unwrap();
				instance.properties = HashMap::new();

				queue.push(
					Message::Update(Update {
						path: rbx_path.clone(),
						properties: HashMap::new(),
					}),
					None,
				);
			} else if dom.remove(&rbx_path) {
				queue.push(Message::Delete(Delete { path: rbx_path }), None);
			}
		}

		Ok(())
	}

	pub fn write(&self, path: &Path) -> Result<()> {
		let ext = util::get_file_ext(path);

		if !self.is_valid(path, ext, false) {
			return Ok(());
		}

		let file_name = util::get_file_stem(path);
		let rbx_paths = self.get_rbx_paths(path, file_name, ext)?;
		let file_kind = self.get_file_kind(file_name, ext, false)?;

		let mut dom = lock!(self.dom);
		let mut queue = lock!(self.queue);

		for rbx_path in rbx_paths {
			match file_kind {
				FileKind::Model(ref kind) => {
					if dom.remove(&rbx_path) {
						queue.push(Message::Delete(Delete { path: rbx_path.clone() }), None);

						let reader = BufReader::new(File::open(path)?);
						let mut model = if *kind == ModelKind::Binary {
							rbx_binary::from_reader(reader)?
						} else {
							rbx_xml::from_reader(reader, DecodeOptions::default())?
						};

						model.root_mut().name = file_name.to_owned();

						let new_instances = dom.append(&mut model, &rbx_path, path);

						for (path, instances) in new_instances {
							for instance in instances {
								queue.push(
									Message::Create(Create {
										class: instance.class.clone(),
										path: path.to_owned(),
										properties: instance.properties.clone(),
									}),
									None,
								);
							}
						}
					}
				}
				_ => {
					let instance = dom.get_mut(&rbx_path).unwrap();
					let properties = self.get_properties(&file_kind, path)?;

					instance.properties = properties.clone();

					queue.push(
						Message::Update(Update {
							path: rbx_path,
							properties,
						}),
						None,
					);
				}
			}
		}

		Ok(())
	}
}
