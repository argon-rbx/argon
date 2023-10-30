use crate::{argon_error, argon_warn, session};
use clap::{ArgAction, Parser};
use sysinfo::{Pid, ProcessExt, System, SystemExt};

/// Stop Argon session by port or all running sessions
#[derive(Parser)]
pub struct Command {
	/// Server host name [type: string]
	#[arg(short = 'H', long)]
	host: Option<String>,

	/// Server port [type: int]
	#[arg(short, long)]
	port: Option<u16>,

	/// Stop all running session
	#[arg(short, long, action = ArgAction::SetTrue)]
	all: Option<bool>,
}

impl Command {
	pub fn run(self) {
		if !self.all.unwrap_or(false) {
			let id = session::get(self.host, self.port);

			if id.is_none() {
				argon_warn!("There is no running session with this host or port!");
				return;
			}

			let system = System::new_all();

			if let Some(process) = system.process(Pid::from(id.unwrap() as usize)) {
				let killed = process.kill();

				if !killed {
					argon_error!("Failed to kill Argon process");
				}
			} else {
				argon_error!("Failed to get process id!");
			}

			session::remove(id.unwrap());
		} else {
			let ids = session::get_all();

			if ids.is_none() {
				argon_warn!("There are no running sessions!");
				return;
			}

			let system = System::new_all();

			for id in ids.unwrap().iter() {
				if let Some(process) = system.process(Pid::from(*id as usize)) {
					let killed = process.kill();

					if !killed {
						argon_error!("Failed to kill Argon process");
					}
				} else {
					argon_error!("Failed to get process id!");
				}
			}

			session::remove_all();
		}
	}
}
