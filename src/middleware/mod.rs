use anyhow::{bail, Result};
use log::error;
use rbx_dom_weak::types::{Attributes, Tags, Variant};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, path::Path};

use crate::{
	core::{meta::Meta, snapshot::Snapshot},
	project::ProjectNode,
	util,
	vfs::Vfs,
};

use self::project::snapshot_project;

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
			FileType::Project => snapshot_project(name, path, meta, vfs),
			_ => bail!("Unsupported file type! (TEMP)"),
			// FileType::InstanceData => {}
			// FileType::ServerScript => {}
			// FileType::ClientScript => {}
			// FileType::ModuleScript => {}
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

	let is_dir = vfs.is_dir(path);

	if let Some(resolved) = meta.sync_rules.iter().find_map(|rule| rule.resolve(path, is_dir)) {
		let file_type = resolved.file_type;
		let path = resolved.path;
		let name = resolved.name;

		file_type.middleware(&name, &path, meta, vfs)
	} else if is_dir {
		//dir
		Ok(Some(Snapshot::new("temp"))) //TEMP
	} else {
		Ok(None)
	}
}
