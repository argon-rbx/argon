use anyhow::Result;
use clap::Parser;
use colored::Colorize;
use log::{debug, info};
use std::{path::PathBuf, process, sync::Arc, thread};

use crate::{
	argon_error, argon_info, argon_warn,
	config::Config,
	core::Core,
	exit,
	ext::PathExt,
	program::{Program, ProgramName},
	project::{self, Project},
	server::{self, Server},
	sessions,
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

	/// Generate sourcemap every time files change
	#[arg(short, long)]
	sourcemap: bool,

	/// Whether to run using roblox-ts
	#[arg(short, long)]
	ts: bool,

	/// Spawn the Argon child process
	#[arg(long, hide = true)]
	argon_spawn: bool,
}

impl Run {
	pub fn main(self) -> Result<()> {
		let config = Config::load();

		if !self.argon_spawn && config.spawn {
			return self.spawn();
		}

		let project_path = project::resolve(self.project.clone().unwrap_or_default())?;
		let sourcemap_path = {
			if self.sourcemap {
				Some(project_path.get_parent().join("sourcemap.json"))
			} else {
				None
			}
		};

		if !project_path.exists() {
			exit!(
				"No project files found in {}. Run {} to create new one.",
				project_path.get_parent().to_string().bold(),
				"argon init".bold(),
			);
		}

		let project = Project::load(&project_path)?;
		let use_ts = self.ts || config.ts_mode || if config.auto_detect { project.is_ts() } else { false };

		if use_ts {
			debug!("Starting roblox-ts");

			let working_dir = project_path.get_parent();

			let child = Program::new(ProgramName::Npx)
				.message("Failed to serve roblox-ts project")
				.current_dir(working_dir)
				.arg("rbxtsc")
				.arg("--watch")
				.spawn()?;

			if child.is_none() {
				return Ok(());
			}
		}

		let core = Core::new(project, true)?;
		let host = self.host.unwrap_or(core.host().unwrap_or(config.host.clone()));
		let mut port = self.port.unwrap_or(core.port().unwrap_or(config.port));

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

		let core = Arc::new(core);

		if let Some(path) = sourcemap_path {
			let core = core.clone();
			let queue = core.queue();

			queue.subscribe(1).unwrap();
			core.sourcemap(Some(path.clone()), false)?;

			argon_info!("Generated sourcemap in: {}", path.to_string().bold());

			thread::spawn(move || loop {
				let _message = queue.get(1).unwrap();

				info!("Regenerating sourcemap..");

				match core.sourcemap(Some(path.clone()), false) {
					Ok(()) => (),
					Err(err) => {
						argon_error!("Failed to regenerate sourcemap: {}", err);
					}
				}
			});
		}

		sessions::add(
			self.session,
			Some(host.clone()),
			Some(port),
			process::id(),
			config.spawn,
		)?;

		let server = Server::new(core, &host, port);

		argon_info!("Running on: {}:{}, project: {}", host, port, project_path.to_string());

		server.start()?;

		Ok(())
	}

	fn spawn(self) -> Result<()> {
		let mut args = vec![String::from("run")];

		if let Some(project) = self.project {
			args.push(project.to_string());
		}

		if let Some(session) = self.session {
			args.push(session);
		}

		if let Some(host) = self.host {
			args.push(String::from("--host"));
			args.push(host)
		}

		if let Some(port) = self.port {
			args.push(String::from("--port"));
			args.push(port.to_string());
		}

		if self.sourcemap {
			args.push(String::from("--sourcemap"));
		}

		if self.ts {
			args.push(String::from("--ts"));
		}

		Program::new(ProgramName::Argon).args(args).spawn()?;

		Ok(())
	}
}
