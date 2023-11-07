use env_logger::WriteStyle;
use log::{info, warn};
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

	let color_choice = cli.get_color_choice();
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

	logger::init(cli.verbose.log_level_filter(), color_choice);

	match installation {
		Ok(()) => info!("Argon installation verified successfully!"),
		Err(error) => warn!("Failed to verify Argon installation: {}", error),
	}

	match cli.main() {
		Ok(()) => info!("Successfully executed command!"),
		Err(error) => argon_error!("Command execution failed: {}", error),
	};
}
