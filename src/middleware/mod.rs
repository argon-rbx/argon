use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

use crate::{
	core::{
		meta::{Meta, ResolvedSyncRule},
		snapshot::Snapshot,
	},
	util,
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

#[derive(Debug, Clone)]
struct ResolvedPaths {
	pub source_path: Option<PathBuf>,
	pub data_path: Option<PathBuf>,
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
		println!("{:#?}", resolve_paths(meta, path, vfs));
		if let Some(resolved) = meta.sync_rules.iter().find_map(|rule| rule.resolve(path)) {
			let file_type = resolved.file_type;
			let resolved_path = resolved.path;
			let name = resolved.name;

			println!("{:#?}", name);

			// println!("{:#?}", path);
			// println!("{:#?}", resolve_pat hs(meta, path, false));
			// println!("{:#?}", "---------------------------------------");

			let snapshot = file_type
				.middleware(&resolved_path, meta, vfs)?
				.with_name(&name)
				.with_path(path)
				.apply_project_data(meta, path);

			Ok(Some(snapshot))
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

#[derive(Debug, Clone)]
struct ResolvedRules {
	pub source_rule: Option<ResolvedSyncRule>,
	pub data_rule: Option<ResolvedSyncRule>,
}

fn resolve_paths(meta: &Meta, path: &Path, vfs: &Vfs) -> Option<ResolvedRules> {
	let mut source_resolved_rule = None;
	let mut data_resolved_rule = None;

	let rule = meta.sync_rules.iter().find_map(|rule| rule.resolve(path))?;

	if rule.file_type == FileType::InstanceData {
		data_resolved_rule = Some(rule);
	} else {
		if let Some(data_rule) = meta
			.sync_rules
			.iter()
			.find(|rule| rule.file_type == FileType::InstanceData)
		{
			if let Some(data_name) = data_rule.full_name(&rule.name) {
				let path = path.parent().unwrap().join(data_name);

				if vfs.exists(&path) {
					data_resolved_rule = data_rule.resolve(&path);
				}
			}
		}

		source_resolved_rule = Some(rule);
	}

	Some(ResolvedRules {
		source_rule: source_resolved_rule,
		data_rule: data_resolved_rule,
	})
}

// fn resolve_child_paths(meta: &Meta, path: &Path, is_file: bool) -> Option<ResolvedPaths> {
// 	let mut source_path = None;
// 	let mut data_path = None;

// 	for rule in &meta.sync_rules {
// 		let resolved_rule = if is_file {
// 			rule.resolve_child(path)
// 		} else {
// 			rule.resolve(path)
// 		};

// 		if let Some(resolved) = resolved_rule {
// 			if resolved.file_type == FileType::InstanceData {
// 				data_path = Some(resolved.path);
// 			} else {
// 				source_path = Some(resolved.path);
// 				break;
// 			}
// 		}
// 	}

// 	if source_path.is_none() && data_path.is_none() {
// 		return None;
// 	}

// 	Some(ResolvedPaths { source_path, data_path })
// }
