use anyhow::Result;
use indexmap::IndexMap;
use multimap::MultiMap;
use rbx_dom_weak::types::Variant;
use serde::{Deserialize, Serialize};
use std::{
	collections::{BTreeMap, HashMap},
	fs, mem,
	path::{Path, PathBuf},
};
use walkdir::WalkDir;

use crate::{argon_warn, glob::Glob, rbx_path::RbxPath, resolution::UnresolvedValue, util, workspace};

#[derive(Debug)]
pub struct ProjectChanges {
	pub address: bool,
	pub paths: bool,
	pub data: bool,
	pub meta: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ProjectNode {
	#[serde(rename = "$className")]
	pub class_name: Option<String>,
	#[serde(rename = "$path")]
	pub path: Option<PathBuf>,
	#[serde(flatten)]
	pub tree: BTreeMap<String, ProjectNode>,

	#[serde(rename = "$properties")]
	pub properties: Option<HashMap<String, UnresolvedValue>>,
	#[serde(rename = "$attributes")]
	pub attributes: Option<HashMap<String, UnresolvedValue>>,
	#[serde(rename = "$tags")]
	pub tags: Option<Vec<String>>,

	// This field is not actually used by Argon
	#[serde(rename = "$ignoreUnknownInstances", skip_serializing)]
	pub ignore_unknown_instances: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Project {
	pub name: String,
	#[serde(rename = "tree")]
	pub node: ProjectNode,
	#[serde(alias = "serveAddress")]
	pub host: Option<String>,
	#[serde(alias = "servePort")]
	pub port: Option<u16>,
	pub game_id: Option<u64>,
	#[serde(alias = "servePlaceIds")]
	pub place_ids: Option<Vec<u64>>,
	#[serde(alias = "globIgnorePaths")]
	pub ignore_globs: Option<Vec<Glob>>,

	#[serde(skip)]
	pub root_class: String,
	#[serde(skip)]
	pub root_dir: Option<PathBuf>,
	#[serde(skip)]
	pub project_path: PathBuf,
	#[serde(skip)]
	pub workspace_dir: PathBuf,

	#[serde(skip)]
	pub path_map: MultiMap<PathBuf, RbxPath>,
	#[serde(skip)]
	pub data_map: IndexMap<RbxPath, HashMap<String, Variant>>,
}

impl Project {
	pub fn load(project_path: &Path) -> Result<Self> {
		let project = fs::read_to_string(project_path)?;
		let mut project: Project = serde_json::from_str(&project)?;

		let workspace_dir = workspace::get_dir(project_path);

		project.root_class = project.node.class_name.clone().unwrap_or(String::from("Folder"));
		project.project_path = project_path.to_owned();
		project.workspace_dir = workspace_dir.to_owned();

		if let Some(path) = project.node.path.clone() {
			let workspace_dir = project.workspace_dir.clone();
			let mut tree = BTreeMap::new();
			tree.insert(project.name.clone(), project.node.clone());

			project.parse_tree(&tree, &workspace_dir, &RbxPath::new())?;

			let path = util::resolve_path(path)?;
			project.root_dir = Some(path);
		} else {
			let workspace_dir = project.workspace_dir.clone();
			let tree = project.node.tree.clone();

			project.parse_tree(&tree, &workspace_dir, &RbxPath::from(&project.name))?;
		}

		Ok(project)
	}

	pub fn reload(&mut self) -> Result<ProjectChanges> {
		let new = Self::load(&self.project_path)?;

		let changes = ProjectChanges {
			address: self.host != new.host || self.port != new.port,
			paths: self.path_map != new.path_map,
			data: self.data_map != new.data_map,
			meta: self.name != new.name || self.game_id != new.game_id || self.place_ids != new.place_ids,
		};

		drop(mem::replace(self, new));

		Ok(changes)
	}

	pub fn get_paths(&self) -> Vec<PathBuf> {
		self.path_map.keys().cloned().collect()
	}

	pub fn is_place(&self) -> bool {
		self.root_class == "DataModel"
	}

	pub fn is_ts(&self) -> bool {
		if let Some(ignore_globs) = &self.ignore_globs {
			for glob in ignore_globs {
				if glob.matches("**/tsconfig.json") {
					return true;
				}
			}
		}

		for path in self.path_map.keys() {
			if path.ends_with("@rbxts") {
				return true;
			}
		}

		false
	}

	pub fn is_rojo(&self) -> bool {
		if util::get_file_name(&self.project_path) == ".argon" {
			return false;
		}

		for path in self.path_map.keys() {
			for entry in WalkDir::new(path).into_iter().filter_map(|e| e.ok()) {
				let stem = util::get_file_stem(entry.path());
				let ext = util::get_file_ext(entry.path());

				if ext == "lua" || ext == "luau" {
					if stem.starts_with(".src") {
						return false;
					} else if stem.starts_with("init") {
						return true;
					}
				} else if ext == "json" {
					if stem == ".data" {
						return false;
					} else if stem == "meta" {
						return true;
					}
				}
			}
		}

		false
	}

	fn parse_tree(
		&mut self,
		tree: &BTreeMap<String, ProjectNode>,
		local_root: &PathBuf,
		rbx_root: &RbxPath,
	) -> Result<()> {
		for (name, node) in tree {
			let rbx_path = rbx_root.join(name);

			if let Some(path) = &node.path {
				if util::get_file_name(path).ends_with("project.json") {
					let project = Self::load(&local_root.join(path))?;

					if project.is_place() {
						argon_warn!("Cannot append place project, only model-like projects are supported!");
						continue;
					}

					let mut tree = BTreeMap::new();
					tree.insert(project.name, project.node);

					self.parse_tree(&tree, local_root, &rbx_path)?;

					continue;
				}

				let mut local_path = path.clone();

				if !local_path.is_absolute() {
					local_path = local_root.join(local_path);
				}

				self.path_map.insert(local_path, rbx_path.clone());
			}

			if node.class_name.is_some()
				|| node.properties.is_some()
				|| node.attributes.is_some()
				|| node.tags.is_some()
			{
				let properties = util::properties::from_node(node.clone(), name)?;
				self.data_map.insert(rbx_path.clone(), properties);
			}

			self.parse_tree(&node.tree, local_root, &rbx_path)?;
		}

		Ok(())
	}
}

pub fn resolve(path: PathBuf) -> Result<PathBuf> {
	let mut project_path = util::resolve_path(path)?;

	if project_path.is_file() || util::get_file_name(&project_path).ends_with(".project.json") {
		return Ok(project_path);
	}

	let glob = project_path.clone().join("*.project.json");

	if let Some(path) = Glob::new(glob.to_str().unwrap())?.first() {
		project_path = path;
	} else {
		project_path = project_path.join(".argon.project.json");
	}

	Ok(project_path)
}
