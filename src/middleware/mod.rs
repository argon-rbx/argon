use anyhow::Result;
use colored::Colorize;
use log::trace;
use rbx_dom_weak::{
	types::{Enum, Variant},
	ustr,
};
use serde::{Deserialize, Serialize};
use std::{
	fmt::{self, Display, Formatter},
	path::Path,
};

use self::data::DataSnapshot;
use crate::{
	argon_warn,
	constants::BLACKLISTED_PATHS,
	core::{
		meta::{Context, Source},
		snapshot::Snapshot,
	},
	ext::{PathExt, ResultExt},
	vfs::Vfs,
	Properties,
};

mod helpers;

pub mod csv;
pub mod data;
pub mod dir;
pub mod json;
pub mod json_model;
pub mod luau;
pub mod md;
pub mod msgpack;
pub mod project;
pub mod rbxm;
pub mod rbxmx;
pub mod toml;
pub mod txt;
pub mod yaml;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum Middleware {
	Project,
	InstanceData,

	ServerScript,
	ClientScript,
	ModuleScript,

	StringValue,
	RichStringValue,
	LocalizationTable,

	JsonModule,
	TomlModule,
	YamlModule,
	MsgpackModule,

	JsonModel,
	RbxmModel,
	RbxmxModel,
}

impl Display for Middleware {
	fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
		write!(f, "{:?}", self)
	}
}

impl Middleware {
	fn read(&self, path: &Path, context: &Context, vfs: &Vfs) -> Result<Snapshot> {
		match self {
			Middleware::Project => project::read_project(path, vfs),
			Middleware::InstanceData => unreachable!(),
			//
			Middleware::ServerScript | Middleware::ClientScript | Middleware::ModuleScript => {
				luau::read_luau(path, context, vfs, self.clone().into())
			}
			//
			Middleware::StringValue => txt::read_txt(path, vfs),
			Middleware::RichStringValue => md::read_md(path, vfs),
			Middleware::LocalizationTable => csv::read_csv(path, vfs),
			//
			Middleware::JsonModule => json::read_json(path, vfs),
			Middleware::TomlModule => toml::read_toml(path, vfs),
			Middleware::YamlModule => yaml::read_yaml(path, vfs),
			Middleware::MsgpackModule => msgpack::read_msgpack(path, vfs),
			//
			Middleware::JsonModel => json_model::read_json_model(path, vfs),
			Middleware::RbxmModel => rbxm::read_rbxm(path, vfs),
			Middleware::RbxmxModel => rbxmx::read_rbxmx(path, vfs),
		}
		.with_desc(|| {
			format!(
				"Failed to read {} at {}",
				self.to_string().bold(),
				path.display().to_string().bold()
			)
		})
	}

	pub fn write(&self, properties: Properties, path: &Path, vfs: &Vfs) -> Result<Properties> {
		match self {
			Middleware::ServerScript | Middleware::ClientScript | Middleware::ModuleScript => {
				luau::write_luau(properties, path, vfs)
			}
			Middleware::StringValue => txt::write_txt(properties, path, vfs),
			Middleware::LocalizationTable => csv::write_csv(properties, path, vfs),
			// TODO: Add support for other middleware
			_ => unimplemented!(),
		}
		.with_desc(|| {
			format!(
				"Failed to write {} at {}",
				self.to_string().bold(),
				path.display().to_string().bold()
			)
		})
	}

	pub fn from_class(class: &str, properties: Option<&mut Properties>) -> Option<Self> {
		// TODO: Implement matcher for detecting remaining middleware
		match class {
			"Script" => {
				if let Some(properties) = properties {
					if let Some(Variant::Enum(run_context)) = properties.remove(&ustr("RunContext")) {
						let run_context = run_context.to_u32();

						return Some(match run_context {
							1 => Middleware::ServerScript,
							2 => Middleware::ClientScript,
							_ => {
								// This is currently unreachable so we can handle it inefficiently just for safety
								properties.insert(ustr("RunContext"), Variant::Enum(Enum::from_u32(run_context)));

								Middleware::ServerScript
							}
						});
					}
				}

				Some(Middleware::ServerScript)
			}
			"LocalScript" => Some(Middleware::ClientScript),
			"ModuleScript" => Some(Middleware::ModuleScript),
			"StringValue" => Some(Middleware::StringValue),
			"LocalizationTable" => Some(Middleware::LocalizationTable),
			_ => None,
		}
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
			trace!("Snapshot of {} not created: no middleware matched", path.display());
			Ok(None)
		}
	} else {
		for path in vfs.read_dir(path)? {
			if let Some(snapshot) = new_snapshot_file_child(&path, context, vfs)? {
				return Ok(Some(snapshot));
			}
		}

		new_snapshot_dir(path, context, vfs)
	}
}

/// Create a snapshot of a regular file,
/// example: `foo/bar.luau`
fn new_snapshot_file(path: &Path, context: &Context, vfs: &Vfs) -> Result<Option<Snapshot>> {
	if let Some(resolved) = context.sync_rules().iter().find_map(|rule| rule.resolve(path)) {
		let middleware = resolved.middleware;
		let name = resolved.name;

		let mut snapshot = middleware.read(path, context, vfs)?;

		if middleware != Middleware::Project {
			snapshot.set_name(&name);
			snapshot.meta.set_context(context);
			snapshot.meta.set_source(Source::file(path));
		} else if snapshot.class == "Folder" && snapshot.children.is_empty() {
			return Ok(None);
		}

		if let Some(instance_data) = get_instance_data(&name, Some(&snapshot.class), path, context, vfs)? {
			snapshot.apply_data(instance_data);
		}

		Ok(Some(snapshot))
	} else {
		Ok(None)
	}
}

/// Create a snapshot of a directory that has a child source or data,
/// example: `foo/bar/init.luau`
fn new_snapshot_file_child(path: &Path, context: &Context, vfs: &Vfs) -> Result<Option<Snapshot>> {
	if path.contains(&[".src.luau"]) || path.contains(&[".src.lua"]) {
		argon_warn!(
			"Your project uses legacy {} files which won't be supported in the next versions of Argon. \
			Make sure to rename {} file to {} for future compatibility!",
			".src".bold(),
			path.to_string().bold(),
			path.to_string().replace(".src", "init").bold()
		);
	}

	if let Some(resolved) = context.sync_rules().iter().find_map(|rule| rule.resolve_child(path)) {
		let middleware = resolved.middleware;
		let name = resolved.name;
		let parent = path.get_parent();

		let mut snapshot = middleware.read(path, context, vfs)?;

		if middleware != Middleware::Project {
			snapshot.set_name(&name);
			snapshot.meta.set_context(context);
			snapshot.meta.set_source(Source::child_file(parent, path));

			for entry in vfs.read_dir(parent)? {
				if entry == path {
					continue;
				}

				if let Some(child_snapshot) = new_snapshot(&entry, context, vfs)? {
					snapshot.add_child(child_snapshot);
				}
			}
		} else if snapshot.class == "Folder" && snapshot.children.is_empty() {
			return Ok(None);
		}

		if let Some(instance_data) = get_instance_data(&name, Some(&snapshot.class), parent, context, vfs)? {
			snapshot.apply_data(instance_data);
		}

		Ok(Some(snapshot))
	} else {
		Ok(None)
	}
}

/// Create snapshot of a directory,
/// example: `foo/bar`
fn new_snapshot_dir(path: &Path, context: &Context, vfs: &Vfs) -> Result<Option<Snapshot>> {
	let mut snapshot = dir::read_dir(path, context, vfs)?;

	if let Some(instance_data) = get_instance_data(&snapshot.name, None, path, context, vfs)? {
		snapshot.apply_data(instance_data);
	}

	Ok(Some(snapshot))
}

fn get_instance_data(
	name: &str,
	class: Option<&str>,
	path: &Path,
	context: &Context,
	vfs: &Vfs,
) -> Result<Option<DataSnapshot>> {
	for sync_rule in context.sync_rules_of_type(&Middleware::InstanceData, false) {
		if let Some(data_path) = sync_rule.locate(path, name, vfs.is_dir(path)) {
			if vfs.exists(&data_path) {
				let data = data::read_data(&data_path, class, vfs).with_desc(|| {
					format!(
						"Failed to get instance data at {}",
						data_path.display().to_string().bold()
					)
				})?;

				return Ok(Some(data));
			}
		}
	}

	Ok(None)
}
