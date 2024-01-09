use env_logger::WriteStyle;
use log::{info, trace, warn};
use std::{
	env,
	io::{self, IsTerminal},
};

use argon::argon_error;
use argon::cli::Cli;
use argon::crash_handler;
use argon::installer;
use argon::logger;

fn main() {
	crash_handler::hook();

	let installation = installer::install();
	let cli = Cli::new();

	let log_level = cli.log_level();
	let color_choice = cli.color_choice();
	let yes = cli.yes();

	if color_choice == WriteStyle::Auto && io::stdin().is_terminal() {
		env::set_var("RUST_LOG_STYLE", "always");
	} else {
		env::set_var(
			"RUST_LOG_STYLE",
			match color_choice {
				WriteStyle::Always => "always",
				_ => "never",
			},
		)
	}

	if yes {
		env::set_var("ARGON_YES", "1");
	}

	env::set_var("ARGON_VERBOSITY", log_level.as_str());

	logger::init(log_level, color_choice);

	match installation {
		Ok(()) => info!("Argon installation verified successfully!"),
		Err(err) => warn!("Failed to verify Argon installation: {}", err),
	}

	match cli.main() {
		Ok(()) => trace!("Successfully executed command!"),
		Err(err) => argon_error!("Command execution failed: {}", err),
	};
}
