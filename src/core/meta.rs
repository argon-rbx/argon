use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

use crate::{glob::Glob, middleware::FileType, util};

#[derive(Debug, Clone)]
pub struct ResolvedSyncRule {
	pub file_type: FileType,
	pub path: PathBuf,
	pub name: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SyncRule {
	#[serde(rename = "type")]
	pub file_type: FileType,

	pub pattern: Option<Glob>,
	pub child_pattern: Option<Glob>,
	pub exclude: Option<Glob>,

	pub suffix: Option<String>,
}

impl SyncRule {
	pub fn matches(&self, path: &Path) -> bool {
		if let Some(pattern) = &self.pattern {
			if pattern.matches_path(path) {
				return !self.is_excluded(path);
			}
		}

		false
	}

	pub fn matches_child(&self, path: &Path) -> bool {
		if let Some(child_pattern) = &self.child_pattern {
			let path = path.join(child_pattern.as_str());

			if child_pattern.matches_path(&path) {
				return !self.is_excluded(&path);
			}
		}

		false
	}

	pub fn is_excluded(&self, path: &Path) -> bool {
		self.exclude
			.as_ref()
			.map(|exclude| exclude.matches_path(path))
			.unwrap_or(false)
	}

	pub fn get_name(&self, path: &Path) -> String {
		if let Some(suffix) = &self.suffix {
			let name = util::get_file_name(path);
			name.strip_suffix(suffix).unwrap_or(name).into()
		} else {
			util::get_file_stem(path).into()
		}
	}

	pub fn resolve(&self, path: &Path) -> Option<ResolvedSyncRule> {
		if let Some(pattern) = &self.pattern {
			if pattern.matches_path(path) && !self.is_excluded(path) {
				return Some(ResolvedSyncRule {
					file_type: self.file_type.clone(),
					path: path.to_path_buf(),
					name: self.get_name(path),
				});
			}
		}

		None
	}

	pub fn resolve_child(&self, path: &Path) -> Option<ResolvedSyncRule> {
		if let Some(child_pattern) = &self.child_pattern {
			let path = path.join(child_pattern.as_str());
			let child_pattern = Glob::from_path(&path).unwrap();

			if let Some(path) = child_pattern.first() {
				if self.is_excluded(&path) {
					return None;
				}

				let name = util::get_file_name(path.parent().unwrap());

				return Some(ResolvedSyncRule {
					file_type: self.file_type.clone(),
					name: name.to_string(),
					path,
				});
			}
		}

		None
	}
}

#[derive(Debug, Clone)]
pub struct Meta {
	pub ignore_globs: Vec<Glob>,
	pub sync_rules: Vec<SyncRule>,
}

impl Meta {
	pub fn empty() -> Self {
		Self {
			ignore_globs: Vec::new(),
			sync_rules: Vec::new(),
		}
	}

	pub fn is_empty(&self) -> bool {
		self.ignore_globs.is_empty() && self.sync_rules.is_empty()
	}
}

macro_rules! sync_rule {
	($pattern:expr, $child_pattern:expr, $file_type:ident) => {
		SyncRule {
			file_type: FileType::$file_type,

			pattern: Some(Glob::new($pattern).unwrap()),
			child_pattern: Some(Glob::new($child_pattern).unwrap()),
			exclude: None,

			suffix: None,
		}
	};
	($pattern:expr, $child_pattern:expr, $file_type:ident, $suffix:expr) => {
		SyncRule {
			file_type: FileType::$file_type,

			pattern: Some(Glob::new($pattern).unwrap()),
			child_pattern: Some(Glob::new($child_pattern).unwrap()),
			exclude: None,

			suffix: Some($suffix.to_string()),
		}
	};
	($pattern:expr, $child_pattern:expr, $file_type:ident, $suffix:expr, $exclude:expr) => {
		SyncRule {
			file_type: FileType::$file_type,

			pattern: Some(Glob::new($pattern).unwrap()),
			child_pattern: Some(Glob::new($child_pattern).unwrap()),
			exclude: Some(Glob::new($exclude).unwrap()),

			suffix: Some($suffix.to_string()),
		}
	};
	($child_pattern:expr, $file_type:ident) => {
		SyncRule {
			file_type: FileType::$file_type,

			pattern: None,
			child_pattern: Some(Glob::new($child_pattern).unwrap()),
			exclude: None,

			suffix: None,
		}
	};
}

impl Default for Meta {
	fn default() -> Self {
		let sync_rules = vec![
			sync_rule!("*.project.json", Project),
			sync_rule!(".data.json", InstanceData),
			//
			sync_rule!("*.server.lua", ".src.server.lua", ServerScript, ".server.lua"),
			sync_rule!("*.client.lua", ".src.client.lua", ClientScript, ".client.lua"),
			sync_rule!("*.lua", ".src.lua", ModuleScript),
			sync_rule!("*.server.luau", ".src.server.luau", ServerScript, ".server.luau"),
			sync_rule!("*.client.luau", ".src.client.luau", ClientScript, ".client.luau"),
			sync_rule!("*.luau", ".src.luau", ModuleScript),
			//
			sync_rule!("*.txt", ".src.txt", StringValue),
			sync_rule!("*.csv", ".src.csv", LocalizationTable),
			sync_rule!("*.json", ".src.json", JsonModule, ".data.json"),
			sync_rule!("*.toml", ".src.toml", TomlModule),
			sync_rule!("*.rbxm", ".src.rbxm", RbxmModel),
			sync_rule!("*.rbxmx", ".src.rbxmx", RbxmxModel),
		];

		Self {
			ignore_globs: vec![],
			sync_rules,
		}
	}
}
