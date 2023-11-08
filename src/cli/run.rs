use anyhow::{bail, Context, Result};
use clap::{ArgAction, Parser};
use log::LevelFilter;
use std::{
	env,
	path::PathBuf,
	process::{self, Command},
};

use crate::{argon_info, argon_warn, config::Config, project, server, session, workspace};

/// Run Argon, start local server and looking for file changes
#[derive(Parser)]
pub struct Run {
	/// Server host name [type: string, default: "localhost"]
	#[arg(short = 'H', long)]
	host: Option<String>,

	/// Server port [type: int, default: 8000]
	#[arg(short = 'P', long)]
	port: Option<u16>,

	/// Project path [type: path, default: ".argon"]
	#[arg()]
	project: Option<PathBuf>,

	/// Actually run Argon, used to spawn new process
	#[arg(short, long, action = ArgAction::SetTrue, hide = true)]
	run: bool,
}

impl Run {
	pub fn main(self, level_filter: LevelFilter) -> Result<()> {
		if !self.run {
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

			let mut args = vec![String::from("run")];

			if self.host.is_some() {
				args.push(String::from("--host"));
				args.push(self.host.unwrap())
			}

			if self.port.is_some() {
				args.push(String::from("--port"));
				args.push(self.port.unwrap().to_string());
			}

			if self.project.is_some() {
				args.push(
					self.project
						.unwrap()
						.to_str()
						.context("Project path contains invalid characters")?
						.to_string(),
				);
			}

			if verbosity != "" {
				args.push(verbosity.to_string())
			}

			Command::new(program)
				.args(args)
				.arg("--run")
				.env("RUST_LOG_STYLE", log_style)
				.env("RUST_BACKTRACE", backtrace)
				.spawn()?;

			return Ok(());
		}

		let config = Config::new();

		let host = self.host.unwrap_or(config.host);
		let port = self.port.unwrap_or(config.port);
		let project = self.project.unwrap_or(config.project);

		let project_resolved = project::resolve(project)?;
		let project_exists = project_resolved.exists();

		if !project_exists && config.auto_init {
			argon_warn!("Cannot find the project, creating new one!");
			workspace::init(&project_resolved, config.template)?;
		} else {
			bail!(
				"Project file does not exist in this directory: {}. Run `argon init` or enable `auto_init` setting.",
				project_resolved.to_str().unwrap()
			)
		}

		argon_info!(
			"Serving on: {}:{}, project: {}",
			host,
			port,
			project_resolved.to_str().unwrap()
		);

		// fs::watch().ok();
		// server::start(host.clone(), port.clone())?;

		session::add(host, port, process::id())
	}
}
