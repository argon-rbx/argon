use anyhow::Result;
use clap::Parser;
use reqwest::{blocking::Client, header::CONTENT_TYPE};
use serde::Serialize;
use std::{fs, path::MAIN_SEPARATOR};

use crate::{argon_error, argon_info, sessions};

/// Execute Luau code in Roblox Studio (requires running session)
#[derive(Parser)]
pub struct Exec {
	/// Luau code to execute (can be file path)
	#[arg()]
	code: String,

	/// Session indentifier
	#[arg()]
	session: Option<String>,

	/// Focus Roblox Studio window when executing code
	#[arg(short, long)]
	focus: bool,

	/// Launch Roblox Studio, run code and return the result
	#[arg(short, long)]
	standalone: bool,

	/// Server host name
	#[arg(short = 'H', long)]
	host: Option<String>,

	/// Server port
	#[arg(short = 'P', long)]
	port: Option<u16>,
}

impl Exec {
	pub fn main(self) -> Result<()> {
		let code = if self.is_path() {
			fs::read_to_string(self.code)?
		} else {
			self.code
		};

		if self.standalone {
			// TODO: Implement standalone mode
			argon_error!("Standalone mode is not implemented yet!");
		} else if let Some(session) = sessions::get(self.session, self.host, self.port)? {
			if let Some(address) = session.get_address() {
				let url = format!("{}/exec", address);

				let body = rmp_serde::to_vec(&Request {
					code: code.to_owned(),
					focus: self.focus,
				})?;

				let response = Client::default()
					.post(url)
					.header(CONTENT_TYPE, "application/msgpack")
					.body(body)
					.send();

				match response {
					Ok(_) => argon_info!("Code executed successfully!"),
					Err(err) => argon_error!("Code execution failed: {}", err),
				}
			} else {
				argon_error!("Code execution failed: running session does not have an address");
			}
		} else {
			argon_error!("Code execution failed: no running session was found");
		}

		Ok(())
	}

	fn is_path(&self) -> bool {
		if self.code.contains('\n') {
			return false;
		}

		if !self.code.contains(MAIN_SEPARATOR) {
			return false;
		}

		true
	}
}

#[derive(Serialize)]
struct Request {
	code: String,
	focus: bool,
}
