use serde::{Deserialize, Serialize};
use std::path::Path;

use crate::{
	core::{dom::Dom, meta::Context},
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
	fn use_middleware(&self) {}
}

pub fn from_path(path: &Path, context: &Context, vfs: &Vfs, dom: &Dom) -> Option<FileType> {
	let is_file = vfs.is_file(path);

	if let Some(file_type) = get_file_type(path, context, is_file) {
		file_type.use_middleware();
	} else if !is_file {
		//dir
	} else {
		return None;
	}

	None
}

fn get_file_type<'a>(path: &Path, context: &'a Context, is_file: bool) -> Option<&'a FileType> {
	if is_file {
		for rule in &context.sync_rules {
			if rule.matches(path) {
				return Some(&rule.file_type);
			}
		}
	} else {
		for rule in &context.sync_rules {
			if rule.matches_child(path) {
				return Some(&rule.file_type);
			}
		}
	}

	None
}
