use anyhow::{bail, Result};
use pathsub::sub_paths;
use rbx_dom_weak::types::{Enum, Variant};
use serde_json::Value;
use std::{
	collections::HashMap,
	fs::{self, File},
	io::BufReader,
	path::{Path, PathBuf},
};

use super::processor::Processor;
use crate::{lock, rbx_path::RbxPath, util};

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

impl Processor {
	pub fn is_valid(&self, path: &Path, ext: &str, is_dir: bool) -> bool {
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

	pub fn is_child_script(&self, file_stem: &str) -> bool {
		matches!(
			file_stem,
			".src" | ".src.server" | ".src.client" | "init" | "init.server" | "init.client"
		)
	}

	pub fn is_instance_data(&self, file_stem: &str) -> bool {
		matches!(file_stem, ".data" | "meta")
	}

	pub fn get_data_file(&self, path: &Path) -> Option<PathBuf> {
		let data_path = path.join(".data.json");
		if data_path.exists() {
			return Some(data_path);
		}

		let data_path = path.join("meta.json");
		if data_path.exists() {
			return Some(data_path);
		}

		None
	}

	pub fn get_rbx_paths(&self, path: &Path, file_stem: &str, ext: &str) -> Result<Vec<RbxPath>> {
		let project = lock!(self.project);

		for local_path in project.get_paths() {
			if let Some(diff) = sub_paths(path, &local_path) {
				let mut rbx_paths = project.path_map.get_vec(&local_path).unwrap().clone();
				let parent = diff.parent().unwrap();

				for rbx_path in rbx_paths.iter_mut() {
					for comp in parent {
						let comp = util::from_os_str(comp);
						rbx_path.push(comp);
					}

					match ext {
						"lua" | "luau" => {
							if !self.is_child_script(file_stem) {
								let name = if file_stem.ends_with(".server") || file_stem.ends_with(".client") {
									&file_stem[..file_stem.len() - 7]
								} else {
									file_stem
								};

								rbx_path.push(name);
							}
						}
						"json" => {
							if !self.is_instance_data(file_stem) {
								rbx_path.push(file_stem);
							}
						}
						_ => {
							rbx_path.push(file_stem);
						}
					}
				}

				return Ok(rbx_paths);
			}
		}

		bail!("{:?} does not exists in the project file", path)
	}

	pub fn get_file_kind(&self, file_stem: &str, ext: &str, is_dir: bool) -> Result<FileKind> {
		if is_dir {
			return Ok(FileKind::Dir);
		}

		if ext == "lua" || ext == "luau" {
			let kind = if file_stem.ends_with(".server") {
				ScriptKind::Server
			} else if file_stem.ends_with(".client") {
				ScriptKind::Client
			} else {
				ScriptKind::Module
			};

			if self.is_child_script(file_stem) {
				return Ok(FileKind::ChildScript(kind));
			} else {
				return Ok(FileKind::Script(kind));
			}
		} else if ext == "json" {
			if self.is_instance_data(file_stem) {
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
		// Sketchy solution to get around borrow checker

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
					if let Some(data_file) = self.get_data_file(path) {
						|| -> Result<&str> {
							let data_file = File::open(data_file)?;
							let reader = BufReader::new(data_file);
							let data: HashMap<String, Value> = serde_json::from_reader(reader)?;

							// .data.json files
							if data.contains_key("ClassName") && data["ClassName"].is_string() {
								temp = data["ClassName"].as_str().unwrap().to_owned();
								Ok(&temp)
							// meta.json files
							} else if data.contains_key("className") && data["className"].is_string() {
								temp = data["className"].as_str().unwrap().to_owned();
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

	pub fn get_name(&self, kind: &FileKind, rbx_path: &RbxPath, file_stem: &str) -> String {
		match kind {
			FileKind::Script(ref kind) => {
				if *kind != ScriptKind::Module {
					let pos = if *kind == ScriptKind::Server {
						file_stem.rfind(".server").unwrap()
					} else {
						file_stem.rfind(".client").unwrap()
					};

					file_stem[..pos].to_owned()
				} else {
					file_stem.to_owned()
				}
			}
			FileKind::ChildScript(_) => rbx_path.last().unwrap().clone(),
			_ => file_stem.to_owned(),
		}
	}

	pub fn get_parent(&self, rbx_path: &RbxPath) -> RbxPath {
		rbx_path.parent().unwrap()
	}

	pub fn get_properties(&self, kind: &FileKind, path: &Path) -> Result<HashMap<String, Variant>> {
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
			FileKind::Dir => {
				if let Some(data_file) = self.get_data_file(path) {
					properties.extend(util::properties::from_json(&data_file)?);
				}
			}
			FileKind::InstanceData => {
				properties.extend(util::properties::from_json(path)?);
			}
			_ => bail!("Cannot get properties of {:?} file kind", kind),
		}

		Ok(properties)
	}
}
