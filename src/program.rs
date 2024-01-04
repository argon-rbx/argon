use anyhow::{bail, Result};
use colored::Colorize;
use std::{
	io::{Error, ErrorKind},
	process::{Child, Output},
};

use crate::{argon_error, logger};

pub enum Program {
	Git,
	Npm,
}

pub fn spawn(result: Result<Child, Error>, program: Program, error: &str) -> Result<Option<Child>> {
	match result {
		Ok(child) => Ok(Some(child)),
		Err(err) => {
			if err.kind() == ErrorKind::NotFound {
				argon_error!("{}", get_error(&program, error));

				if logger::prompt(get_prompt(&program), false) {
					open::that(get_link(&program))?;
				}

				Ok(None)
			} else {
				bail!("{}: {}", error, err)
			}
		}
	}
}

pub fn output(result: Result<Output, Error>, program: Program, error: &str) -> Result<Option<Output>> {
	match result {
		Ok(output) => Ok(Some(output)),
		Err(err) => {
			if err.kind() == ErrorKind::NotFound {
				argon_error!("{}", get_error(&program, error));

				if logger::prompt(get_prompt(&program), false) {
					open::that(get_link(&program))?;
				}

				Ok(None)
			} else {
				bail!("{}: {}", error, err)
			}
		}
	}
}

fn get_error(program: &Program, error: &str) -> String {
	match program {
		Program::Git => format!(
			"{}: Git is not installed. To suppress this message remove {} option or disable {} setting.",
			error,
			"--git".bold(),
			"use_git".bold()
		),
		Program::Npm => format!("{}: npm is not installed", error),
	}
}

fn get_prompt(program: &Program) -> &'static str {
	match program {
		Program::Git => "Do you want to install Git now?",
		Program::Npm => "Do you want to install npm now?",
	}
}

fn get_link(program: &Program) -> &'static str {
	match program {
		Program::Git => "https://git-scm.com/downloads",
		Program::Npm => "https://nodejs.org/en/download/",
	}
}
