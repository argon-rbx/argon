use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::Path;

use crate::{
	core::{
		meta::{Meta, ResolvedSyncRule},
		snapshot::Snapshot,
	},
	vfs::Vfs,
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

#[derive(Debug, Clone)]
struct RelevantRules {
	pub source_rule: Option<ResolvedSyncRule>,
	pub data_rule: Option<ResolvedSyncRule>,
}

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
			FileType::StringValue => snapshot_txt(path, vfs),
			FileType::LocalizationTable => snapshot_csv(path, vfs),
			FileType::JsonModule => snapshot_json(path, vfs),
			FileType::TomlModule => snapshot_toml(path, vfs),
			//
			FileType::JsonModel => snapshot_json_model(path, vfs),
			FileType::RbxmModel => snapshot_rbxm(path, vfs),
			FileType::RbxmxModel => snapshot_rbxmx(path, vfs),
		}
	}
}

/// Returns a snapshot of the given path, `None` if path no longer exists
pub fn new_snapshot(path: &Path, meta: &Meta, vfs: &Vfs) -> Result<Option<Snapshot>> {
	if meta.ignore_rules.iter().any(|rule| rule.matches(path)) {
		return Ok(None);
	}

	if !vfs.exists(path) {
		vfs.unwatch(path)?;

		return Ok(None);
	}

	// Get snapshot of a regular file
	if vfs.is_file(path) {
		if let Some(resolved_rules) = resolve_relevant_rules(path, meta, vfs) {
			match (resolved_rules.source_rule, resolved_rules.data_rule) {
				(Some(source_rule), Some(data_rule)) => {
					// We don't want to create same snapshot twice
					if meta.included_data_paths.contains(&data_rule.path)
						|| meta.included_data_paths.contains(&source_rule.path)
					{
						return Ok(None);
					}

					let data_snapshot = {
						let file_type = data_rule.file_type;
						let path = data_rule.path;
						let name = data_rule.name;

						file_type
							.middleware(&path, meta, vfs)?
							.with_name(&name)
							.with_path(&path)
					};

					let source_snapshot = {
						let file_type = source_rule.file_type;
						let path = source_rule.path;
						let name = source_rule.name;

						file_type
							.middleware(&path, meta, vfs)?
							.with_name(&name)
							.with_path(&path)
							.with_data(data_snapshot)
							.apply_project_data(meta, &path)
					};

					Ok(Some(source_snapshot))
				}
				(Some(rule), None) | (None, Some(rule)) => {
					let file_type = rule.file_type;
					let path = rule.path;
					let name = rule.name;

					let snapshot = file_type
						.middleware(&path, meta, vfs)?
						.with_name(&name)
						.with_path(&path)
						.apply_project_data(meta, &path);

					Ok(Some(snapshot))
				}
				_ => unreachable!(),
			}
		} else {
			Ok(None)
		}
	// Get snapshot of directory that contains child source or data
	} else if let Some(resolved) = meta.sync_rules.iter().find_map(|rule| rule.resolve_child(path)) {
		vfs.watch(path)?;

		let file_type = resolved.file_type;
		let resolved_path = resolved.path;
		let name = resolved.name;

		let mut snapshot = file_type
			.middleware(&resolved_path, meta, vfs)?
			.with_path(path)
			.apply_project_data(meta, path);

		if file_type != FileType::Project {
			snapshot.set_name(&name);

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

fn resolve_relevant_rules(path: &Path, meta: &Meta, vfs: &Vfs) -> Option<RelevantRules> {
	let mut source_resolved_rule = None;
	let mut data_resolved_rule = None;

	let resolved_rule = meta.sync_rules.iter().find_map(|rule| rule.resolve(path))?;

	if resolved_rule.file_type == FileType::InstanceData {
		for rule in &meta.sync_rules {
			if rule.file_type == FileType::InstanceData {
				continue;
			}

			if let Some(full_name) = rule.full_name(&resolved_rule.name) {
				let path = path.parent().unwrap().join(full_name);

				if let Some(resolved) = rule.resolve(&path) {
					if vfs.exists(&resolved.path) {
						source_resolved_rule = Some(resolved);
						break;
					}
				}
			}
		}

		data_resolved_rule = Some(resolved_rule);
	} else {
		if let Some(data_rule) = meta
			.sync_rules
			.iter()
			.find(|rule| rule.file_type == FileType::InstanceData)
		{
			if let Some(data_name) = data_rule.full_name(&resolved_rule.name) {
				let path = path.parent().unwrap().join(data_name);

				if vfs.exists(&path) {
					data_resolved_rule = data_rule.resolve(&path);
				}
			}
		}

		source_resolved_rule = Some(resolved_rule);
	}

	Some(RelevantRules {
		source_rule: source_resolved_rule,
		data_rule: data_resolved_rule,
	})
}

fn resolve_child_paths(path: &Path, meta: &Meta, vfs: &Vfs) -> Option<RelevantRules> {
	let mut source_resolved_rule = None;
	let mut data_resolved_rule = None;

	let resolved_rule = meta.sync_rules.iter().find_map(|rule| rule.resolve(path))?;

	// for rule in &meta.sync_rules {
	// 	let resolved_rule = if is_file {
	// 		rule.resolve_child(path)
	// 	} else {
	// 		rule.resolve(path)
	// 	};

	// 	if let Some(resolved) = resolved_rule {
	// 		if resolved.file_type == FileType::InstanceData {
	// 			data_path = Some(resolved.path);
	// 		} else {
	// 			source_path = Some(resolved.path);
	// 			break;
	// 		}
	// 	}
	// }

	Some(RelevantRules {
		source_rule: source_resolved_rule,
		data_rule: data_resolved_rule,
	})
}
