use anyhow::{bail, Result};
use clap::Parser;
use colored::Colorize;
use log::{info, trace, LevelFilter};
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
	program::{self, Program},
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

	/// Session indentifier
	#[arg()]
	session: Option<String>,

	/// Build plugin and place it into plugins folder
	#[arg(short, long)]
	plugin: bool,

	/// Whether to build in XML format (.rbxlx or .rbxmx)
	#[arg(short, long)]
	xml: bool,

	/// Whether to build using roblox-ts
	#[arg(short, long)]
	ts: bool,

	/// Rebuild project every time files change
	#[arg(short, long)]
	watch: bool,

	/// Spawn the Argon child process
	#[arg(short, long, hide = true)]
	spawn: bool,
}

impl Build {
	pub fn main(self, level_filter: LevelFilter) -> Result<()> {
		if self.watch && !self.spawn {
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
			let ext = if project.is_place() {
				if xml {
					"rbxlx"
				} else {
					"rbxl"
				}
			} else if xml {
				"rbxmx"
			} else {
				"rbxm"
			};

			PathBuf::from(format!("{}.{}", project.name, ext))
		};

		if self.plugin {
			if project.is_place() {
				bail!("Cannot build plugin from place project")
			}

			let plugins_path = RobloxStudio::locate()?.plugins_path().to_owned();
			let ext = if xml { "rbxmx" } else { "rbxm" };

			path = plugins_path.join(format!("{}.{}", project.name, ext));
		}

		if self.ts {
			argon_info!("Compiling TypeScript files...");

			let mut working_dir = project_path.clone();
			working_dir.pop();

			let child = program::spawn(
				Command::new(program::NPX)
					.current_dir(&working_dir)
					.arg("rbxtsc")
					.arg("build")
					.spawn(),
				Program::Npm,
				"Failed to start roblox-ts",
			)?;

			if let Some(mut child) = child {
				child.wait()?;
			} else {
				return Ok(());
			}
		}

		let mut core = Core::new(config, project)?;

		core.load_dom()?;
		core.build(&path, xml)?;

		argon_info!("Successfully built project: {}", project_path.to_str().unwrap().bold());

		if self.watch {
			if self.ts {
				trace!("Starting roblox-ts");

				let mut working_dir = project_path.clone();
				working_dir.pop();

				let mut child = Command::new(program::NPX)
					.current_dir(&working_dir)
					.arg("rbxtsc")
					.arg("--watch")
					.spawn()?;

				util::handle_kill(move || {
					child.kill().ok();
				})?;
			}

			sessions::add(self.session, None, None, process::id())?;

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
			args.push(util::path_to_string(&project))
		}

		if let Some(output) = self.output {
			args.push(util::path_to_string(&output))
		}

		if self.plugin {
			args.push(String::from("--plugin"))
		}

		if self.xml {
			args.push(String::from("--xml"))
		}

		if self.ts {
			args.push(String::from("--ts"))
		}

		if self.watch {
			args.push(String::from("--watch"))
		}

		if !verbosity.is_empty() {
			args.push(verbosity.to_string())
		}

		Command::new(program)
			.args(args)
			.arg("--yes")
			.arg("--spawn")
			.env("RUST_LOG_STYLE", log_style)
			.env("RUST_BACKTRACE", backtrace)
			.spawn()?;

		Ok(())
	}
}
