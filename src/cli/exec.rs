use anyhow::Result;
use clap::Parser;
use reqwest::blocking::Client;
use serde_json::json;
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

		let session = sessions::get(self.session, self.host, self.port)?;

		if let Some(session) = session {
			if let Some(address) = session.get_address() {
				Self::make_request(&address, &code);
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

	fn make_request(address: &String, code: &str) {
		let url = format!("{}/exec", address);

		let data = json!({
			"code": code,
		});

		match Client::default().post(url).json(&data).send() {
			Ok(_) => argon_info!("Code executed successfully!"),
			Err(err) => {
				argon_error!("Code execution failed: {}", err);
			}
		}
	}
}
