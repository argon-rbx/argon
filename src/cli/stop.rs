use crate::{argon_error, argon_warn, session};
use awc::Client;
use clap::{ArgAction, Parser};
use log::trace;

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
	#[actix_web::main]
	async fn make_request(&self, client: &Client, address: &String, id: &u32) {
		let mut url = String::from("http://");
		url.push_str(address);
		url.push_str("/stop");

		let result = client.post(url).send().await;

		match result {
			Err(error) => {
				argon_error!("Failed to stop Argon session: {}", error);
				argon_warn!("You might wanna stop it manually using session's PID: {}", id)
			}
			Ok(_) => trace!("Stopped Argon session {}", address),
		}
	}

	pub fn run(self) {
		if self.all.unwrap_or_default() {
			let sessions = session::get_all();

			if sessions.is_none() {
				argon_warn!("There are no running sessions");
				return;
			}

			let client = Client::default();

			for session in sessions.unwrap().iter() {
				let (address, id) = session;

				self.make_request(&client, &address, &id);
			}

			match session::remove_all() {
				Err(error) => argon_error!("Failed to clear session data: {}", error),
				Ok(()) => trace!("Cleared session data"),
			}

			return;
		}

		let session = session::get(self.host.clone(), self.port);

		if session.is_none() {
			argon_warn!("There is no running session on this address");
			return;
		}

		let (address, id) = session.unwrap();
		let client = Client::default();

		self.make_request(&client, &address, &id);

		match session::remove(&address) {
			Err(error) => argon_error!("Failed to remove session {}: {}", address, error),
			Ok(()) => trace!("Removed session {}", address),
		}
	}
}
