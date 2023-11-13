use anyhow::Result;
use awc::Client;
use clap::{ArgAction, Parser};
use log::trace;

use crate::{argon_error, argon_warn, session};

/// Stop Argon session by port or all running sessions
#[derive(Parser)]
pub struct Stop {
	/// Server host name [type: string]
	#[arg(short = 'H', long)]
	host: Option<String>,

	/// Server port [type: int]
	#[arg(short, long)]
	port: Option<u16>,

	/// Stop all running session
	#[arg(short, long, action = ArgAction::SetTrue)]
	all: bool,
}

impl Stop {
	#[actix_web::main]
	async fn make_request(client: &Client, address: &String, id: &u32) {
		let mut url = String::from("http://");
		url.push_str(address);
		url.push_str("/stop");

		let result = client.post(url).send().await;

		match result {
			Err(error) => {
				argon_error!("Failed to stop Argon session: {}", error);
				argon_warn!("You might wanna stop process manually using its PID: {}", id)
			}
			Ok(_) => trace!("Stopped Argon session {}", address),
		}
	}

	pub fn main(self) -> Result<()> {
		if self.all {
			let sessions = session::get_all();

			if sessions.is_none() {
				argon_warn!("There are no running sessions");
				return Ok(());
			}

			let client = Client::default();

			for session in sessions.unwrap().iter() {
				let (address, id) = session;

				Stop::make_request(&client, address, id);
			}

			return session::remove_all();
		}

		let session = session::get(self.host.clone(), self.port);

		if session.is_none() {
			argon_warn!("There is no running session on this address");
			return Ok(());
		}

		let (address, id) = session.unwrap();
		let client = Client::default();

		Stop::make_request(&client, &address, &id);

		session::remove(&address)
	}
}
