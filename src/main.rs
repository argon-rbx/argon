use env_logger::WriteStyle;
use log::{debug, error, info, warn};
use puffin_http::Server;
use std::{
	env,
	io::{self, IsTerminal},
	mem::ManuallyDrop,
	thread,
};

use argon::{argon_error, cli::Cli, config::Config, crash_handler, installer, logger, stats, updater};

const PROFILER_ADDRESS: &str = "localhost:8888";

fn main() {
	crash_handler::hook();

	let config_state = Config::load();
	let config = Config::new();

	let is_aftman = installer::is_aftman();
	let installation = installer::verify(is_aftman, config.install_plugin);

	let cli = Cli::new();

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

	match config_state {
		Ok(()) => info!("Config file loaded"),
		Err(err) => warn!("Failed to load config file: {}", err),
	}

	match installation {
		Ok(()) => info!("Argon installation verified successfully!"),
		Err(err) => warn!("Failed to verify Argon installation: {}", err),
	}

	let handle = thread::spawn(move || {
		if !is_aftman && config.check_updates {
			match updater::check_for_updates(config.install_plugin, !config.auto_update) {
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

	match cli.main() {
		Ok(()) => debug!("Successfully executed command!"),
		Err(err) => argon_error!("Command execution failed: {}", err),
	};

	handle.join().ok();
	stats::save().ok();
}
