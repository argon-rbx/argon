use anyhow::Result;
use colored::Colorize;
use log::trace;
use serde::{Deserialize, Serialize};
use std::{
	fmt::{self, Display, Formatter},
	path::Path,
};

use self::{
	csv::snapshot_csv,
	data::{snapshot_data, DataSnapshot},
	dir::snapshot_dir,
	json::snapshot_json,
	json_model::snapshot_json_model,
	lua::snapshot_lua,
	project::snapshot_project,
	rbxm::snapshot_rbxm,
	rbxmx::snapshot_rbxmx,
	toml::snapshot_toml,
	txt::snapshot_txt,
};
use crate::{
	core::{
		meta::{Context, Meta, Source},
		snapshot::Snapshot,
	},
	ext::{PathExt, ResultExt},
	vfs::Vfs,
	BLACKLISTED_PATHS,
};

pub mod csv;
pub mod data;
pub mod dir;
pub mod json;
pub mod json_model;
pub mod lua;
pub mod project;
pub mod rbxm;
pub mod rbxmx;
pub mod toml;
pub mod txt;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum FileType {
	Project,
	InstanceData,

	ServerScript,
	ClientScript,
	ModuleScript,

	StringValue,
	LocalizationTable,
	JsonModule,
	TomlModule,

	JsonModel,
	RbxmModel,
	RbxmxModel,
}

impl Display for FileType {
	fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
		write!(f, "{:?}", self)
	}
}

impl FileType {
	fn middleware(&self, path: &Path, context: &Context, vfs: &Vfs) -> Result<Snapshot> {
		let result = match self {
			FileType::Project => snapshot_project(path, vfs),
			FileType::InstanceData => unreachable!(),
			//
			FileType::ServerScript | FileType::ClientScript | FileType::ModuleScript => {
				snapshot_lua(path, context, vfs, self.clone().into())
			}
			//
			FileType::StringValue => snapshot_txt(path, vfs),
			FileType::LocalizationTable => snapshot_csv(path, vfs),
			FileType::JsonModule => snapshot_json(path, vfs),
			FileType::TomlModule => snapshot_toml(path, vfs),
			//
			FileType::JsonModel => snapshot_json_model(path, vfs),
			FileType::RbxmModel => snapshot_rbxm(path, vfs),
			FileType::RbxmxModel => snapshot_rbxmx(path, vfs),
		};

		result.with_desc(|| {
			format!(
				"Failed to snapshot {} at {}",
				self.to_string().bold(),
				path.display().to_string().bold()
			)
		})
	}
}

/// Returns a snapshot of the given path, `None` if path no longer exists
pub fn new_snapshot(path: &Path, context: &Context, vfs: &Vfs) -> Result<Option<Snapshot>> {
	if BLACKLISTED_PATHS.iter().any(|blacklisted| path.ends_with(blacklisted))
		|| context.ignore_rules().iter().any(|rule| rule.matches(path))
	{
		trace!("Snapshot of {} not created: ignored or blacklisted", path.display());
		return Ok(None);
	}

	if !vfs.exists(path) {
		trace!("Snapshot of {} not created: path does not exist", path.display());

		vfs.unwatch(path)?;

		return Ok(None);
	}

	trace!("Creating snapshot of {}", path.display());

	if vfs.is_file(path) {
		if let Some(snapshot) = new_snapshot_file_child(path, context, vfs)? {
			Ok(Some(snapshot))
		} else if let Some(snapshot) = new_snapshot_file(path, context, vfs)? {
			Ok(Some(snapshot))
		} else {
			Ok(None)
		}
	} else {
		vfs.watch(path)?;

		for path in vfs.read_dir(path)? {
			if let Some(snapshot) = new_snapshot_file_child(&path, context, vfs)? {
				return Ok(Some(snapshot));
			}
		}

		new_snapshot_dir(path, context, vfs)
	}
}

/// Create a snapshot of a regular file,
/// example: `foo/bar.lua`
fn new_snapshot_file(path: &Path, context: &Context, vfs: &Vfs) -> Result<Option<Snapshot>> {
	if let Some(resolved) = context.sync_rules().iter().find_map(|rule| rule.resolve(path)) {
		let file_type = resolved.file_type;
		let name = resolved.name;

		let mut snapshot = file_type.middleware(path, context, vfs)?;

		if file_type != FileType::Project {
			snapshot.set_name(&name);
			snapshot.set_meta(Meta::new().with_context(context).with_source(Source::file(path)));
		}

		if let Some(instance_data) = get_instance_data(&name, path, context, vfs)? {
			snapshot.set_data(instance_data);
		}

		Ok(Some(snapshot))
	} else {
		Ok(None)
	}
}

/// Create a snapshot of a directory that has a child source or data,
/// example: `foo/bar/.src.lua`
fn new_snapshot_file_child(path: &Path, context: &Context, vfs: &Vfs) -> Result<Option<Snapshot>> {
	if let Some(resolved) = context.sync_rules().iter().find_map(|rule| rule.resolve_child(path)) {
		let file_type = resolved.file_type;
		let name = resolved.name;
		let parent = path.get_parent();

		let mut snapshot = file_type.middleware(path, context, vfs)?;

		if file_type != FileType::Project {
			snapshot.set_name(&name);
			snapshot.set_meta(
				Meta::new()
					.with_context(context)
					.with_source(Source::child_file(parent, path)),
			);

			for entry in vfs.read_dir(parent)? {
				if entry == path {
					continue;
				}

				if let Some(child_snapshot) = new_snapshot(&entry, context, vfs)? {
					snapshot.add_child(child_snapshot);
				}
			}
		}

		if let Some(instance_data) = get_instance_data(&name, parent, context, vfs)? {
			snapshot.set_data(instance_data);
		}

		Ok(Some(snapshot))
	} else {
		Ok(None)
	}
}

/// Create snapshot of a directory,
/// example: `foo/bar`
fn new_snapshot_dir(path: &Path, context: &Context, vfs: &Vfs) -> Result<Option<Snapshot>> {
	let mut snapshot = snapshot_dir(path, context, vfs)?;

	if let Some(instance_data) = get_instance_data(&snapshot.name, path, context, vfs)? {
		snapshot.set_data(instance_data);
	}

	Ok(Some(snapshot))
}

fn get_instance_data(name: &str, path: &Path, context: &Context, vfs: &Vfs) -> Result<Option<DataSnapshot>> {
	for sync_rule in context.sync_rules_of_type(&FileType::InstanceData) {
		if let Some(data_path) = sync_rule.locate_data(path, name, vfs.is_dir(path)) {
			if vfs.exists(&data_path) {
				return Ok(Some(snapshot_data(&data_path, vfs)?));
			}
		}
	}

	Ok(None)
}
