use serde::{Deserialize, Serialize};
use std::path::Path;

use crate::{
	core::{dom::Tree, meta::Meta, snapshot::Snapshot},
	vfs::Vfs,
};

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
	LocalizationTable,
	StringValue,
	RbxmModel,
	RbxmxModel,
}

impl FileType {
	fn middleware(&self, path: &Path, meta: &Meta, vfs: &Vfs) -> Option<Snapshot> {
		match self {
			FileType::Project => project::main(path, meta, vfs),
			_ => None,
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

pub fn from_path(path: &Path, meta: &Meta, vfs: &Vfs) -> Option<Snapshot> {
	let is_file = vfs.is_file(path);

	if let Some(file_type) = get_file_type(path, meta, is_file) {
		file_type.middleware(path, meta, vfs)
	} else if !is_file {
		//dir
		Some(Snapshot::new()) //TEMP
	} else {
		None
	}
}

fn get_file_type<'a>(path: &Path, meta: &'a Meta, is_file: bool) -> Option<&'a FileType> {
	if is_file {
		for rule in &meta.sync_rules {
			if rule.matches(path) {
				return Some(&rule.file_type);
			}
		}
	} else {
		for rule in &meta.sync_rules {
			if rule.matches_child(path) {
				return Some(&rule.file_type);
			}
		}
	}

	None
}
