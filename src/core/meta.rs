use serde::{Deserialize, Serialize};
use std::{
	fmt::Display,
	path::{Path, PathBuf},
};

use crate::{
	config::Config,
	constants::default_sync_rules,
	ext::PathExt,
	glob::Glob,
	middleware::Middleware,
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

impl Display for NodePath {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "tree/{}", self.inner.join("/"))
	}
}

#[derive(Debug, Clone, PartialEq)]
pub enum SourceKind {
	Path(PathBuf),
	Project(String, PathBuf, Box<ProjectNode>, NodePath),
	None,
}

impl SourceKind {
	pub fn path(&self) -> Option<&Path> {
		match self {
			SourceKind::Path(path) => Some(path),
			SourceKind::Project(_, path, _, _) => Some(path),
			_ => None,
		}
	}
}

#[derive(Debug, Clone, PartialEq)]
#[repr(usize)]
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
		unsafe { *<*const _>::from(self).cast::<usize>() }
	}
}

#[derive(Debug, Clone, PartialEq)]
pub struct Source {
	// Source used to rebuild the snapshot
	inner: SourceKind,
	// Existing paths associated with the instance
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
			inner: SourceKind::Project(name.to_owned(), path.to_owned(), Box::new(node), node_path),
			relevant: Vec::new(),
		}
	}

	pub fn with_relevant(mut self, relevant: Vec<SourceEntry>) -> Self {
		self.relevant = relevant;
		self
	}

	pub fn add_file(&mut self, path: &Path) {
		self.relevant.push(SourceEntry::File(path.to_owned()))
	}

	pub fn add_data(&mut self, path: &Path) {
		self.relevant.push(SourceEntry::Data(path.to_owned()))
	}

	pub fn add_project(&mut self, path: &Path) {
		self.relevant.push(SourceEntry::Project(path.to_owned()))
	}

	pub fn set_data(&mut self, path: Option<&Path>) {
		self.relevant.retain(|entry| !matches!(entry, SourceEntry::Data(_)));

		if let Some(path) = path {
			self.add_data(path)
		}
	}

	pub fn extend_relevant(&mut self, entries: Vec<SourceEntry>) {
		self.relevant.extend(entries)
	}

	pub fn get(&self) -> &SourceKind {
		&self.inner
	}

	pub fn get_mut(&mut self) -> &mut SourceKind {
		&mut self.inner
	}

	pub fn get_file(&self) -> Option<&SourceEntry> {
		self.relevant.iter().find(|entry| matches!(entry, SourceEntry::File(_)))
	}

	pub fn get_file_mut(&mut self) -> Option<&mut SourceEntry> {
		self.relevant
			.iter_mut()
			.find(|entry| matches!(entry, SourceEntry::File(_)))
	}

	pub fn get_folder_mut(&mut self) -> Option<&mut SourceEntry> {
		self.relevant
			.iter_mut()
			.find(|entry| matches!(entry, SourceEntry::Folder(_)))
	}

	pub fn get_data(&self) -> Option<&SourceEntry> {
		self.relevant.iter().find(|entry| matches!(entry, SourceEntry::Data(_)))
	}

	pub fn relevant(&self) -> &Vec<SourceEntry> {
		&self.relevant
	}

	pub fn relevant_mut(&mut self) -> &mut Vec<SourceEntry> {
		&mut self.relevant
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
	pub middleware: Middleware,
	pub name: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct SyncRule {
	#[serde(rename = "type")]
	pub middleware: Middleware,

	pub pattern: Option<Glob>,
	pub child_pattern: Option<Glob>,
	#[serde(default)]
	pub exclude: Vec<Glob>,

	pub suffix: Option<String>,
}

impl SyncRule {
	pub fn new(middleware: Middleware) -> Self {
		Self {
			middleware,
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
			if pattern.matches_path(path) && !self.is_excluded(path) && self.middleware != Middleware::InstanceData {
				return Some(ResolvedSyncRule {
					middleware: self.middleware.clone(),
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
				&& self.middleware != Middleware::InstanceData
			{
				return Some(ResolvedSyncRule {
					middleware: self.middleware.clone(),
					name: path.get_parent().get_name().to_owned(),
				});
			}
		}

		None
	}

	pub fn matches(&self, path: &Path) -> bool {
		self.resolve(path).is_some()
	}

	pub fn matches_child(&self, path: &Path) -> bool {
		self.resolve_child(path).is_some()
	}

	pub fn locate(&self, path: &Path, name: &str, is_dir: bool) -> Option<PathBuf> {
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

	pub fn matches_with_dir(&self, path: &Path) -> bool {
		match path.strip_prefix(&self.path) {
			Ok(suffix) => self.pattern.matches_path_with_dir(suffix),
			Err(_) => false,
		}
	}

	pub fn from_globs(globs: Vec<Glob>, path: PathBuf) -> Vec<Self> {
		globs
			.into_iter()
			.map(|glob| IgnoreRule {
				pattern: glob,
				path: path.clone(),
			})
			.collect()
	}
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct SyncbackFilter {
	pub ignore_rules: Vec<IgnoreRule>,
	pub ignore_names: Vec<String>,
	pub ignore_classes: Vec<String>,
	pub ignore_properties: Vec<String>,
}

impl SyncbackFilter {
	pub fn matches_path(&self, path: &Path) -> bool {
		self.ignore_rules.iter().any(|rule| rule.matches_with_dir(path))
	}

	pub fn matches_name(&self, name: &str) -> bool {
		self.ignore_names.contains(&name.to_owned())
	}

	pub fn matches_class(&self, class: &str) -> bool {
		self.ignore_classes.contains(&class.to_owned())
	}

	pub fn matches_property(&self, property: &str) -> bool {
		self.ignore_properties.contains(&property.to_owned())
	}
}

#[derive(Debug, Clone, PartialEq)]
pub struct Context {
	/// Rules that define how files are synced
	sync_rules: Vec<SyncRule>,
	/// Rules that define which files are ignored
	ignore_rules: Vec<IgnoreRule>,
	/// Filter which ignores specific instances and properties
	syncback_filter: SyncbackFilter,
	/// Whether to use legacy script context
	legacy_scripts: bool,
}

impl Context {
	fn new() -> Self {
		Self {
			sync_rules: Vec::new(),
			ignore_rules: Vec::new(),
			syncback_filter: SyncbackFilter::default(),
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

	pub fn sync_rules_of_type(&self, middleware: &Middleware, syncback: bool) -> Vec<&SyncRule> {
		let config = Config::new();

		self.sync_rules()
			.iter()
			.filter(|rule| {
				if let Some(child_pattern) = rule.child_pattern.as_ref() {
					if child_pattern.as_str().starts_with(".src") && config.rojo_mode && syncback {
						return false;
					}
				}

				if let Some(pattern) = rule.pattern.as_ref().or(rule.child_pattern.as_ref()) {
					if pattern.as_str().ends_with(".data.json") && config.rojo_mode && syncback {
						return false;
					}

					if pattern.as_str().ends_with(".luau") && config.lua_extension {
						return false;
					}
				}

				rule.middleware == *middleware
			})
			.collect()
	}

	pub fn ignore_rules(&self) -> &Vec<IgnoreRule> {
		&self.ignore_rules
	}

	pub fn syncback_filter(&self) -> &SyncbackFilter {
		&self.syncback_filter
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
	#[serde(skip)]
	/// Original name of the instance
	pub original_name: Option<String>,
	/// Custom Mesh Part source path
	pub mesh_source: Option<String>,
}

impl Meta {
	// Creating new meta

	pub fn new() -> Self {
		Self {
			source: Source::new(),
			context: Context::new(),
			keep_unknowns: false,
			original_name: None,
			mesh_source: None,
		}
	}

	pub fn from_project(project: &Project) -> Self {
		let syncback_filter = if let Some(syncback) = &project.syncback {
			SyncbackFilter {
				ignore_rules: IgnoreRule::from_globs(syncback.ignore_globs.clone(), project.workspace_dir.clone()),
				ignore_names: syncback.ignore_names.clone(),
				ignore_classes: syncback.ignore_classes.clone(),
				ignore_properties: syncback.ignore_properties.clone(),
			}
		} else {
			SyncbackFilter::default()
		};

		let context = Context {
			sync_rules: project.sync_rules.clone(),
			ignore_rules: IgnoreRule::from_globs(project.ignore_globs.clone(), project.workspace_dir.clone()),
			syncback_filter,
			legacy_scripts: project.legacy_scripts.unwrap_or(true),
		};

		Self {
			context,
			..Self::default()
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

	pub fn with_original_name(mut self, original_name: String) -> Self {
		self.original_name = Some(original_name);
		self
	}

	pub fn with_mesh_source(mut self, mesh_source: String) -> Self {
		self.mesh_source = Some(mesh_source);
		self
	}

	// Overwriting meta fields

	pub fn set_source<S: Into<Source>>(&mut self, source: S) {
		self.source = source.into();
	}

	pub fn set_context(&mut self, context: &Context) {
		self.context = context.clone();
	}

	pub fn set_keep_unknowns(&mut self, keep_unknowns: bool) {
		self.keep_unknowns = keep_unknowns;
	}

	pub fn set_original_name(&mut self, original_name: Option<String>) {
		self.original_name = original_name;
	}

	pub fn set_mesh_source(&mut self, mesh_source: Option<String>) {
		self.mesh_source = mesh_source;
	}
}
