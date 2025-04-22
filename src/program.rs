use anyhow::{bail, Result};
use colored::Colorize;
use log::LevelFilter;
use std::{
	env,
	io::{Error, ErrorKind},
	path::{Path, PathBuf},
	process::{Child, Command, Output, Stdio},
};

use crate::{argon_error, config::Config, ext::WriteStyleExt, logger, util};

#[derive(PartialEq)]
pub enum ProgramName {
	Argon,
	Git,
	Npm,
	Npx,
	Wally,
}

pub struct Program {
	program: ProgramName,
	args: Vec<String>,
	current_dir: PathBuf,
	message: String,
}

impl Program {
	pub fn new(program: ProgramName) -> Self {
		Self {
			program,
			args: Vec::new(),
			current_dir: env::current_dir().unwrap(),
			message: String::from("Failed to start child process"),
		}
	}

	pub fn arg<S>(&mut self, arg: S) -> &mut Self
	where
		S: Into<String>,
	{
		let arg: String = arg.into();

		if arg.is_empty() {
			return self;
		}

		// npm create requires -- before the command
		if self.args.len() == 1 && self.program == ProgramName::Npm && Config::new().package_manager.as_str() == "npm" {
			self.args.push("--".into());
		}

		self.args.push(arg);
		self
	}

	pub fn args<I, S>(&mut self, args: I) -> &mut Self
	where
		I: IntoIterator<Item = S>,
		S: Into<String>,
	{
		for arg in args {
			self.arg(arg);
		}
		self
	}

	pub fn current_dir(&mut self, dir: &Path) -> &mut Self {
		dir.clone_into(&mut self.current_dir);
		self
	}

	pub fn message(&mut self, message: &str) -> &mut Self {
		message.clone_into(&mut self.message);
		self
	}

	pub fn spawn(&mut self) -> Result<Option<Child>> {
		let result = self.get_command().spawn();

		match result {
			Ok(child) => Ok(Some(child)),
			Err(err) => self.handle_error(err),
		}
	}

	pub fn output(&mut self) -> Result<Option<Output>> {
		let result = self.get_command().output();

		match result {
			Ok(output) => Ok(Some(output)),
			Err(err) => self.handle_error(err),
		}
	}

	fn get_command(&self) -> Command {
		if self.program == ProgramName::Argon {
			let mut command = Command::new(env::current_exe().unwrap_or(PathBuf::from("argon")));

			let verbosity = util::env_verbosity().as_str();
			let log_style = util::env_log_style().to_string();
			let backtrace = if util::env_backtrace() { "1" } else { "0" };
			let yes = if util::env_yes() { "1" } else { "0" };

			command
				.args(self.args.clone())
				.arg("--argon-spawn")
				.env("RUST_VERBOSE", verbosity)
				.env("RUST_LOG_STYLE", log_style)
				.env("RUST_BACKTRACE", backtrace)
				.env("RUST_YES", yes);

			return command;
		};

		let config = Config::new();
		let package_manager = config.package_manager.as_str();

		#[allow(unused_mut)]
		let mut program = match (&self.program, package_manager) {
			(ProgramName::Npm, _) => package_manager,
			(ProgramName::Npx, "npm") => "npx",
			(ProgramName::Npx, _) => package_manager,
			(ProgramName::Git, _) => "git",
			(ProgramName::Wally, _) => "wally",
			(ProgramName::Argon, _) => unreachable!(),
		}
		.to_owned();

		#[cfg(target_os = "windows")]
		if (self.program == ProgramName::Npm || self.program == ProgramName::Npx)
			&& Command::new(package_manager)
				.spawn()
				.is_err_and(|err| err.kind() == ErrorKind::NotFound)
		{
			program += ".cmd";
		}

		let mut command = Command::new(program);
		command.current_dir(&self.current_dir).args(&self.args);

		if util::env_verbosity() == LevelFilter::Off {
			command.stdout(Stdio::null());
			command.stderr(Stdio::null());
		}

		command
	}

	fn handle_error<T>(&self, error: Error) -> Result<Option<T>> {
		if error.kind() == ErrorKind::NotFound {
			argon_error!("{}", self.get_error(&self.message));

			if logger::prompt(&self.get_prompt(), false) {
				open::that(self.get_link())?;
			}

			Ok(None)
		} else {
			bail!("{}: {}", self.message, error)
		}
	}

	fn get_error(&self, error: &str) -> String {
		match self.program {
			ProgramName::Git => format!(
				"{}: {} is not installed. To suppress this message remove {} option or disable {} setting",
				error,
				"Git".bold(),
				"--git".bold(),
				"use_git".bold()
			),
			ProgramName::Npm | ProgramName::Npx => {
				format!(
					"{}: {} is not installed",
					error,
					Config::new().package_manager.as_str().bold()
				)
			}
			ProgramName::Wally => format!("{}: {} is not installed", error, "Wally"),
			ProgramName::Argon => unreachable!(),
		}
	}

	fn get_prompt(&self) -> String {
		let config = Config::new();

		let program = match self.program {
			ProgramName::Git => "Git",
			ProgramName::Npm | ProgramName::Npx => &config.package_manager,
			ProgramName::Wally => "Wally",
			ProgramName::Argon => unreachable!(),
		};

		format!("Do you want to install {} now?", program.bold())
	}

	fn get_link(&self) -> String {
		match self.program {
			ProgramName::Git => "https://git-scm.com/downloads".into(),
			ProgramName::Npm | ProgramName::Npx => match Config::new().package_manager.as_str() {
				"yarn" => "https://yarnpkg.com/getting-started/install",
				"pnpm" => "https://pnpm.io/installation",
				"bun" => "https://bun.sh/docs/installation",
				"npm" => "https://nodejs.org/en/download/",
				package_manager => return format!("https://www.google.com/search?q={}", package_manager),
			}
			.to_owned(),
			ProgramName::Wally => "https://wally.run".into(),
			ProgramName::Argon => unreachable!(),
		}
	}
}
