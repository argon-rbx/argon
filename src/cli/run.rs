use crate::config::Config;
use crate::{argon_error, session};
use clap::Parser;
use log::{trace, LevelFilter};
use std::env;

/// Run Argon, start local server and looking for file changes
#[derive(Parser)]
pub struct Command {
	/// Server host name [type: string, default: "localhost"]
	#[arg(short = 'H', long)]
	host: Option<String>,

	/// Server port [type: int, default: "8000"]
	#[arg(short, long)]
	port: Option<u16>,
}

impl Command {
	pub fn run(self, level: LevelFilter) {
		let config = Config::new();

		let host = self.host.unwrap_or(config.host);
		let port = self.port.unwrap_or(config.port);

		let log_style = env::var("RUST_LOG_STYLE").unwrap_or("auto".to_string());
		let backtrace = env::var("RUST_BACKTRACE").unwrap_or("0".to_string());

		let verbosity = match level {
			LevelFilter::Off => "-q",
			LevelFilter::Error => "",
			LevelFilter::Warn => "-v",
			LevelFilter::Info => "-vv",
			LevelFilter::Debug => "-vvv",
			LevelFilter::Trace => "-vvvv",
		};

		let port_string = port.to_string();
		let mut args = vec!["serve", "--host", &host, "--port", &port_string];

		if verbosity != "" {
			args.push(verbosity)
		}

		let handle = std::process::Command::new("argon")
			.args(args)
			.env("RUST_LOG_STYLE", log_style)
			.env("RUST_BACKTRACE", backtrace)
			.spawn();

		match handle {
			Err(error) => {
				argon_error!("Failed to start new Argon process: {}", error);
				return;
			}
			Ok(_) => trace!("Started new Argon process"),
		}

		let session_result = session::add(host, port, handle.unwrap().id());

		match session_result {
			Err(error) => argon_error!("Failed to update session data: {}", error),
			Ok(()) => trace!("Saved session data"),
		}
	}
}
