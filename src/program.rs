use anyhow::{bail, Result};
use colored::Colorize;
use log::LevelFilter;
use std::{
	env,
	io::{Error, ErrorKind},
	path::{Path, PathBuf},
	process::{Child, Command, Output},
};

use crate::{argon_error, logger, util};

#[derive(PartialEq)]
pub enum ProgramKind {
	Argon,
	Git,
	Npm,
	Npx,
}

pub struct Program {
	program: ProgramKind,
	args: Vec<String>,
	current_dir: PathBuf,
	message: String,
}

impl Program {
	pub fn new(program: ProgramKind) -> Self {
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
		if self.program == ProgramKind::Argon {
			let mut command = Command::new(env::current_exe().unwrap_or(PathBuf::from("argon")));

			let log_style = env::var("RUST_LOG_STYLE").unwrap_or("auto".to_string());
			let backtrace = env::var("RUST_BACKTRACE").unwrap_or("0".to_string());

			command
				.args(self.args.clone())
				.arg("--yes")
				.arg("--argon-spawn")
				.env("RUST_LOG_STYLE", log_style)
				.env("RUST_BACKTRACE", backtrace);

			return command;
		};

		#[cfg(not(target_os = "windows"))]
		let program = match self.program {
			ProgramKind::Git => "git",
			ProgramKind::Npm => "npm",
			ProgramKind::Npx => "npx",
			ProgramKind::Argon => unreachable!(),
		};

		#[cfg(target_os = "windows")]
		let program = match self.program {
			ProgramKind::Git => "git",
			ProgramKind::Npm => "npm.cmd",
			ProgramKind::Npx => "npx.cmd",
			ProgramKind::Argon => unreachable!(),
		};

		let mut command = Command::new(program);
		command.current_dir(self.current_dir.clone()).args(self.args.clone());

		if util::get_verbosity() == LevelFilter::Off {
			command.stdout(std::process::Stdio::null());
			command.stderr(std::process::Stdio::null());
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
			ProgramKind::Git => format!(
				"{}: Git is not installed. To suppress this message remove {} option or disable {} setting.",
				error,
				"--git".bold(),
				"use_git".bold()
			),
			ProgramKind::Npm | ProgramKind::Npx => format!("{}: npm is not installed", error),
			ProgramKind::Argon => unreachable!(),
		}
	}

	fn get_prompt(&self) -> &'static str {
		match self.program {
			ProgramKind::Git => "Do you want to install Git now?",
			ProgramKind::Npm | ProgramKind::Npx => "Do you want to install npm now?",
			ProgramKind::Argon => unreachable!(),
		}
	}

	fn get_link(&self) -> &'static str {
		match self.program {
			ProgramKind::Git => "https://git-scm.com/downloads",
			ProgramKind::Npm | ProgramKind::Npx => "https://nodejs.org/en/download/",
			ProgramKind::Argon => unreachable!(),
		}
	}
}
