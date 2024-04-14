use serde::{Deserialize, Serialize};
use std::{
	path::{Path, PathBuf},
	sync::OnceLock,
};

use crate::{
	ext::PathExt,
	glob::Glob,
	middleware::FileType,
	project::{Project, ProjectNode},
};

#[derive(Debug, Clone, PartialEq)]
pub struct NodePath {
	inner: Vec<String>,
}

impl NodePath {
	pub fn new() -> Self {
		Self { inner: Vec::new() }
	}

	pub fn join(&self, name: &str) -> Self {
		let mut inner = self.inner.clone();
		inner.push(name.to_owned());

		Self { inner }
	}

	pub fn parent(&self) -> Self {
		let mut inner = self.inner.clone();
		inner.pop();

		Self { inner }
	}

	pub fn iter(&self) -> impl Iterator<Item = &String> {
		self.inner.iter()
	}

	pub fn is_root(&self) -> bool {
		self.inner.is_empty()
	}
}

#[derive(Debug, Clone, PartialEq)]
pub enum SourceKind {
	Path(PathBuf),
	Project(String, PathBuf, ProjectNode, NodePath),
	None,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SourceEntry {
	File(PathBuf),
	Folder(PathBuf),
	Data(PathBuf),
	Project(PathBuf),
}

impl SourceEntry {
	pub fn path(&self) -> &Path {
		match self {
			SourceEntry::File(path) => path,
			SourceEntry::Folder(path) => path,
			SourceEntry::Data(path) => path,
			SourceEntry::Project(path) => path,
		}
	}

	pub fn index(&self) -> usize {
		match self {
			SourceEntry::File(_) => 0,
			SourceEntry::Data(_) => 1,
			SourceEntry::Project(_) => 2,
			SourceEntry::Folder(_) => 3,
		}
	}
}

#[derive(Debug, Clone, PartialEq)]
pub struct Source {
	// Source used to rebuild the snapshot
	inner: SourceKind,
	// Existing paths associated with the snapshot
	relevant: Vec<SourceEntry>,
}

impl Source {
	pub fn new() -> Self {
		Self {
			inner: SourceKind::None,
			relevant: Vec::new(),
		}
	}

	pub fn file(path: &Path) -> Self {
		Self {
			inner: SourceKind::Path(path.to_owned()),
			relevant: vec![SourceEntry::File(path.to_owned())],
		}
	}

	pub fn child_file(folder: &Path, file: &Path) -> Self {
		Self {
			inner: SourceKind::Path(folder.to_owned()),
			relevant: vec![
				SourceEntry::Folder(folder.to_owned()),
				SourceEntry::File(file.to_owned()),
			],
		}
	}

	pub fn directory(path: &Path) -> Self {
		Self {
			inner: SourceKind::Path(path.to_owned()),
			relevant: vec![SourceEntry::Folder(path.to_owned())],
		}
	}

	pub fn project(name: &str, path: &Path, node: ProjectNode, node_path: NodePath) -> Self {
		Self {
			inner: SourceKind::Project(name.to_owned(), path.to_owned(), node, node_path),
			relevant: vec![SourceEntry::Project(path.to_owned())],
		}
	}

	pub fn add_relevant(&mut self, entry: SourceEntry) {
		self.relevant.push(entry)
	}

	pub fn add_data(&mut self, path: &Path) {
		self.add_relevant(SourceEntry::Data(path.to_owned()))
	}

	pub fn extend_relavants(&mut self, entries: Vec<SourceEntry>) {
		self.relevant.extend(entries)
	}

	pub fn get(&self) -> &SourceKind {
		&self.inner
	}

	pub fn get_file(&self) -> Option<&SourceEntry> {
		self.relevant.iter().find(|entry| matches!(entry, SourceEntry::File(_)))
	}

	pub fn get_folder(&self) -> Option<&SourceEntry> {
		self.relevant
			.iter()
			.find(|entry| matches!(entry, SourceEntry::Folder(_)))
	}

	pub fn get_data(&self) -> Option<&SourceEntry> {
		self.relevant.iter().find(|entry| matches!(entry, SourceEntry::Data(_)))
	}

	pub fn get_project(&self) -> Option<&SourceEntry> {
		self.relevant
			.iter()
			.find(|entry| matches!(entry, SourceEntry::Project(_)))
	}

	pub fn relevants(&self) -> &Vec<SourceEntry> {
		&self.relevant
	}

	pub fn paths(&self) -> Vec<&Path> {
		self.relevant.iter().map(|entry| entry.path()).collect()
	}
}

impl Default for Source {
	fn default() -> Self {
		Self::new()
	}
}

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
	pub exclude: Vec<Glob>,

	pub suffix: Option<String>,
}

impl SyncRule {
	// Creating new sync rule

	pub fn new(file_type: FileType) -> Self {
		Self {
			file_type,
			pattern: None,
			child_pattern: None,
			exclude: Vec::new(),
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
		self.exclude = vec![Glob::new(exclude).unwrap()];
		self
	}

	pub fn with_excludes(mut self, excludes: &[&str]) -> Self {
		self.exclude = excludes.iter().map(|exclude| Glob::new(exclude).unwrap()).collect();
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
			let name = path.get_name();
			name.strip_suffix(suffix).unwrap_or(name).to_owned()
		} else {
			path.get_stem().to_owned()
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
					name: path.get_parent().get_name().to_owned(),
				});
			}
		}

		None
	}

	pub fn locate_data(&self, path: &Path, name: &str, is_dir: bool) -> Option<PathBuf> {
		if self.file_type != FileType::InstanceData {
			return None;
		}

		if is_dir {
			if let Some(child_pattern) = &self.child_pattern {
				let data_path = path.join(child_pattern.as_str());
				return Some(data_path);
			}
		} else if let Some(pattern) = &self.pattern {
			let data_path = path.with_file_name(pattern.as_str().replace('*', name));
			return Some(data_path);
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
pub struct Context {
	/// Rules that define how files are synced
	sync_rules: Vec<SyncRule>,
	/// Rules that define which files are ignored
	ignore_rules: Vec<IgnoreRule>,
	/// Whether to use legacy script context
	legacy_scripts: bool,
}

impl Context {
	fn new() -> Self {
		Self {
			sync_rules: Vec::new(),
			ignore_rules: Vec::new(),
			legacy_scripts: true,
		}
	}

	pub fn sync_rules(&self) -> &Vec<SyncRule> {
		if self.sync_rules.is_empty() {
			default_sync_rules()
		} else {
			&self.sync_rules
		}
	}

	pub fn sync_rules_of_type(&self, file_type: &FileType) -> Vec<&SyncRule> {
		self.sync_rules()
			.iter()
			.filter(|rule| rule.file_type == *file_type)
			.collect()
	}

	pub fn ignore_rules(&self) -> &Vec<IgnoreRule> {
		&self.ignore_rules
	}

	pub fn use_legacy_scripts(&self) -> bool {
		self.legacy_scripts
	}
}

impl Default for Context {
	fn default() -> Self {
		Self::new()
	}
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Meta {
	/// Instance source that is guaranteed to exist
	#[serde(skip)]
	pub source: Source,
	/// Project context
	#[serde(skip)]
	pub context: Context,
	/// Whether to keep unknown child instances
	pub keep_unknowns: bool,
}

impl Meta {
	pub fn new() -> Self {
		Self {
			source: Source::new(),
			context: Context::new(),
			keep_unknowns: false,
		}
	}

	pub fn with_source<S: Into<Source>>(mut self, source: S) -> Self {
		self.source = source.into();
		self
	}

	pub fn with_context(mut self, context: &Context) -> Self {
		self.context = context.clone();
		self
	}

	pub fn with_keep_unknowns(mut self, keep_unknowns: bool) -> Self {
		self.keep_unknowns = keep_unknowns;
		self
	}

	pub fn from_project(project: &Project) -> Self {
		let ignore_rules = project
			.ignore_globs
			.clone()
			.into_iter()
			.map(|glob| IgnoreRule {
				pattern: glob,
				path: project.workspace_dir.clone(),
			})
			.collect();

		let context = Context {
			sync_rules: project.sync_rules.clone(),
			ignore_rules,
			legacy_scripts: project.legacy_scripts.unwrap_or(true),
		};

		Self {
			context,
			..Self::default()
		}
	}
}

fn default_sync_rules() -> &'static Vec<SyncRule> {
	static SYNC_RULES: OnceLock<Vec<SyncRule>> = OnceLock::new();

	SYNC_RULES.get_or_init(|| {
		vec![
			SyncRule::new(FileType::Project)
				.with_pattern("*.project.json")
				.with_child_pattern("default.project.json"),
			SyncRule::new(FileType::InstanceData)
				.with_pattern("*.data.json")
				.with_child_pattern(".data.json"),
			SyncRule::new(FileType::InstanceData) // Rojo
				.with_pattern("*.meta.json")
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
				.with_excludes(&["*.model.json", "*.data.json", "*.meta.json"]),
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
		]
	})
}
