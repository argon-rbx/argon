use anyhow::{bail, Result};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

use crate::{
	core::{meta::Meta, snapshot::Snapshot},
	vfs::Vfs,
};

use self::{
	csv::snapshot_csv, data::snapshot_data, dir::snapshot_dir, json::snapshot_json, lua::snapshot_lua,
	project::snapshot_project, txt::snapshot_txt,
};

pub mod csv;
pub mod data;
pub mod dir;
pub mod json;
pub mod lua;
pub mod project;
pub mod txt;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum FileType {
	Project,
	InstanceData,

	ServerScript,
	ClientScript,
	ModuleScript,

	JsonModule,
	JsonModel,
	TomlModel,
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
	pub is_child: bool,
}

impl FileType {
	fn middleware(&self, path: &Path, meta: &Meta, vfs: &Vfs) -> Result<Snapshot> {
		match self {
			FileType::Project => snapshot_project(path, meta, vfs),
			FileType::InstanceData => snapshot_data(path, meta, vfs),
			//
			FileType::ServerScript | FileType::ClientScript | FileType::ModuleScript => {
				snapshot_lua(path, vfs, self.clone().into())
			}
			//
			FileType::JsonModule => snapshot_json(path, vfs),
			FileType::StringValue => snapshot_txt(path, vfs),
			FileType::LocalizationTable => snapshot_csv(path),
			// FileType::JsonModel => {}
			// FileType::TomlModel => {}
			// FileType::RbxmModel => {}
			// FileType::RbxmxModel => {}
			_ => bail!("Unsupported file type! (TEMP)"),
		}
	}
}

/// Returns a snapshot of the given path, `None` if path no longer exists
pub fn new_snapshot(path: &Path, meta: &Meta, vfs: &Vfs) -> Result<Option<Snapshot>> {
	if meta.ignore_globs.iter().any(|glob| glob.matches_path(path)) {
		return Ok(None);
	}

	if !vfs.exists(path) {
		vfs.unwatch(path)?;

		return Ok(None);
	}

	// Get snapshot of a regular file
	if vfs.is_file(path) {
		if let Some(resolved) = meta.sync_rules.iter().find_map(|rule| rule.resolve(path)) {
			let file_type = resolved.file_type;
			let resolved_path = resolved.path;
			let name = resolved.name;

			let snapshot = file_type
				.middleware(&resolved_path, meta, vfs)?
				.with_name(&name)
				.with_path(path)
				.with_file_type(file_type)
				.apply_project_data(meta, path);

			Ok(Some(snapshot))
		} else {
			Ok(None)
		}
	// Get snapshot of directory that contains child source
	} else if let Some(resolved) = meta.sync_rules.iter().find_map(|rule| rule.resolve_child(path)) {
		vfs.watch(path)?;

		let file_type = resolved.file_type;
		let resolved_path = resolved.path;
		let name = resolved.name;

		let mut snapshot = file_type
			.middleware(&resolved_path, meta, vfs)?
			.with_name(&name)
			.with_path(path)
			.with_file_type(file_type.clone())
			.apply_project_data(meta, path);

		if file_type != FileType::Project {
			for path in vfs.read_dir(path)? {
				if path == resolved_path {
					continue;
				}

				if let Some(child_snapshot) = new_snapshot(&path, meta, vfs)? {
					snapshot.add_child(child_snapshot);
				}
			}
		}

		Ok(Some(snapshot))
	// Get snapshot of a directory
	} else {
		vfs.watch(path)?;

		let snapshot = snapshot_dir(path, meta, vfs)?.apply_project_data(meta, path);

		Ok(Some(snapshot))
	}
}
