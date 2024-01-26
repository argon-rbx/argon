use anyhow::{bail, Result};
use serde::{Deserialize, Serialize};
use std::path::Path;

use crate::{
	core::{meta::Meta, snapshot::Snapshot},
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
		return Ok(None);
	}

	// Check if the path is not ignored here

	let is_dir = vfs.is_dir(path);

	if let Some(resolved) = meta.sync_rules.iter().find_map(|rule| rule.resolve(path, is_dir)) {
		let file_type = resolved.file_type;
		let path = resolved.path;
		let name = resolved.name;

		file_type.middleware(&name, &path, meta, vfs)
	} else if is_dir {
		snapshot_dir(path, meta, vfs)
	} else {
		Ok(None)
	}
}
