use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::{
	collections::BTreeMap,
	fs,
	path::{PathBuf, MAIN_SEPARATOR},
};

use crate::{glob::Glob, utils, workspace, ROBLOX_SEPARATOR};

#[derive(Serialize, Deserialize, Debug)]
pub struct ProjectNode {
	#[serde(rename = "$className")]
	pub class_name: Option<String>,

	#[serde(rename = "$path")]
	pub path: Option<PathBuf>,

	#[serde(flatten)]
	pub tree: BTreeMap<String, ProjectNode>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Project {
	pub name: String,
	#[serde(rename = "tree")]
	pub node: ProjectNode,
	pub host: Option<String>,
	pub port: Option<u16>,
	pub game_id: Option<i64>,
	pub place_ids: Option<Vec<u64>>,
	pub ignore_globs: Option<Vec<Glob>>,

	#[serde(skip)]
	pub project_path: PathBuf,
	#[serde(skip)]
	pub workspace_dir: PathBuf,
	#[serde(skip)]
	pub local_paths: Vec<PathBuf>,
	#[serde(skip)]
	pub roblox_paths: Vec<String>,
}

impl Project {
	pub fn load(project_path: &PathBuf) -> Result<Self> {
		let project = fs::read_to_string(project_path)?;
		let mut project: Project = serde_json::from_str(&project)?;

		let workspace_dir = workspace::get_dir(project_path.to_owned());

		project.project_path = project_path.to_owned();
		project.workspace_dir = workspace_dir;

		(project.local_paths, project.roblox_paths) =
			project.get_paths(&project.node.tree, &project.workspace_dir, String::new());

		Ok(project)
	}

	#[allow(clippy::only_used_in_recursion)]
	fn get_paths(
		&self,
		tree: &BTreeMap<String, ProjectNode>,
		local_root: &PathBuf,
		roblox_root: String,
	) -> (Vec<PathBuf>, Vec<String>) {
		let mut local_paths = vec![];
		let mut roblox_paths = vec![];

		for (name, node) in tree.iter() {
			let mut roblox_path = roblox_root.clone();

			if !roblox_path.is_empty() {
				roblox_path.push(ROBLOX_SEPARATOR);
			}

			roblox_path.push_str(name);

			if let Some(path) = &node.path {
				let mut local_path = path.clone();

				if !local_path.is_absolute() {
					local_path = local_root.join(local_path);
				}

				local_paths.push(local_path);
				roblox_paths.push(roblox_path.clone());
			}

			let mut paths = self.get_paths(&node.tree, local_root, roblox_path);

			local_paths.append(&mut paths.0);
			roblox_paths.append(&mut paths.1);
		}

		(local_paths, roblox_paths)
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
