use anyhow::Result;
use colored::Colorize;
use log::trace;
use serde::{Deserialize, Serialize};
use std::{
	fmt::{self, Display, Formatter},
	path::Path,
};

use crate::{
	core::{
		meta::{Meta, ResolvedSyncRule},
		snapshot::Snapshot,
	},
	util::{Desc, PathExt},
	vfs::Vfs,
	BLACKLISTED_PATHS,
};

use self::{
	csv::snapshot_csv, data::snapshot_data, dir::snapshot_dir, json::snapshot_json, json_model::snapshot_json_model,
	lua::snapshot_lua, project::snapshot_project, rbxm::snapshot_rbxm, rbxmx::snapshot_rbxmx, toml::snapshot_toml,
	txt::snapshot_txt,
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
	fn middleware(&self, path: &Path, meta: &Meta, vfs: &Vfs) -> Result<Snapshot> {
		let snapshot = match self {
			FileType::Project => snapshot_project(path, vfs),
			FileType::InstanceData => snapshot_data(path, meta, vfs),
			//
			FileType::ServerScript | FileType::ClientScript | FileType::ModuleScript => {
				snapshot_lua(path, vfs, self.clone().into())
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

		snapshot.with_desc(|| {
			format!(
				"Failed to snapshot {} at {}",
				self.to_string().bold(),
				path.display().to_string().bold()
			)
		})
	}
}

/// Returns a snapshot of the given path, `None` if path no longer exists
pub fn new_snapshot(path: &Path, meta: &Meta, vfs: &Vfs) -> Result<Option<Snapshot>> {
	if BLACKLISTED_PATHS.iter().any(|blacklisted| path.ends_with(blacklisted))
		|| meta.ignore_rules.iter().any(|rule| rule.matches(path))
	{
		trace!("Snapshot of {} not created: ignored or blacklisted", path.display());
		return Ok(None);
	}

	if !vfs.exists(path) {
		trace!("Snapshot of {} not created: path does not exist", path.display());

		vfs.unwatch(path)?;

		return Ok(None);
	}

	trace!("Snapshot of {} created", path.display());

	if vfs.is_file(path) {
		// Get a snapshot of a file that is child source or data
		if meta.sync_rules.iter().any(|rule| rule.matches_child(path)) {
			new_snapshot_file_child(path.get_parent(), meta, vfs)
		} else {
			// Get a snapshot of a regular file
			new_snapshot_file(path, meta, vfs)
		}

	// Get a snapshot of a directory that might contain child source or data
	} else if let Some(snapshot) = new_snapshot_file_child(path, meta, vfs)? {
		// We don't need to watch whole parent directory of a project
		if let Some(file_type) = &snapshot.file_type {
			if *file_type == FileType::Project {
				return Ok(Some(snapshot));
			}
		}

		vfs.watch(path)?;

		Ok(Some(snapshot))
	// Get a snapshot of a directory
	} else {
		vfs.watch(path)?;
		new_snapshot_dir(path, meta, vfs)
	}
}

/// Create a snapshot of a regular file,
/// example: `foo/bar.lua`
fn new_snapshot_file(path: &Path, meta: &Meta, vfs: &Vfs) -> Result<Option<Snapshot>> {
	if let Some(resolved) = meta.sync_rules.iter().find_map(|rule| rule.resolve(path)) {
		let file_type = resolved.file_type;
		let resolved_path = resolved.path;
		let name = resolved.name;

		let snapshot = file_type
			.middleware(&resolved_path, meta, vfs)?
			.with_file_type(file_type.clone())
			.with_name(&name)
			.with_path(path)
			.apply_project_data(meta, path);

		Ok(Some(snapshot))
	} else {
		Ok(None)
	}
}

/// Create a snapshot of a directory that has a child source or data,
/// example: `foo/bar` that contains: `foo/bar/.src.lua`
fn new_snapshot_file_child(path: &Path, meta: &Meta, vfs: &Vfs) -> Result<Option<Snapshot>> {
	if meta.was_processed(path) {
		return Ok(None);
	}

	if let Some(resolved_rules) = resolve_child_rules(path, meta) {
		let (mut snapshot, file_type, resolved_path) = match (resolved_rules.source_rule, resolved_rules.data_rule) {
			(Some(source_rule), Some(data_rule)) => {
				let mut paths = vec![path.to_owned()];

				let data_snapshot = {
					let file_type = data_rule.file_type;
					let resolved_path = data_rule.path;

					paths.push(resolved_path.clone());

					file_type.middleware(&resolved_path, meta, vfs)?
				};

				let file_type = source_rule.file_type;
				let resolved_path = source_rule.path;
				let name = source_rule.name;

				paths.push(resolved_path.clone());

				(
					file_type
						.middleware(&resolved_path, meta, vfs)?
						.with_file_type(file_type.clone())
						.with_name(&name)
						.with_paths(paths)
						.with_data(data_snapshot)
						.apply_project_data(meta, path),
					file_type,
					resolved_path,
				)
			}
			(Some(rule), None) | (None, Some(rule)) => {
				let file_type = rule.file_type;
				let resolved_path = rule.path;
				let name = rule.name;

				let paths = vec![path.to_owned(), resolved_path.clone()];

				(
					file_type
						.middleware(&resolved_path, meta, vfs)?
						.with_file_type(file_type.clone())
						.with_name(&name)
						.with_paths(paths)
						.apply_project_data(meta, path),
					file_type,
					resolved_path,
				)
			}
			_ => unreachable!(),
		};

		if file_type != FileType::Project {
			let meta = meta.clone().with_processed_path(path);

			for path in vfs.read_dir(path)? {
				if path == resolved_path {
					continue;
				}

				if let Some(child_snapshot) = new_snapshot(&path, &meta, vfs)? {
					snapshot.add_child(child_snapshot);
				}
			}
		}

		Ok(Some(snapshot))
	} else {
		Ok(None)
	}
}

/// Create snapshot of a directory,
/// example: `foo/bar`
fn new_snapshot_dir(path: &Path, meta: &Meta, vfs: &Vfs) -> Result<Option<Snapshot>> {
	let snapshot = snapshot_dir(path, meta, vfs)?.apply_project_data(meta, path);

	Ok(Some(snapshot))
}

#[derive(Debug, Clone)]
struct ResolvedChildRules {
	pub source_rule: Option<ResolvedSyncRule>,
	pub data_rule: Option<ResolvedSyncRule>,
}

fn resolve_child_rules(path: &Path, meta: &Meta) -> Option<ResolvedChildRules> {
	let mut source_resolved_rule = None;
	let mut data_resolved_rule = None;

	let resolved_rule = meta.sync_rules.iter().find_map(|rule| rule.resolve_child(path))?;

	if resolved_rule.file_type == FileType::InstanceData {
		for rule in &meta.sync_rules {
			if rule.file_type == FileType::InstanceData {
				continue;
			}

			if let Some(source_rule) = rule.resolve_child(path) {
				source_resolved_rule = Some(source_rule);
				break;
			}
		}

		data_resolved_rule = Some(resolved_rule);
	} else {
		if let Some(data_rule) = meta
			.sync_rules
			.iter()
			.find(|rule| rule.file_type == FileType::InstanceData)
		{
			if let Some(data_rule) = data_rule.resolve_child(path) {
				data_resolved_rule = Some(data_rule);
			}
		}

		source_resolved_rule = Some(resolved_rule);
	}

	Some(ResolvedChildRules {
		source_rule: source_resolved_rule,
		data_rule: data_resolved_rule,
	})
}
