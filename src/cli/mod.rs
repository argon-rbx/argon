use anyhow::Result;
use clap::{ArgAction, ColorChoice, Parser, Subcommand};
use clap_verbosity_flag::Verbosity;
use env_logger::fmt::WriteStyle;
use std::env;

mod build;
mod config;
mod doc;
mod exec;
mod init;
mod run;
mod stop;
mod studio;

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

	/// Automatically answer to any prompts
	#[arg(short, long, global = true, action = ArgAction::SetTrue)]
	yes: bool,

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

	pub fn yes(&self) -> bool {
		self.yes
	}

	pub fn get_color_choice(&self) -> WriteStyle {
		if let Ok(log_style) = env::var("RUST_LOG_STYLE") {
			return match log_style.as_str() {
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
			Commands::Init(command) => command.main(),
			Commands::Run(command) => command.main(self.verbose.log_level_filter()),
			Commands::Stop(command) => command.main(),
			Commands::Build(command) => command.main(self.verbose.log_level_filter()),
			Commands::Studio(command) => command.main(),
			Commands::Exec(command) => command.main(),
			Commands::Config(command) => command.main(),
			Commands::Doc(command) => command.main(),
		}
	}
}

#[derive(Subcommand)]
pub enum Commands {
	Init(init::Init),
	Run(run::Run),
	Stop(stop::Stop),
	Build(build::Build),
	Studio(studio::Studio),
	Exec(exec::Exec),
	Config(config::Config),
	Doc(doc::Doc),
}
