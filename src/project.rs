use anyhow::Result;
use multimap::MultiMap;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{
	collections::{BTreeMap, HashMap},
	fs,
	path::PathBuf,
};

use crate::{glob::Glob, rbx_path::RbxPath, util, workspace};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ProjectNode {
	#[serde(rename = "$className")]
	pub class_name: Option<String>,

	#[serde(rename = "$path")]
	pub path: Option<PathBuf>,

	#[serde(flatten)]
	pub tree: BTreeMap<String, ProjectNode>,

	#[serde(rename = "$properties")]
	pub properties: Option<HashMap<String, Value>>,
	#[serde(rename = "$attributes")]
	pub attributes: Option<HashMap<String, Value>>,
	#[serde(rename = "$ignoreUnknownInstances")]
	pub ignore_unknown_instances: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Project {
	pub name: String,
	#[serde(rename = "tree")]
	pub node: ProjectNode,
	#[serde(alias = "serveAddress")]
	pub host: Option<String>,
	#[serde(alias = "servePort")]
	pub port: Option<u16>,
	#[serde(rename = "gameId")]
	pub game_id: Option<u64>,
	#[serde(rename = "placeIds", alias = "servePlaceIds")]
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
}

impl Project {
	pub fn load(project_path: &PathBuf) -> Result<Self> {
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

			project.parse_paths(&tree, &workspace_dir, &RbxPath::new());

			let path = util::resolve_path(path)?;
			project.root_dir = Some(path);
		} else {
			let workspace_dir = project.workspace_dir.clone();
			let tree = project.node.tree.clone();

			project.parse_paths(&tree, &workspace_dir, &RbxPath::from(&project.name));
		}

		Ok(project)
	}

	fn parse_paths(&mut self, tree: &BTreeMap<String, ProjectNode>, local_root: &PathBuf, rbx_root: &RbxPath) {
		for (name, node) in tree.iter() {
			let mut rbx_path = rbx_root.clone();
			rbx_path.push(name);

			if let Some(path) = &node.path {
				let mut local_path = path.clone();

				if !local_path.is_absolute() {
					local_path = local_root.join(local_path);
				}

				self.path_map.insert(local_path, rbx_path.clone());
			}

			self.parse_paths(&node.tree, local_root, &rbx_path);
		}
	}

	pub fn get_paths(&self) -> Vec<PathBuf> {
		self.path_map.keys().cloned().collect()
	}

	pub fn is_place(&self) -> bool {
		self.root_class == "DataModel"
	}
}

pub fn resolve(path: PathBuf, default: &str) -> Result<PathBuf> {
	let mut project_path = util::resolve_path(path)?;

	if project_path.is_file() || util::get_file_name(&project_path).ends_with(".project.json") {
		return Ok(project_path);
	}

	let glob = project_path.clone().join("*.project.json");

	if let Some(path) = Glob::new(glob.to_str().unwrap())?.first() {
		project_path = path;
	} else {
		project_path = project_path.join(format!("{}.project.json", default));
	}

	Ok(project_path)
}
