use anyhow::Result;
use rbx_dom_weak::types::Ref;
use serde::{Deserialize, Serialize};
use std::{
	collections::{BTreeMap, HashMap},
	fs, mem,
	path::{Path, PathBuf},
};

use crate::{
	core::{
		meta::{NodePath, SyncRule},
		tree::Tree,
	},
	ext::PathExt,
	glob::Glob,
	resolution::UnresolvedValue,
	workspace,
};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct ProjectNode {
	#[serde(rename = "$className", skip_serializing_if = "Option::is_none")]
	pub class_name: Option<String>,
	#[serde(rename = "$path", skip_serializing_if = "Option::is_none")]
	pub path: Option<PathBuf>,
	#[serde(flatten)]
	pub tree: BTreeMap<String, ProjectNode>,

	#[serde(rename = "$properties", default, skip_serializing_if = "HashMap::is_empty")]
	pub properties: HashMap<String, UnresolvedValue>,
	#[serde(rename = "$attributes", skip_serializing_if = "Option::is_none")]
	pub attributes: Option<UnresolvedValue>,
	#[serde(rename = "$tags", default, skip_serializing_if = "Vec::is_empty")]
	pub tags: Vec<String>,

	#[serde(
		rename = "$keepUnknowns",
		alias = "$ignoreUnknownInstances",
		default,
		skip_serializing_if = "Option::is_none"
	)]
	pub keep_unknowns: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Project {
	pub name: String,
	#[serde(rename = "tree")]
	pub node: ProjectNode,

	#[serde(alias = "serveAddress", skip_serializing_if = "Option::is_none")]
	pub host: Option<String>,
	#[serde(alias = "servePort", skip_serializing_if = "Option::is_none")]
	pub port: Option<u16>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub game_id: Option<u64>,
	#[serde(alias = "servePlaceIds", default, skip_serializing_if = "Vec::is_empty")]
	pub place_ids: Vec<u64>,

	#[serde(alias = "globIgnorePaths", default, skip_serializing_if = "Vec::is_empty")]
	pub ignore_globs: Vec<Glob>,
	#[serde(default, skip_serializing_if = "Vec::is_empty")]
	pub sync_rules: Vec<SyncRule>,

	#[serde(alias = "emitLegacyScripts", skip_serializing_if = "Option::is_none")]
	pub legacy_scripts: Option<bool>,

	#[serde(skip)]
	pub path: PathBuf,
	#[serde(skip)]
	pub workspace_dir: PathBuf,
}

impl Project {
	pub fn load(project_path: &Path) -> Result<Self> {
		let project = fs::read_to_string(project_path)?;
		let mut project: Project = serde_json::from_str(&project)?;

		let workspace_dir = workspace::get_dir(project_path);

		project.path = project_path.to_owned();
		project.workspace_dir = workspace_dir.to_owned();

		Ok(project)
	}

	pub fn save(&self, path: &Path) -> Result<()> {
		let mut project = serde_json::to_string_pretty(self)?;
		project.push('\n');

		fs::write(path, project)?;

		Ok(())
	}

	pub fn reload(&mut self) -> Result<&Self> {
		let new = Self::load(&self.path)?;

		drop(mem::replace(self, new));

		Ok(self)
	}

	pub fn is_place(&self) -> bool {
		if let Some(class) = &self.node.class_name {
			class == "DataModel"
		} else {
			false
		}
	}

	pub fn is_ts(&self) -> bool {
		for glob in &self.ignore_globs {
			if glob.matches("**/tsconfig.json") {
				return true;
			}

			if glob.matches("**/package.json") {
				return true;
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
	let path = path_clean::clean(path.resolve()?);

	if path.is_file() || path.get_name().ends_with(".project.json") {
		return Ok(path);
	}

	let default_project = path.join("default.project.json");
	if default_project.exists() {
		return Ok(default_project);
	}

	let glob = path.clone().join("*.project.json");

	if let Some(path) = Glob::from_path(&glob)?.first() {
		Ok(path)
	} else {
		Ok(path.join("default.project.json"))
	}
}

pub fn find_node_by_path<'a>(project: &'a mut Project, node_path: &NodePath) -> Option<&'a mut ProjectNode> {
	let mut node = &mut project.node;

	for name in node_path.iter() {
		node = node.tree.get_mut(name)?;
	}

	Some(node)
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectDetails {
	version: String,
	name: String,
	game_id: Option<u64>,
	place_ids: Vec<u64>,
	root_dirs: Vec<Ref>,
}

impl ProjectDetails {
	pub fn from_project(project: &Project, tree: &Tree) -> Self {
		Self {
			version: env!("CARGO_PKG_VERSION").to_owned(),

			name: project.name.clone(),
			game_id: project.game_id,
			place_ids: project.place_ids.clone(),

			root_dirs: if project.is_place() {
				tree.place_root_refs().to_owned()
			} else {
				vec![tree.root_ref()]
			},
		}
	}
}
