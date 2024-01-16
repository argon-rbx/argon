use anyhow::Result;
use clap::Parser;
use colored::Colorize;
use log::trace;
use std::{
	env,
	path::PathBuf,
	process::{self, Command},
};

use crate::{
	argon_info, argon_warn,
	config::Config,
	core::Core,
	exit,
	program::{Program, ProgramKind},
	project::{self, Project},
	server::{self, Server},
	sessions, util,
};

/// Run Argon, start local server and looking for file changes
#[derive(Parser)]
pub struct Run {
	/// Project path
	#[arg()]
	project: Option<PathBuf>,

	/// Session indentifier
	#[arg()]
	session: Option<String>,

	/// Server host name
	#[arg(short = 'H', long)]
	host: Option<String>,

	/// Server port
	#[arg(short = 'P', long)]
	port: Option<u16>,

	/// Whether to run using roblox-ts
	#[arg(short, long)]
	ts: bool,

	/// Whether to run using Rojo namespace
	#[arg(short, long)]
	rojo: bool,

	/// Spawn the Argon child process
	#[arg(long, hide = true)]
	argon_spawn: bool,
}

impl Run {
	pub fn main(self) -> Result<()> {
		let mut config = Config::load();

		if !self.argon_spawn && config.spawn {
			return self.spawn();
		}

		let project_path = project::resolve(self.project.clone().unwrap_or_default(), &config.project_name)?;

		if !project_path.exists() {
			exit!(
				"Project {} does not exist. Run {} to create new one.",
				project_path.to_str().unwrap().bold(),
				"argon init".bold(),
			);
		}

		let project = Project::load(&project_path)?;

		let use_ts = self.ts || config.ts_mode || if config.auto_detect { project.is_ts() } else { false };
		let use_rojo =
			self.rojo || config.rojo_mode || use_ts || if config.auto_detect { project.is_rojo() } else { false };

		if use_ts {
			trace!("Starting roblox-ts");

			let working_dir = project_path.parent().unwrap();

			let child = Program::new(ProgramKind::Npx)
				.message("Failed to serve roblox-ts project")
				.current_dir(working_dir)
				.arg("rbxtsc")
				.arg("--watch")
				.spawn()?;

			if let Some(mut child) = child {
				util::handle_kill(move || {
					child.kill().ok();
				})?;
			} else {
				return Ok(());
			}
		}

		if use_rojo {
			config.make_rojo();
		}

		let mut core = Core::new(config.clone(), project)?;
		let host = self.host.unwrap_or(core.host());
		let mut port = self.port.unwrap_or(core.port());

		if !server::is_port_free(&host, port) {
			if config.scan_ports {
				let new_port = server::get_free_port(&host, port);

				argon_warn!("Port {} is already in use, using {} instead!", port, new_port);

				port = new_port;
			} else {
				exit!(
					"Port {} is already in use! Enable {} setting to use first available port automatically.",
					port,
					"scan_ports".bold()
				);
			}
		}

		core.load_dom()?;

		let server = Server::new(core, &host, &port);

		if config.spawn {
			sessions::add(self.session, Some(host.clone()), Some(port), process::id())?;
		}

		argon_info!(
			"Running on: {}:{}, project: {}",
			host,
			port,
			project_path.to_str().unwrap()
		);

		server.start()?;

		Ok(())
	}

	fn spawn(self) -> Result<()> {
		let program = env::current_exe().unwrap_or(PathBuf::from("argon"));

		let log_style = env::var("RUST_LOG_STYLE").unwrap_or("auto".to_string());
		let backtrace = env::var("RUST_BACKTRACE").unwrap_or("0".to_string());

		let mut args = vec![String::from("run"), util::get_verbosity_flag()];

		if let Some(host) = self.host {
			args.push(String::from("--host"));
			args.push(host)
		}

		if let Some(port) = self.port {
			args.push(String::from("--port"));
			args.push(port.to_string());
		}

		if let Some(project) = self.project {
			args.push(project.to_str().unwrap().to_string());
		}

		if let Some(session) = self.session {
			args.push(session);
		}

		if self.ts {
			args.push(String::from("--ts"));
		}

		Command::new(program)
			.args(args)
			.arg("--yes")
			.arg("--argon-spawn")
			.env("RUST_LOG_STYLE", log_style)
			.env("RUST_BACKTRACE", backtrace)
			.spawn()?;

		Ok(())
	}
}
