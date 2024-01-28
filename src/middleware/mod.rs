use anyhow::{bail, Result};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

use crate::{
	core::{meta::Meta, snapshot::Snapshot},
	vfs::Vfs,
};

use self::{dir::snapshot_dir, lua::snapshot_lua, project::snapshot_project};

pub mod dir;
pub mod lua;
pub mod project;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
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
	pub is_child: bool,
}

impl FileType {
	fn middleware(&self, name: &str, path: &Path, meta: &Meta, vfs: &Vfs) -> Result<Snapshot> {
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

	if vfs.is_file(path) {
		if let Some(resolved) = meta.sync_rules.iter().find_map(|rule| rule.resolve(path)) {
			let file_type = resolved.file_type;
			let resolved_path = resolved.path;
			let name = resolved.name;

			let t = file_type.middleware(&name, &resolved_path, meta, vfs)?;

			if name == "lib" {
				println!("{:#?}", t);
			}

			Ok(Some(t))
		} else {
			Ok(None)
		}
	} else if let Some(resolved) = meta.sync_rules.iter().find_map(|rule| rule.resolve_child(path)) {
		vfs.watch(path)?;

		let file_type = resolved.file_type;
		let resolved_path = resolved.path;
		let name = resolved.name;

		let mut snapshot = file_type.middleware(&name, &resolved_path, meta, vfs)?;

		if file_type != FileType::Project {
			// println!("{:#?}", path);

			for path in vfs.read_dir(path)? {
				if path == resolved_path {
					continue;
				}

				if let Some(child_snapshot) = new_snapshot(&path, meta, vfs)? {
					snapshot.children.push(child_snapshot);
				}
			}
		}

		Ok(Some(snapshot))
	} else {
		vfs.watch(path)?;
		snapshot_dir(path, meta, vfs)
	}
}
