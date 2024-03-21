use rbx_dom_weak::types::Variant;
use serde::{Deserialize, Serialize};
use std::{
	collections::HashMap,
	fmt::{self, Debug, Formatter},
	path::{Path, PathBuf},
};

use crate::{ext::PathExt, glob::Glob, middleware::FileType, project::Project};

#[derive(Debug, Clone)]
pub struct ResolvedSyncRule {
	pub file_type: FileType,
	pub name: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct SyncRule {
	#[serde(rename = "type")]
	pub file_type: FileType,

	pub pattern: Option<Glob>,
	pub child_pattern: Option<Glob>,
	pub exclude: Option<Glob>,

	pub suffix: Option<String>,
}

impl SyncRule {
	// Creating new sync rule

	pub fn new(file_type: FileType) -> Self {
		Self {
			file_type,
			pattern: None,
			child_pattern: None,
			exclude: None,
			suffix: None,
		}
	}

	pub fn with_pattern(mut self, pattern: &str) -> Self {
		self.pattern = Some(Glob::new(pattern).unwrap());
		self
	}

	pub fn with_child_pattern(mut self, child_pattern: &str) -> Self {
		self.child_pattern = Some(Glob::new(child_pattern).unwrap());
		self
	}

	pub fn with_exclude(mut self, exclude: &str) -> Self {
		self.exclude = Some(Glob::new(exclude).unwrap());
		self
	}

	pub fn with_suffix(mut self, suffix: &str) -> Self {
		self.suffix = Some(suffix.to_owned());
		self
	}

	// Matching and resolving

	pub fn is_excluded(&self, path: &Path) -> bool {
		self.exclude.iter().any(|exclude| exclude.matches_path(path))
	}

	pub fn get_name(&self, path: &Path) -> String {
		if let Some(suffix) = &self.suffix {
			let name = path.get_file_name();
			name.strip_suffix(suffix).unwrap_or(name).to_owned()
		} else {
			path.get_file_stem().to_owned()
		}
	}

	pub fn resolve(&self, path: &Path) -> Option<ResolvedSyncRule> {
		if let Some(pattern) = &self.pattern {
			if pattern.matches_path(path) && !self.is_excluded(path) && self.file_type != FileType::InstanceData {
				return Some(ResolvedSyncRule {
					file_type: self.file_type.clone(),
					name: self.get_name(path),
				});
			}
		}

		None
	}

	pub fn resolve_child(&self, path: &Path) -> Option<ResolvedSyncRule> {
		if let Some(child_pattern) = &self.child_pattern {
			let stripped_path = path.strip_prefix(path.get_parent()).unwrap();

			if child_pattern.matches_path(stripped_path)
				&& !self.is_excluded(path)
				&& self.file_type != FileType::InstanceData
			{
				return Some(ResolvedSyncRule {
					file_type: self.file_type.clone(),
					name: path.get_parent().get_file_name().to_owned(),
				});
			}
		}

		None
	}
}

#[derive(Debug, Clone, PartialEq)]
pub struct IgnoreRule {
	pattern: Glob,
	path: PathBuf,
}

impl IgnoreRule {
	pub fn matches(&self, path: &Path) -> bool {
		match path.strip_prefix(&self.path) {
			Ok(suffix) => self.pattern.matches_path(suffix),
			Err(_) => false,
		}
	}
}

#[derive(Debug, Clone, PartialEq)]
pub struct ProjectData {
	pub affects: PathBuf,
	pub source: PathBuf,
	pub name: String,
	pub class: Option<String>,
	pub properties: Option<HashMap<String, Variant>>,
}

impl ProjectData {
	pub fn new(name: &str, affects: &Path, source: &Path) -> Self {
		Self {
			affects: affects.to_owned(),
			source: source.to_owned(),

			name: name.to_owned(),
			class: None,
			properties: None,
		}
	}

	pub fn set_class(&mut self, class: String) {
		self.class = Some(class);
	}

	pub fn set_properties(&mut self, properties: HashMap<String, Variant>) {
		self.properties = Some(properties);
	}
}

#[derive(Clone)]
pub struct Meta {
	/// Rules that define how files are synced
	pub sync_rules: Vec<SyncRule>,
	/// Rules that define which files are ignored
	pub ignore_rules: Vec<IgnoreRule>,
	/// Project data that is included in the project node in `*.project.json`
	pub project_data: Option<ProjectData>,
}

impl PartialEq for Meta {
	fn eq(&self, other: &Self) -> bool {
		self.sync_rules == other.sync_rules
			&& self.ignore_rules == other.ignore_rules
			&& self.project_data == other.project_data
	}
}

impl Meta {
	// Creating new meta

	pub fn new() -> Self {
		Self {
			sync_rules: Vec::new(),
			ignore_rules: Vec::new(),
			project_data: None,
		}
	}

	pub fn from_project(project: &Project) -> Self {
		let ignore_rules = project
			.ignore_globs
			.clone()
			.unwrap_or_default()
			.into_iter()
			.map(|glob| IgnoreRule {
				pattern: glob,
				path: project.workspace_dir.clone(),
			})
			.collect();

		// Initial project data required for node project data
		let project_data = ProjectData::new(&project.name, &project.workspace_dir, &project.path);

		Self {
			sync_rules: project.sync_rules.clone().unwrap_or_else(|| Self::default().sync_rules),
			ignore_rules,
			project_data: Some(project_data),
		}
	}

	pub fn with_sync_rules(mut self, sync_rules: Vec<SyncRule>) -> Self {
		self.sync_rules = sync_rules;
		self
	}

	pub fn with_ignore_rules(mut self, ignore_rules: Vec<IgnoreRule>) -> Self {
		self.ignore_rules = ignore_rules;
		self
	}

	pub fn with_project_data(mut self, project_data: ProjectData) -> Self {
		self.project_data = Some(project_data);
		self
	}

	// Overwriting meta fields

	pub fn set_sync_rules(&mut self, sync_rules: Vec<SyncRule>) {
		self.sync_rules = sync_rules;
	}

	pub fn set_ignore_rules(&mut self, ignore_rules: Vec<IgnoreRule>) {
		self.ignore_rules = ignore_rules;
	}

	pub fn set_project_data(&mut self, project_data: ProjectData) {
		self.project_data = Some(project_data);
	}

	// Adding to meta fields

	pub fn add_sync_rule(&mut self, sync_rule: SyncRule) {
		self.sync_rules.push(sync_rule);
	}

	pub fn add_ignore_rule(&mut self, ignore_rule: IgnoreRule) {
		self.ignore_rules.push(ignore_rule);
	}

	// Joining meta fields

	pub fn extend_sync_rules(&mut self, sync_rules: Vec<SyncRule>) {
		self.sync_rules.extend(sync_rules);
	}

	pub fn extend_ignore_rules(&mut self, ignore_rules: Vec<IgnoreRule>) {
		self.ignore_rules.extend(ignore_rules);
	}

	pub fn extend(&mut self, meta: Meta) {
		self.extend_sync_rules(meta.sync_rules);
		self.extend_ignore_rules(meta.ignore_rules);

		if meta.project_data.is_some() {
			self.project_data = meta.project_data;
		}
	}

	// Misc

	pub fn is_empty(&self) -> bool {
		// We intentionally omit `processed_paths` here
		// as it's a temporary field used only in middleware
		// so there is no need to keep it in the tree
		self.sync_rules.is_empty() && self.ignore_rules.is_empty() && self.project_data.is_none()
	}

	pub fn get_sync_rule(&self, file_type: &FileType) -> Option<&SyncRule> {
		self.sync_rules.iter().find(|rule| &rule.file_type == file_type)
	}
}

impl Debug for Meta {
	fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
		let mut debug = f.debug_struct("Meta");

		if !self.sync_rules.is_empty() {
			debug.field("sync_rules", &self.sync_rules);
		}

		if !self.ignore_rules.is_empty() {
			debug.field("ignore_rules", &self.ignore_rules);
		}

		if let Some(project_data) = &self.project_data {
			debug.field("project_data", project_data);
		}

		debug.finish()
	}
}

impl Default for Meta {
	fn default() -> Self {
		let sync_rules = vec![
			SyncRule::new(FileType::Project)
				.with_pattern("*.project.json")
				.with_child_pattern("default.project.json"),
			SyncRule::new(FileType::InstanceData)
				.with_pattern("*.data.json")
				.with_child_pattern(".data.json"),
			SyncRule::new(FileType::InstanceData) // Rojo
				.with_pattern("*.data.json")
				.with_child_pattern("init.meta.json"),
			//////////////////////////////////////////////////////////////////////////////////////////
			// Argon scripts
			SyncRule::new(FileType::ServerScript)
				.with_pattern("*.server.lua")
				.with_child_pattern(".src.server.lua")
				.with_suffix(".server.lua")
				.with_exclude("init.server.lua"),
			SyncRule::new(FileType::ClientScript)
				.with_pattern("*.client.lua")
				.with_child_pattern(".src.client.lua")
				.with_suffix(".client.lua")
				.with_exclude("init.client.lua"),
			SyncRule::new(FileType::ModuleScript)
				.with_pattern("*.lua")
				.with_child_pattern(".src.lua")
				.with_exclude("init.lua"),
			// Rojo scripts
			SyncRule::new(FileType::ServerScript)
				.with_pattern("*.server.lua")
				.with_child_pattern("init.server.lua")
				.with_suffix(".server.lua"),
			SyncRule::new(FileType::ClientScript)
				.with_pattern("*.client.lua")
				.with_child_pattern("init.client.lua")
				.with_suffix(".client.lua"),
			SyncRule::new(FileType::ModuleScript)
				.with_pattern("*.lua")
				.with_child_pattern("init.lua"),
			//////////////////////////////////////////////////////////////////////////////////////////
			// Luau variants for Argon
			SyncRule::new(FileType::ServerScript)
				.with_pattern("*.server.luau")
				.with_child_pattern(".src.server.luau")
				.with_suffix(".server.luau")
				.with_exclude("init.server.luau"),
			SyncRule::new(FileType::ClientScript)
				.with_pattern("*.client.luau")
				.with_child_pattern(".src.client.luau")
				.with_suffix(".client.luau")
				.with_exclude("init.client.luau"),
			SyncRule::new(FileType::ModuleScript)
				.with_pattern("*.luau")
				.with_child_pattern(".src.luau")
				.with_exclude("init.luau"),
			// Luau variants for Rojo
			SyncRule::new(FileType::ServerScript)
				.with_pattern("*.server.luau")
				.with_child_pattern("init.server.luau")
				.with_suffix(".server.luau"),
			SyncRule::new(FileType::ClientScript)
				.with_pattern("*.client.luau")
				.with_child_pattern("init.client.luau")
				.with_suffix(".client.luau"),
			SyncRule::new(FileType::ModuleScript)
				.with_pattern("*.luau")
				.with_child_pattern("init.luau"),
			//////////////////////////////////////////////////////////////////////////////////////////
			// Other file types, Argon only
			SyncRule::new(FileType::StringValue)
				.with_pattern("*.txt")
				.with_child_pattern(".src.txt"),
			SyncRule::new(FileType::LocalizationTable)
				.with_pattern("*.csv")
				.with_child_pattern(".src.csv"),
			SyncRule::new(FileType::JsonModule)
				.with_pattern("*.json")
				.with_child_pattern(".src.json")
				.with_exclude("*.model.json"),
			SyncRule::new(FileType::TomlModule)
				.with_pattern("*.toml")
				.with_child_pattern(".src.toml"),
			// Model files, Argon only
			SyncRule::new(FileType::JsonModel)
				.with_pattern("*.model.json")
				.with_child_pattern(".src.model.json")
				.with_suffix(".model.json"),
			SyncRule::new(FileType::RbxmModel)
				.with_pattern("*.rbxm")
				.with_child_pattern(".src.rbxm"),
			SyncRule::new(FileType::RbxmxModel)
				.with_pattern("*.rbxmx")
				.with_child_pattern(".src.rbxmx"),
		];

		Self {
			sync_rules,
			..Self::new()
		}
	}
}
