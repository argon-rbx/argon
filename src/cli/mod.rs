use anyhow::Result;
use clap::{ColorChoice, Parser, Subcommand};
use clap_verbosity_flag::Verbosity;
use env_logger::fmt::WriteStyle;
use log::LevelFilter;
use std::env;

use crate::util;

mod build;
mod config;
mod doc;
mod exec;
mod init;
mod plugin;
mod serve;
mod sourcemap;
mod stop;
mod studio;
mod update;

// Like `anyhow::bail!`, but exits gracefully, for CLI only!
#[macro_export]
macro_rules! exit {
    ($msg:literal $(,)?) => {
        $crate::argon_error!($msg);
		return Ok(());
    };
    ($err:expr $(,)?) => {
        $crate::argon_error!($err);
		return Ok(());
    };
    ($fmt:expr, $($arg:tt)*) => {
        $crate::argon_error!($fmt, $($arg)*);
		return Ok(());
    };
}

macro_rules! about {
	() => {
		concat!("Argon ", env!("CARGO_PKG_VERSION"))
	};
}

macro_rules! long_about {
	() => {
		concat!(
			"Argon ",
			env!("CARGO_PKG_VERSION"),
			"\n",
			env!("CARGO_PKG_DESCRIPTION"),
			"\n",
			"Made with <3 by ",
			env!("CARGO_PKG_AUTHORS")
		)
	};
}

#[derive(Parser)]
#[clap(about = about!(), long_about = long_about!(), version)]
pub struct Cli {
	#[command(subcommand)]
	command: Commands,

	#[command(flatten)]
	verbose: Verbosity,

	/// Automatically answer to any prompts
	#[arg(short, long, global = true)]
	yes: bool,

	/// Print full backtrace on panic
	#[arg(short = 'B', long, global = true)]
	backtrace: bool,

	#[arg(long, hide = true, global = true)]
	profile: bool,

	/// Output coloring: auto, always, never
	#[arg(
		long,
		short = 'C',
		global = true,
		value_name = "WHEN",
		default_value = "auto",
		hide_default_value = true,
		hide_possible_values = true
	)]
	pub color: ColorChoice,
}

impl Cli {
	pub fn new() -> Cli {
		Cli::parse()
	}

	pub fn profile(&self) -> bool {
		self.profile
	}

	pub fn yes(&self) -> bool {
		if env::var("RUST_YES").is_ok() {
			return util::get_yes();
		}

		self.yes
	}

	pub fn backtrace(&self) -> bool {
		if env::var("RUST_BACKTRACE").is_ok() {
			return util::get_backtrace();
		}

		self.backtrace
	}

	pub fn verbosity(&self) -> LevelFilter {
		if env::var("RUST_VERBOSE").is_ok() {
			return util::get_verbosity();
		}

		self.verbose.log_level_filter()
	}

	pub fn log_style(&self) -> WriteStyle {
		if env::var("RUST_LOG_STYLE").is_ok() {
			return util::get_log_style();
		}

		match self.color {
			ColorChoice::Always => WriteStyle::Always,
			ColorChoice::Never => WriteStyle::Never,
			_ => WriteStyle::Auto,
		}
	}

	pub fn main(self) -> Result<()> {
		match self.command {
			Commands::Init(command) => command.main(),
			Commands::Serve(command) => command.main(),
			Commands::Stop(command) => command.main(),
			Commands::Build(command) => command.main(),
			Commands::Sourcemap(command) => command.main(),
			Commands::Studio(command) => command.main(),
			Commands::Exec(command) => command.main(),
			Commands::Update(command) => command.main(),
			Commands::Plugin(command) => command.main(),
			Commands::Config(command) => command.main(),
			Commands::Doc(command) => command.main(),
		}
	}
}

#[derive(Subcommand)]
pub enum Commands {
	Init(init::Init),
	Serve(serve::Serve),
	Stop(stop::Stop),
	Build(build::Build),
	Sourcemap(sourcemap::Sourcemap),
	Studio(studio::Studio),
	Exec(exec::Exec),
	Update(update::Update),
	Plugin(plugin::Plugin),
	Config(config::Config),
	Doc(doc::Doc),
}
