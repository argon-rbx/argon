use anyhow::Result;
use clap::Parser;
use colored::Colorize;
use reqwest::blocking::Client;

use crate::{argon_info, argon_warn, logger::Table, sessions, util};

/// Stop Argon session by address, ID or all running sessions
#[derive(Parser)]
pub struct Stop {
	/// Session identifier
	#[arg()]
	session: Vec<String>,

	/// Server host name
	#[arg(short = 'H', long)]
	host: Option<String>,

	/// Server port
	#[arg(short = 'P', long)]
	port: Option<u16>,

	/// Stop all running session
	#[arg(short, long)]
	all: bool,

	/// List all running session
	#[arg(short, long)]
	list: bool,
}

impl Stop {
	pub fn main(self) -> Result<()> {
		if self.list {
			let sessions = sessions::get_all()?;

			if sessions.is_empty() {
				argon_warn!("There are no running sessions");
				return Ok(());
			}

			let mut table = Table::new();
			table.set_header(vec!["ID", "Host", "Port", "PID"]);

			for (id, session) in sessions {
				table.add_row(vec![
					id,
					session.host.unwrap_or("None".into()),
					session.port.map(|p| p.to_string()).unwrap_or("None".into()),
					session.pid.to_string(),
				]);
			}

			argon_info!("All running sessions:\n\n{}", table);

			return Ok(());
		}

		if self.all {
			let sessions = sessions::get_all()?;

			if sessions.is_empty() {
				argon_warn!("There are no running sessions");
				return Ok(());
			}

			for (_, session) in sessions {
				if let Some(address) = session.get_address() {
					Self::make_request(&address, session.pid);
				} else {
					Self::kill_process(session.pid);
				}
			}

			return sessions::remove_all();
		}

		if self.session.is_empty() {
			if let Some(session) = sessions::get(None, self.host, self.port)? {
				if let Some(address) = session.get_address() {
					Self::make_request(&address, session.pid);
				} else {
					Self::kill_process(session.pid);
				}

				sessions::remove(&session)?;
			} else {
				argon_warn!("There is no matching session to stop");
			}
		} else {
			let sessions = sessions::get_multiple(&self.session)?;

			if sessions.is_empty() {
				argon_warn!("There are no running sessions with provided IDs");
			} else {
				for session in sessions.values() {
					if let Some(address) = session.get_address() {
						Self::make_request(&address, session.pid);
					} else {
						Self::kill_process(session.pid);
					}
				}

				sessions::remove_multiple(&self.session)?;
			}
		}

		Ok(())
	}

	fn make_request(address: &String, pid: u32) {
		let url = format!("{}/stop", address);

		match Client::new().post(url).send() {
			Ok(_) => argon_info!("Stopped Argon session with address: {}", address.bold()),
			Err(_) => {
				Self::kill_process(pid);
			}
		}
	}

	fn kill_process(pid: u32) {
		util::kill_process(pid);
		argon_info!("Stopped Argon process with PID: {}", pid.to_string().bold())
	}
}
