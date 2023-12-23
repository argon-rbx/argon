use anyhow::{bail, Result};
use clap::{ArgAction, Parser};
use colored::Colorize;
use roblox_install::RobloxStudio;
use std::path::PathBuf;

use crate::{
	config::Config,
	core::Core,
	project::{self, Project},
	utils,
};

/// Build project into Roblox place or model
#[derive(Parser)]
pub struct Build {
	/// Project path
	#[arg()]
	project: Option<PathBuf>,

	/// Output path
	#[arg()]
	output: Option<PathBuf>,

	/// Build plugin and place it into plugins folder
	#[arg(short, long, action = ArgAction::SetTrue)]
	plugin: bool,

	/// Whether to build in XML format (.rbxlx or .rbxmx)
	#[arg(short, long, action = ArgAction::SetTrue)]
	xml: bool,

	/// Rebuild project every time files change (TODO)
	#[arg(short, long, action = ArgAction::SetTrue)]
	watch: bool,
}

impl Build {
	pub fn main(self) -> Result<()> {
		let config = Config::load();

		let project = self.project.unwrap_or_default();
		let project_path = project::resolve(project, &config.project_name)?;

		if !project_path.exists() {
			bail!("Project {} does not exist", project_path.to_str().unwrap().bold(),)
		}

		let project = Project::load(&project_path)?;

		let mut xml = self.xml;
		let mut path = if let Some(path) = self.output {
			let ext = utils::get_file_ext(&path);

			if ext == "rbxlx" || ext == "rbxmx" {
				xml = true;
			} else if ext == "rbxl" || ext == "rbxm" {
				xml = false;
			}

			if ext.starts_with("rbxm") && project.is_place() {
				bail!("Cannot build model or plugin from place project")
			}

			path
		} else {
			let mut name = project.name.clone();

			let ext = if project.is_place() {
				if xml {
					".rbxlx"
				} else {
					".rbxl"
				}
			} else if xml {
				".rbxmx"
			} else {
				".rbxm"
			};

			name.push_str(ext);

			PathBuf::from(name)
		};

		if self.plugin {
			if project.is_place() {
				bail!("Cannot build plugin from place project")
			}

			let plugins_path = RobloxStudio::locate()?.plugins_path().to_owned();
			path = plugins_path.join(project.name.clone());
		}

		let mut core = Core::new(config, project)?;

		core.load_dom()?;
		core.build(&path, xml)?;

		Ok(())
	}
}
