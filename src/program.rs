use anyhow::{bail, Result};
use colored::Colorize;
use log::LevelFilter;
use std::{
	env,
	io::{Error, ErrorKind},
	path::{Path, PathBuf},
	process::{Child, Command, Output, Stdio},
};

use crate::{argon_error, ext::WriteStyleExt, logger, util};

#[derive(PartialEq)]
pub enum ProgramName {
	Argon,
	Git,
	Npm,
	Npx,
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
			args: vec![],
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
		self.current_dir = dir.to_owned();
		self
	}

	pub fn message(&mut self, message: &str) -> &mut Self {
		self.message = message.to_owned();
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

			let verbosity = util::get_verbosity().as_str();
			let log_style = util::get_log_style().to_string();
			let backtrace = if util::get_backtrace() { "1" } else { "0" };
			let yes = if util::get_yes() { "1" } else { "0" };

			command
				.args(self.args.clone())
				.arg("--argon-spawn")
				.env("RUST_VERBOSE", verbosity)
				.env("RUST_LOG_STYLE", log_style)
				.env("RUST_BACKTRACE", backtrace)
				.env("RUST_YES", yes);

			return command;
		};

		#[cfg(not(target_os = "windows"))]
		let program = match self.program {
			ProgramName::Git => "git",
			ProgramName::Npm => "npm",
			ProgramName::Npx => "npx",
			ProgramName::Argon => unreachable!(),
		};

		#[cfg(target_os = "windows")]
		let program = match self.program {
			ProgramName::Git => "git",
			ProgramName::Npm => "npm.cmd",
			ProgramName::Npx => "npx.cmd",
			ProgramName::Argon => unreachable!(),
		};

		let mut command = Command::new(program);
		command.current_dir(self.current_dir.clone()).args(self.args.clone());

		if util::get_verbosity() == LevelFilter::Off {
			command.stdout(Stdio::null());
			command.stderr(Stdio::null());
		}

		command
	}

	fn handle_error<T>(&self, error: Error) -> Result<Option<T>> {
		if error.kind() == ErrorKind::NotFound {
			argon_error!("{}", self.get_error(&self.message));

			if logger::prompt(self.get_prompt(), false) {
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
				"{}: Git is not installed. To suppress this message remove {} option or disable {} setting",
				error,
				"--git".bold(),
				"use_git".bold()
			),
			ProgramName::Npm | ProgramName::Npx => format!("{}: npm is not installed", error),
			ProgramName::Argon => unreachable!(),
		}
	}

	fn get_prompt(&self) -> &'static str {
		match self.program {
			ProgramName::Git => "Do you want to install Git now?",
			ProgramName::Npm | ProgramName::Npx => "Do you want to install npm now?",
			ProgramName::Argon => unreachable!(),
		}
	}

	fn get_link(&self) -> &'static str {
		match self.program {
			ProgramName::Git => "https://git-scm.com/downloads",
			ProgramName::Npm | ProgramName::Npx => "https://nodejs.org/en/download/",
			ProgramName::Argon => unreachable!(),
		}
	}
}
