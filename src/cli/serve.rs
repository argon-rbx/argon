use anyhow::{bail, Result};
use clap::Parser;
use colored::Colorize;
use log::{debug, info};
use std::{path::PathBuf, process, sync::Arc, thread};

use crate::{
	argon_error, argon_info, argon_warn,
	config::Config,
	core::Core,
	ext::PathExt,
	integration,
	program::{Program, ProgramName},
	project::{self, Project},
	server::{self, Server},
	sessions,
};

/// Start local server and listen for file changes
#[derive(Parser)]
pub struct Serve {
	/// Project path
	#[arg()]
	project: Option<PathBuf>,

	/// Session identifier
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

	/// Run Argon asynchronously
	#[arg(short = 'A', long = "async")]
	run_async: bool,

	/// Spawn the Argon child process (internal)
	#[arg(long, hide = true)]
	argon_spawn: bool,
}

impl Serve {
	pub fn main(self) -> Result<()> {
		let project_path = project::resolve(self.project.clone().unwrap_or_default())?;

		Config::load_workspace(project_path.get_parent());
		let config = Config::new();

		if !self.argon_spawn && (self.run_async || config.run_async) {
			return self.spawn();
		}

		let sourcemap_path = if self.sourcemap || config.with_sourcemap {
			Some(project_path.with_file_name("sourcemap.json"))
		} else {
			None
		};

		if !project_path.exists() {
			bail!(
				"No project files found in {}. Run {} to create new one",
				project_path.get_parent().to_string().bold(),
				"argon init".bold(),
			);
		}

		let project = Project::load(&project_path)?;

		if !project.is_place() {
			bail!("Cannot serve non-place project!");
		}

		let use_wally = config.use_wally || (config.detect_project && project.is_wally());
		let use_ts = self.ts || config.ts_mode || (config.detect_project && project.is_ts());

		if use_wally {
			integration::check_wally_packages(&project.workspace_dir);
		}

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

				argon_warn!(
					"Port {} is already in use, using {} instead!",
					port.to_string().bold(),
					new_port.to_string().bold()
				);

				port = new_port;
			} else {
				bail!(
					"Port {} is already in use! Enable {} setting to use first available port automatically",
					port.to_string().bold(),
					"scan_ports".bold()
				);
			}
		}

		let core = Arc::new(core);

		if let Some(path) = sourcemap_path {
			let core = core.clone();
			let queue = core.queue();

			queue.subscribe_internal().unwrap();
			core.sourcemap(Some(path.clone()), false)?;

			argon_info!("Generated sourcemap at: {}", path.to_string().bold());

			thread::spawn(move || loop {
				let _message = queue.get_change(0).unwrap();

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
			config.run_async,
		)?;

		let server = Server::new(core, &host, port);

		argon_info!(
			"Serving on: {}, project: {}",
			server::format_address(&host, port).bold(),
			project_path.to_string().bold()
		);

		server.start()?;

		Ok(())
	}

	fn spawn(self) -> Result<()> {
		let mut args = vec![String::from("serve")];

		if let Some(project) = self.project {
			args.push(project.to_string());
		}

		if let Some(session) = self.session {
			args.push(session);
		}

		if let Some(host) = self.host {
			args.push("--host".into());
			args.push(host)
		}

		if let Some(port) = self.port {
			args.push("--port".into());
			args.push(port.to_string());
		}

		if self.sourcemap {
			args.push("--sourcemap".into());
		}

		if self.ts {
			args.push("--ts".into());
		}

		Program::new(ProgramName::Argon).args(args).spawn()?;

		Ok(())
	}
}
