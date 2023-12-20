use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::{
	collections::BTreeMap,
	fs,
	path::{PathBuf, MAIN_SEPARATOR},
};

use crate::{glob::Glob, types::RbxPath, utils, workspace};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ProjectNode {
	#[serde(rename = "$className")]
	pub class_name: Option<String>,

	#[serde(rename = "$path")]
	pub path: Option<PathBuf>,

	#[serde(flatten)]
	pub tree: BTreeMap<String, ProjectNode>,
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
	pub local_paths: Vec<PathBuf>,
	#[serde(skip)]
	pub rbx_paths: Vec<RbxPath>,
}

impl Project {
	pub fn load(project_path: &PathBuf) -> Result<Self> {
		let project = fs::read_to_string(project_path)?;
		let mut project: Project = serde_json::from_str(&project)?;

		let workspace_dir = workspace::get_dir(project_path.to_owned());

		project.root_class = project.node.class_name.clone().unwrap_or(String::from("Folder"));
		project.project_path = project_path.to_owned();
		project.workspace_dir = workspace_dir;

		if let Some(path) = &project.node.path {
			let mut tree = BTreeMap::new();
			tree.insert(project.name.clone(), project.node.clone());

			(project.local_paths, project.rbx_paths) =
				project.get_paths(&tree, &project.workspace_dir, &RbxPath::new());

			let path = utils::resolve_path(path.to_owned())?;
			project.root_dir = Some(path);
		} else {
			(project.local_paths, project.rbx_paths) = project.get_paths(
				&project.node.tree,
				&project.workspace_dir,
				&RbxPath::from(&project.name),
			);
		}

		Ok(project)
	}

	#[allow(clippy::only_used_in_recursion)]
	fn get_paths(
		&self,
		tree: &BTreeMap<String, ProjectNode>,
		local_root: &PathBuf,
		rbx_root: &RbxPath,
	) -> (Vec<PathBuf>, Vec<RbxPath>) {
		let mut local_paths = vec![];
		let mut rbx_paths = vec![];

		for (name, node) in tree.iter() {
			let mut rbx_path = rbx_root.clone();
			rbx_path.push(name);

			if let Some(path) = &node.path {
				let mut local_path = path.clone();

				if !local_path.is_absolute() {
					local_path = local_root.join(local_path);
				}

				local_paths.push(local_path);
				rbx_paths.push(rbx_path.clone());
			}

			let mut paths = self.get_paths(&node.tree, local_root, &rbx_path);

			local_paths.append(&mut paths.0);
			rbx_paths.append(&mut paths.1);
		}

		(local_paths, rbx_paths)
	}
}

pub fn resolve(mut project: String, default: &str) -> Result<PathBuf> {
	if project.ends_with(MAIN_SEPARATOR) {
		let mut project_path = PathBuf::from(project.clone());

		project.push_str("*.project.json");

		if let Some(path) = Glob::new(&project)?.first() {
			project_path = path;
		} else {
			let mut default_project = default.to_owned();
			default_project.push_str(".project.json");

			project_path = project_path.join(default_project);
		}

		project_path = utils::resolve_path(project_path)?;

		return Ok(project_path);
	}

	if !project.ends_with(".project.json") {
		project.push_str(".project.json")
	}

	utils::resolve_path(PathBuf::from(project))
}
