use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::{
	collections::BTreeMap,
	env, fs,
	path::{PathBuf, MAIN_SEPARATOR},
};

use crate::utils;

#[derive(Serialize, Deserialize, Debug)]
pub struct Project {
	pub name: String,
	pub tree: ProjectTree,
	pub host: Option<String>,
	pub port: Option<u16>,
	pub place_ids: Option<Vec<u64>>,
	pub ignore_paths: Option<Vec<String>>,
}

impl Project {
	pub fn load(project_path: PathBuf) -> Result<Project> {
		let project = fs::read_to_string(project_path)?;
		let project: Project = serde_json::from_str(&project)?;

		Ok(project)
	}

	pub fn get_sync_paths(&self) -> Vec<PathBuf> {
		vec![]
	}
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ProjectTree {
	#[serde(rename = "$className")]
	pub class_name: Option<String>,

	#[serde(rename = "$path")]
	pub path: Option<PathBuf>,

	#[serde(flatten)]
	pub node: BTreeMap<String, ProjectTree>,
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

	let mut project_path = PathBuf::from(project);

	if !project_path.is_absolute() {
		let current_dir = env::current_dir()?;
		project_path = current_dir.join(project_path);
	}

	Ok(project_path)
}
