use anyhow::{bail, Result};
use clap::{ArgAction, Parser};
use colored::Colorize;
use log::{info, LevelFilter};
use roblox_install::RobloxStudio;
use std::{
	env,
	path::PathBuf,
	process::{self, Command},
	sync::mpsc,
};

use crate::{
	argon_info,
	config::Config,
	core::Core,
	project::{self, Project},
	sessions, util,
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

	/// Optional session indentifier
	#[arg()]
	session_id: Option<String>,

	/// Build plugin and place it into plugins folder
	#[arg(short, long, action = ArgAction::SetTrue)]
	plugin: bool,

	/// Whether to build in XML format (.rbxlx or .rbxmx)
	#[arg(short, long, action = ArgAction::SetTrue)]
	xml: bool,

	/// Rebuild project every time files change
	#[arg(short, long, action = ArgAction::SetTrue)]
	watch: bool,

	/// Actually run Argon, used to spawn new process
	#[arg(short, long, action = ArgAction::SetTrue, hide = true)]
	run: bool,
}

impl Build {
	pub fn main(self, level_filter: LevelFilter) -> Result<()> {
		if self.watch && !self.run {
			return self.spawn(level_filter);
		}

		let config = Config::load();

		let project = self.project.unwrap_or_default();
		let project_path = project::resolve(project, &config.project_name)?;

		if !project_path.exists() {
			bail!("Project {} does not exist", project_path.to_str().unwrap().bold(),)
		}

		let project = Project::load(&project_path)?;

		let mut xml = self.xml;
		let mut path = if let Some(path) = self.output {
			let ext = util::get_file_ext(&path);

			if ext == "rbxlx" || ext == "rbxmx" {
				xml = true;
			} else if ext == "rbxl" || ext == "rbxm" {
				xml = false;
			}

			if ext.starts_with("rbxm") && project.is_place() {
				bail!("Cannot build model or plugin from place project")
			} else if ext.starts_with("rbxl") && !project.is_place() {
				bail!("Cannot build place from plugin or model project")
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

			let mut name = project.name.clone();
			let ext = if xml { ".rbxmx" } else { ".rbxm" };

			name.push_str(ext);
			path = plugins_path.join(name);
		}

		let mut core = Core::new(config, project)?;

		core.load_dom()?;
		core.build(&path, xml)?;

		argon_info!("Successfully built project: {}", project_path.to_str().unwrap().bold());

		if self.watch {
			sessions::add(self.session_id, None, None, process::id())?;

			let (sender, receiver) = mpsc::channel();

			core.watch(Some(sender));

			argon_info!("Watching for changes...");

			for _ in receiver {
				info!("Rebuilding project...");

				core.build(&path, xml)?;
			}
		}

		Ok(())
	}

	fn spawn(self, level_filter: LevelFilter) -> Result<()> {
		let program = env::current_exe().unwrap_or(PathBuf::from("argon"));

		let log_style = env::var("RUST_LOG_STYLE").unwrap_or("auto".to_string());
		let backtrace = env::var("RUST_BACKTRACE").unwrap_or("0".to_string());

		let verbosity = match level_filter {
			LevelFilter::Off => "-q",
			LevelFilter::Error => "",
			LevelFilter::Warn => "-v",
			LevelFilter::Info => "-vv",
			LevelFilter::Debug => "-vvv",
			LevelFilter::Trace => "-vvvv",
		};

		let mut args = vec![String::from("build")];

		if let Some(project) = self.project {
			args.push(project.to_str().unwrap().to_string())
		}

		if let Some(output) = self.output {
			args.push(output.to_str().unwrap().to_string())
		}

		if self.plugin {
			args.push(String::from("--plugin"))
		}

		if self.xml {
			args.push(String::from("--xml"))
		}

		if self.watch {
			args.push(String::from("--watch"))
		}

		if !verbosity.is_empty() {
			args.push(verbosity.to_string())
		}

		Command::new(program)
			.args(args)
			.arg("--run")
			.env("RUST_LOG_STYLE", log_style)
			.env("RUST_BACKTRACE", backtrace)
			.spawn()?;

		Ok(())
	}
}
