use anyhow::Result;
use clap::Parser;
use colored::Colorize;
use reqwest::blocking::Client;

use crate::{argon_info, argon_warn, logger::Table, sessions, util};

/// Stop Argon session by address, ID or all running sessions
#[derive(Parser)]
pub struct Stop {
	/// Session indentifier
	#[arg()]
	session: Option<String>,

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

			if sessions.is_none() {
				argon_warn!("There are no running sessions");
				return Ok(());
			}

			let mut table = Table::new();
			table.set_header(vec!["ID", "Host", "Port", "PID"]);

			for (id, session) in sessions.unwrap() {
				let port = if let Some(port) = session.port {
					port.to_string()
				} else {
					String::from("None")
				};

				table.add_row(vec![
					id,
					session.host.unwrap_or(String::from("None")),
					port,
					session.pid.to_string(),
				]);
			}

			argon_info!("All running sessions:\n\n{}", table);

			return Ok(());
		}

		if self.all {
			let sessions = sessions::get_all()?;

			if sessions.is_none() {
				argon_warn!("There are no running sessions");
				return Ok(());
			}

			for (_, session) in sessions.unwrap() {
				if let Some(address) = session.get_address() {
					Self::make_request(&address, session.pid);
				} else {
					Self::kill_process(session.pid);
				}
			}

			return sessions::remove_all();
		}

		let session = sessions::get(self.session, self.host, self.port)?;

		if let Some(session) = session {
			if let Some(address) = session.get_address() {
				Self::make_request(&address, session.pid);
			} else {
				Self::kill_process(session.pid);
			}

			sessions::remove(&session)
		} else {
			argon_warn!("There is no running session on this address");
			Ok(())
		}
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
