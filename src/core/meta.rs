use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

use crate::{glob::Glob, middleware::FileType, util};

#[derive(Serialize, Deserialize, Debug)]
pub struct SyncRule {
	#[serde(rename = "type")]
	pub file_type: FileType,

	pub pattern: Option<Glob>,
	pub child_pattern: Option<Glob>,
	pub exclude: Option<Glob>,

	pub suffix: Option<String>,
}

#[derive(Debug)]
pub struct ResolvedSyncRule {
	pub file_type: FileType,
	pub path: PathBuf,
	pub name: String,
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
			name.strip_prefix(suffix).unwrap_or(name).into()
		} else {
			util::get_file_stem(path).into()
		}
	}

	pub fn resolve(&self, path: &Path, is_dir: bool) -> Option<ResolvedSyncRule> {
		if is_dir {
			if let Some(child_pattern) = &self.child_pattern {
				let path = path.join(child_pattern.as_str());
				let child_pattern = Glob::from_path(&path).unwrap();

				if let Some(path) = child_pattern.first() {
					if self.is_excluded(&path) {
						return None;
					}

					return Some(ResolvedSyncRule {
						file_type: self.file_type.clone(),
						name: self.get_name(&path),
						path,
					});
				}
			}
		} else if let Some(pattern) = &self.pattern {
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
}

#[derive(Debug)]
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
			sync_rule!("*.server.luau", ".src.server.luau", ServerScript, ".server.luau"),
			sync_rule!("*.client.luau", ".src.client.luau", ClientScript, ".client.luau"),
			sync_rule!("*.{lua, luau}", ".src.{lua, luau}", ModuleScript),
			//
			sync_rule!("*.json", ".src.json", JsonModule, ".data.json"),
			sync_rule!("*.toml", ".src.toml", TomlModule),
			sync_rule!("*.csv", ".src.csv", LocalizationTable),
			sync_rule!("*.txt", ".src.txt", StringValue),
			sync_rule!("*.rbxm", ".src.rbxm", RbxmModel),
			sync_rule!("*.rbxmx", ".src.rbxmx", RbxmxModel),
		];

		Self {
			ignore_globs: vec![],
			sync_rules,
		}
	}
}
