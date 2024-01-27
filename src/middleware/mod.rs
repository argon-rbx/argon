use anyhow::{bail, Result};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

use crate::{
	core::{
		meta::{Meta, SyncRule},
		snapshot::Snapshot,
	},
	glob::Glob,
	util,
	vfs::Vfs,
};

use self::{dir::snapshot_dir, lua::snapshot_lua, project::snapshot_project};

pub mod dir;
pub mod lua;
pub mod project;

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub enum FileType {
	Project,
	InstanceData,

	ServerScript,
	ClientScript,
	ModuleScript,

	JsonModule,
	TomlModule,
	LocalizationTable,
	StringValue,
	RbxmModel,
	RbxmxModel,
}

#[derive(Debug, Clone)]
pub struct ResolvedSyncRule {
	pub file_type: FileType,
	pub path: PathBuf,
	pub name: String,
}

impl FileType {
	fn middleware(&self, name: &str, path: &Path, meta: &Meta, vfs: &Vfs) -> Result<Option<Snapshot>> {
		match self {
			FileType::Project => snapshot_project(path, meta, vfs),
			// FileType::InstanceData => {}
			//
			FileType::ServerScript | FileType::ClientScript | FileType::ModuleScript => {
				snapshot_lua(name, path, vfs, self.clone().into())
			}
			_ => bail!("Unsupported file type! (TEMP)"),
			// FileType::JsonModule => {}
			// FileType::LocalizationTable => {}
			// FileType::StringValue => {}
			// FileType::RbxmModel => {}
			// FileType::RbxmxModel => {}
		}
	}
}

pub fn new_snapshot(path: &Path, meta: &Meta, vfs: &Vfs) -> Result<Option<Snapshot>> {
	if !vfs.exists(path) {
		vfs.unwatch(path)?;

		return Ok(None);
	}

	// Check if the path is not ignored here

	let is_dir = vfs.is_dir(path);

	if let Some(resolved) = meta
		.sync_rules
		.iter()
		.find_map(|rule| resolve_sync_rule(rule, path, is_dir))
	{
		let file_type = resolved.file_type;
		let path = resolved.path;
		let name = resolved.name;

		file_type.middleware(&name, &path, meta, vfs)
	} else if is_dir {
		vfs.watch(path)?;

		snapshot_dir(path, meta, vfs)
	} else {
		Ok(None)
	}
}

//TODO: support child patterns even when target is a file
fn resolve_sync_rule(rule: &SyncRule, path: &Path, is_dir: bool) -> Option<ResolvedSyncRule> {
	if is_dir {
		if let Some(child_pattern) = &rule.child_pattern {
			let path = path.join(child_pattern.as_str());
			let child_pattern = Glob::from_path(&path).unwrap();

			if let Some(path) = child_pattern.first() {
				if rule.is_excluded(&path) {
					return None;
				}

				let name = util::get_file_name(path.parent().unwrap());

				return Some(ResolvedSyncRule {
					file_type: rule.file_type.clone(),
					name: name.to_string(),
					path,
				});
			}
		}
	// } else if let Some(child_pattern) = &rule.child_pattern {
	// 	if child_pattern.matches_path(path) && !rule.is_excluded(path) {
	// 		return Some(ResolvedSyncRule {
	// 			file_type: rule.file_type.clone(),
	// 			path: path.to_path_buf(),
	// 			name: rule.get_name(path),
	// 		});
	// 	}
	} else if let Some(pattern) = &rule.pattern {
		if pattern.matches_path(path) && !rule.is_excluded(path) {
			return Some(ResolvedSyncRule {
				file_type: rule.file_type.clone(),
				path: path.to_path_buf(),
				name: rule.get_name(path),
			});
		}
	}

	None
}
