use clap::Parser;
use env_logger::WriteStyle;
use log::{debug, error, info, warn};
use puffin_http::Server;
use std::{
	env,
	io::{self, IsTerminal},
	mem::ManuallyDrop,
	process::ExitCode,
	thread,
};

use argon::{argon_error, cli::Cli, config::Config, crash_handler, installer, logger, stats, updater};

const PROFILER_ADDRESS: &str = "localhost:8888";

#[macro_use]
extern crate log;

mod cli;
mod config;
mod logger;
mod project;
mod server;
mod updater;
mod util;
mod vfs;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
	// Initialize file logging EARLY. Log to /tmp/argon_mcp.log (macOS/Linux)
	// On Windows, this might go to C:\tmp or fail if /tmp doesn't exist.
	// Consider using dirs crate for a platform-agnostic temp dir if needed.
	match simple_log::log_to_file("/tmp/argon_mcp.log", log::LevelFilter::Trace) {
		Ok(_) => {}
		Err(e) => eprintln!("!!! FAILED TO INITIALIZE FILE LOGGING: {} !!!", e),
	}
	// Log that we *tried* to initialize logging (this will go to the file if successful)
	log::info!("--- Argon process started, file logging initialized (or attempted) ---");

	crash_handler::hook();

	let config_kind = Config::load();
	let config = Config::new().clone();

	let is_managed = installer::is_managed();
	let installation = installer::verify(is_managed, config.install_plugin);

	let cli = cli::Cli::parse();
	log::trace!("CLI arguments parsed: {:?}", cli); // Example log

	let yes = cli.yes();
	let backtrace = cli.backtrace();
	let verbosity = cli.verbosity();
	let log_style = cli.log_style();

	if log_style == WriteStyle::Auto && io::stdin().is_terminal() {
		env::set_var("RUST_LOG_STYLE", "always");
	} else {
		env::set_var(
			"RUST_LOG_STYLE",
			match log_style {
				WriteStyle::Always => "always",
				_ => "never",
			},
		)
	}

	env::set_var("RUST_VERBOSE", verbosity.as_str());
	env::set_var("RUST_YES", if yes { "1" } else { "0" });
	env::set_var("RUST_BACKTRACE", if backtrace { "1" } else { "0" });

	logger::init(verbosity, log_style);

	match config_kind {
		Ok(kind) => info!("{:?} config loaded", kind),
		Err(err) => error!("Failed to load config file: {}", err),
	}

	match installation {
		Ok(()) => info!("Argon installation verified successfully!"),
		Err(err) => warn!("Failed to verify Argon installation: {}", err),
	}

	let handle = thread::spawn(move || {
		if !is_managed && config.check_updates {
			match updater::check_for_updates(config.install_plugin, config.update_templates, !config.auto_update) {
				Ok(()) => info!("Update check completed successfully!"),
				Err(err) => warn!("Update check failed: {}", err),
			}
		}

		if config.share_stats {
			match stats::track() {
				Ok(()) => info!("Stat tracker initialized successfully!"),
				Err(err) => warn!("Failed to initialize stat tracker: {}", err),
			}
		}
	});

	if cfg!(debug_assertions) && cli.profile() {
		match Server::new(PROFILER_ADDRESS) {
			Ok(server) => {
				let _ = ManuallyDrop::new(server);

				info!("Profiler started at {}", PROFILER_ADDRESS);
			}
			Err(err) => {
				error!("Failed to start profiler: {}", err);
			}
		}

		puffin::set_scopes_on(true);
	}

	let exit_code = match cli::handle_command(cli).await {
		Ok(()) => {
			debug!("Successfully executed command!");
			ExitCode::SUCCESS
		}
		Err(err) => {
			argon_error!("{}", err);
			ExitCode::FAILURE
		}
	};

	handle.join().ok();
	stats::save().ok();

	exit_code
}
