use anyhow::Result;
use clap::{ColorChoice, Parser, Subcommand};
use clap_verbosity_flag::Verbosity;
use env_logger::fmt::WriteStyle;
use std::env;

mod config;
mod init;
mod run;
mod stop;
mod test;

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
	pub command: Commands,

	#[command(flatten)]
	pub verbose: Verbosity,

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

	pub fn get_color_choice(&self) -> WriteStyle {
		let log_style = env::var("RUST_LOG_STYLE");

		if log_style.is_ok() {
			return match log_style.unwrap().as_str() {
				"always" => WriteStyle::Always,
				"never" => WriteStyle::Never,
				_ => WriteStyle::Auto,
			};
		}

		match self.color {
			ColorChoice::Always => WriteStyle::Always,
			ColorChoice::Never => WriteStyle::Never,
			_ => WriteStyle::Auto,
		}
	}

	pub fn main(self) -> Result<()> {
		match self.command {
			Commands::Run(command) => command.main(self.verbose.log_level_filter()),
			Commands::Test(command) => command.main(),
			Commands::Stop(command) => command.main(),
			Commands::Config(command) => command.main(),
			Commands::Init(command) => command.main(),
		}
	}
}

#[derive(Subcommand)]
pub enum Commands {
	Run(run::Run),
	Stop(stop::Stop),
	Test(test::Test),
	Config(config::Config),
	Init(init::Init),
}
