use anyhow::Result;
use clap::{ColorChoice, Parser, Subcommand};
use clap_verbosity_flag::Verbosity;
use env_logger::fmt::WriteStyle;
use log::LevelFilter;
use std::env;

use crate::util;

mod build;
mod config;
mod debug;
mod doc;
mod exec;
mod init;
mod plugin;
mod serve;
mod sourcemap;
mod stop;
mod studio;
mod update;

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
			return util::env_yes();
		}

		self.yes
	}

	pub fn backtrace(&self) -> bool {
		if env::var("RUST_BACKTRACE").is_ok() {
			return util::env_backtrace();
		}

		self.backtrace
	}

	pub fn verbosity(&self) -> LevelFilter {
		if env::var("RUST_VERBOSE").is_ok() {
			return util::env_verbosity();
		}

		self.verbose.log_level_filter()
	}

	pub fn log_style(&self) -> WriteStyle {
		if env::var("RUST_LOG_STYLE").is_ok() {
			return util::env_log_style();
		}

		match self.color {
			ColorChoice::Always => WriteStyle::Always,
			ColorChoice::Never => WriteStyle::Never,
			_ => WriteStyle::Auto,
		}
	}

	pub fn main(self) -> Result<()> {
		match self.command {
			Commands::Init(command) => command.run(),
			Commands::Serve(command) => command.run(),
			Commands::Build(command) => command.run(),
			Commands::Sourcemap(command) => command.run(),
			Commands::Stop(command) => command.run(),
			Commands::Studio(command) => command.run(),
			Commands::Debug(command) => command.run(),
			Commands::Exec(command) => command.run(),
			Commands::Update(command) => command.run(),
			Commands::Plugin(command) => command.run(),
			Commands::Config(command) => command.run(),
			Commands::Doc(command) => command.run(),
		}
	}
}

#[derive(Subcommand)]
pub enum Commands {
	Init(init::Init),
	Serve(serve::Serve),
	Build(build::Build),
	Sourcemap(sourcemap::Sourcemap),
	Stop(stop::Stop),
	Studio(studio::Studio),
	Debug(debug::Debug),
	Exec(exec::Exec),
	Update(update::Update),
	Plugin(plugin::Plugin),
	Config(config::Config),
	Doc(doc::Doc),
}
