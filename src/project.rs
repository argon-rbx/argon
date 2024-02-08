use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::{
	collections::{BTreeMap, HashMap},
	fs,
	path::{Path, PathBuf},
};

use crate::{core::meta::SyncRule, glob::Glob, resolution::UnresolvedValue, util::PathExt, workspace};

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
	pub attributes: Option<UnresolvedValue>,
	// For consistency
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
	pub sync_rules: Option<Vec<SyncRule>>,

	#[serde(skip)]
	pub project_path: PathBuf,
	#[serde(skip)]
	pub workspace_dir: PathBuf,
}

impl Project {
	pub fn load(project_path: &Path) -> Result<Self> {
		let project = fs::read_to_string(project_path)?;
		let mut project: Project = serde_json::from_str(&project)?;

		let workspace_dir = workspace::get_dir(project_path);

		project.project_path = project_path.to_owned();
		project.workspace_dir = workspace_dir.to_owned();

		Ok(project)
	}

	pub fn is_place(&self) -> bool {
		if let Some(class) = &self.node.class_name {
			class == "DataModel"
		} else {
			false
		}
	}

	pub fn is_ts(&self) -> bool {
		if let Some(ignore_globs) = &self.ignore_globs {
			for glob in ignore_globs {
				if glob.matches("**/tsconfig.json") {
					return true;
				}

				if glob.matches("**/package.json") {
					return true;
				}
			}
		}

		fn walk(node: &ProjectNode) -> bool {
			if let Some(path) = &node.path {
				if path.ends_with("@rbxts") {
					return true;
				}
			}

			for node in node.tree.values() {
				if walk(node) {
					return true;
				}
			}

			false
		}

		walk(&self.node)
	}
}

pub fn resolve(path: PathBuf) -> Result<PathBuf> {
	let mut project_path = path.resolve()?;

	if project_path.is_file() || project_path.get_file_name().ends_with(".project.json") {
		return Ok(project_path);
	}

	let glob = project_path.clone().join("*.project.json");

	if let Some(path) = Glob::from_path(&glob)?.first() {
		project_path = path;
	} else {
		project_path = project_path.join(".argon.project.json");
	}

	Ok(project_path)
}
