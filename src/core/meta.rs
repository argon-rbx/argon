use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::{glob::Glob, middleware::FileType};

#[derive(Serialize, Deserialize, Debug)]
pub struct SyncRule {
	pub pattern: Glob,
	pub exclude: Option<Glob>,
	#[serde(rename = "type")]
	pub file_type: FileType,
	pub suffix: Option<String>,
	pub child: Option<Glob>,
}

impl SyncRule {
	pub fn matches(&self, path: &Path) -> bool {
		if self.pattern.matches_path(path) {
			!self.is_excluded(path)
		} else {
			false
		}
	}

	pub fn matches_child(&self, path: &Path) -> bool {
		if let Some(child) = &self.child {
			if child.matches_path(path) {
				return !self.is_excluded(path);
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
