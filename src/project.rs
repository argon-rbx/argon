use anyhow::Result;
use colored::Colorize;
use rbx_dom_weak::{types::Ref, Ustr, UstrMap};
use serde::{Deserialize, Serialize};
use serde_json::Serializer;
use std::{
	collections::{BTreeMap, HashMap},
	fs, mem,
	path::{Path, PathBuf},
};

use crate::{
	config::Config,
	core::{
		meta::{NodePath, SyncRule},
		tree::Tree,
	},
	ext::{PathExt, ResultExt},
	glob::Glob,
	resolution::UnresolvedValue,
	util::get_json_formatter,
};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ProjectPath {
	Required(PathBuf),
	Optional { optional: PathBuf },
}

impl ProjectPath {
	pub fn path(&self) -> &Path {
		match self {
			ProjectPath::Required(path) => path.as_ref(),
			ProjectPath::Optional { optional } => optional.as_ref(),
		}
	}
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct ProjectNode {
	#[serde(rename = "$className", skip_serializing_if = "Option::is_none")]
	pub class_name: Option<Ustr>,
	#[serde(rename = "$path", skip_serializing_if = "Option::is_none")]
	pub path: Option<ProjectPath>,
	#[serde(flatten)]
	pub tree: BTreeMap<String, ProjectNode>,

	#[serde(rename = "$properties", default, skip_serializing_if = "HashMap::is_empty")]
	pub properties: UstrMap<UnresolvedValue>,
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
pub struct SyncbackSettings {
	#[serde(alias = "excludeGlobs", default, skip_serializing_if = "Vec::is_empty")]
	pub ignore_globs: Vec<Glob>,

	#[serde(alias = "skipInstanceNames", default, skip_serializing_if = "Vec::is_empty")]
	pub ignore_names: Vec<String>,
	#[serde(default, skip_serializing_if = "Vec::is_empty")]
	pub ignore_classes: Vec<String>,
	#[serde(default, skip_serializing_if = "Vec::is_empty")]
	pub ignore_properties: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Project {
	#[serde(default = "default_project_name")]
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

	#[serde(skip_serializing_if = "Option::is_none")]
	pub syncback: Option<SyncbackSettings>,

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
		let mut project: Project = serde_json::from_str(&project).with_desc(|| {
			format!(
				"Failed to parse project at {}",
				project_path.display().to_string().bold()
			)
		})?;

		let workspace_dir = project_path.get_parent();

		project_path.clone_into(&mut project.path);
		workspace_dir.clone_into(&mut project.workspace_dir);

		Ok(project)
	}

	pub fn save(&self, path: &Path) -> Result<()> {
		let mut writer = Vec::new();
		let mut serializer = Serializer::with_formatter(&mut writer, get_json_formatter());

		self.serialize(&mut serializer)?;
		fs::write(path, &writer)?;

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
			if node.path.as_ref().is_some_and(|p| p.path().ends_with("@rbxts")) {
				return true;
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

	pub fn is_wally(&self) -> bool {
		fn walk(node: &ProjectNode) -> bool {
			if node.path.as_ref().is_some_and(|p| p.path() == Path::new("Packages")) {
				return true;
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

	pub fn find_node_by_path(&mut self, node_path: &NodePath) -> Option<&mut ProjectNode> {
		let mut node = &mut self.node;

		for name in node_path.iter() {
			node = node.tree.get_mut(name)?;
		}

		Some(node)
	}
}

pub fn resolve(path: PathBuf) -> Result<PathBuf> {
	let path = path.resolve()?;

	if path.is_file() || path.get_name().ends_with(".project.json") {
		return Ok(path);
	}

	if Config::new().smart_paths {
		let path = path.with_file_name(path.get_name().to_owned() + ".project.json");

		if path.exists() {
			return Ok(path);
		}
	}

	let default_project = path.join("default.project.json");

	if default_project.exists() {
		return Ok(default_project);
	}

	let glob = path.clone().join("*.project.json");

	if let Some(path) = Glob::from_path(&glob)?.first() {
		Ok(path)
	} else {
		Ok(default_project)
	}
}

#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ProjectDetails {
	version: String,
	name: String,
	game_id: Option<u64>,
	place_ids: Vec<u64>,
	root_refs: Vec<Ref>,
}

impl ProjectDetails {
	pub fn from_project(project: &Project, tree: &Tree) -> Self {
		Self {
			version: env!("CARGO_PKG_VERSION").to_owned(),

			name: project.name.clone(),
			game_id: project.game_id,
			place_ids: project.place_ids.clone(),

			root_refs: if project.is_place() {
				tree.place_root_refs().to_owned()
			} else {
				vec![tree.root_ref()]
			},
		}
	}
}

fn default_project_name() -> String {
	String::from("default")
}
