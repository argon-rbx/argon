use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::{
	collections::BTreeMap,
	fs,
	path::{PathBuf, MAIN_SEPARATOR},
};

use crate::{utils, workspace};

#[derive(Serialize, Deserialize, Debug)]
pub struct Project {
	pub name: String,
	#[serde(rename = "tree")]
	pub node: ProjectNode,
	pub host: Option<String>,
	pub port: Option<u16>,
	pub place_ids: Option<Vec<u64>>,
	pub ignore_paths: Option<Vec<String>>,

	#[serde(skip)]
	pub project: PathBuf,

	#[serde(skip)]
	pub workspace: PathBuf,
}

impl Project {
	pub fn load(project_path: &PathBuf) -> Result<Self> {
		let project = fs::read_to_string(project_path)?;
		let mut project: Project = serde_json::from_str(&project)?;

		let workspace_dir = workspace::get_dir(project_path.to_owned());

		project.project = project_path.to_owned();
		project.workspace = workspace_dir;

		Ok(project)
	}

	pub fn get_sync_paths(&self) -> Vec<PathBuf> {
		fn get_paths(tree: &BTreeMap<String, ProjectNode>, root: &PathBuf) -> Vec<PathBuf> {
			let mut paths: Vec<PathBuf> = vec![];

			for node in tree.values() {
				if let Some(path) = &node.path {
					let mut path = path.clone();

					if !path.is_absolute() {
						path = root.join(path);
					}

					paths.push(path);
				}

				paths.append(&mut get_paths(&node.tree, root));
			}

			paths
		}

		// TODO: Utilize `class_name` to create `from` field for
		// Redirect object, later used for two-way sync

		get_paths(&self.node.tree, &self.workspace)
	}
}

// struct Redirect {
// 	from: PathBuf,
// 	to: PathBuf,
// }

#[derive(Serialize, Deserialize, Debug)]
pub struct ProjectNode {
	#[serde(rename = "$className")]
	pub class_name: Option<String>,

	#[serde(rename = "$path")]
	pub path: Option<PathBuf>,

	#[serde(flatten)]
	pub tree: BTreeMap<String, ProjectNode>,
}

pub fn resolve(mut project: String, default: String) -> Result<PathBuf> {
	if project.ends_with(MAIN_SEPARATOR) {
		let mut project_glob = project.clone();
		project_glob.push_str("*.project.json");

		let mut project_path = PathBuf::from(project);
		let mut found_project = false;

		if let Some(path) = (glob::glob(&project_glob)?).next() {
			project_path = path?;
			found_project = true;
		}

		if !found_project {
			let mut default_project = default;
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
