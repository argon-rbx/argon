use anyhow::Result;
use awc::Client;
use clap::{ArgAction, Parser};

use crate::{argon_info, argon_warn, sessions, util};

/// Stop Argon session by port or all running sessions
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
	#[arg(short, long, action = ArgAction::SetTrue)]
	all: bool,
}

impl Stop {
	fn kill_process(pid: u32) {
		util::kill(pid);
		argon_info!("Killed Argon process {}", pid.to_string())
	}

	#[actix_web::main]
	async fn make_request(client: &Client, address: &String, pid: u32) {
		let mut url = String::from("http://");
		url.push_str(address);
		url.push_str("/stop");

		match client.post(url).send().await {
			Err(_) => {
				Self::kill_process(pid);
			}
			Ok(_) => argon_info!("Stopped Argon session {}", address),
		}
	}

	pub fn main(self) -> Result<()> {
		if self.all {
			let sessions = sessions::get_all()?;

			if sessions.is_none() {
				argon_warn!("There are no running sessions");
				return Ok(());
			}

			let client = Client::default();

			for (_, session) in sessions.unwrap() {
				if let Some(address) = session.get_address() {
					Stop::make_request(&client, &address, session.pid);
				} else {
					Stop::kill_process(session.pid);
				}
			}

			return sessions::remove_all();
		}

		let session = sessions::get(self.session, self.host, self.port)?;

		if let Some(session) = session {
			if let Some(address) = session.get_address() {
				let client = Client::default();
				Stop::make_request(&client, &address, session.pid);
			} else {
				Stop::kill_process(session.pid);
			}

			sessions::remove(&session)
		} else {
			argon_warn!("There is no running session on this address");
			Ok(())
		}
	}
}
