use directories::UserDirs;
use log::{error, trace};
use std::{
	error::Error,
	fs,
	path::{Path, PathBuf},
};
use toml_edit::{table, value, Document};

use crate::{argon_error, confirm::prompt, unwrap_or_return};

fn get_session_dir() -> Result<PathBuf, Box<dyn Error>> {
	let user_dirs = unwrap_or_return!(UserDirs::new(), Err("Failed to get user directory!".into()));
	let home_dir = user_dirs.home_dir();
	let session_dir = home_dir.join(Path::new(".argon/session.toml"));

	Ok(session_dir)
}

fn write_session_template(dir: &PathBuf) -> Result<Document, Box<dyn Error>> {
	let mut document = Document::new();

	document["last_session"] = value("");
	document["sessions"] = table();

	fs::write(dir, document.to_string())?;

	Ok(document)
}

fn get_session_data(read_only: bool) -> Result<(Document, PathBuf), Box<dyn Error>> {
	let session_dir = get_session_dir()?;

	if session_dir.exists() {
		let session = fs::read_to_string(&session_dir)?;
		let session = session.parse::<Document>()?;

		return Ok((session, session_dir));
	}

	if read_only {
		return Err("Session data does not exist".into());
	}

	let document = write_session_template(&session_dir)?;

	Ok((document, session_dir))
}

pub fn add(host: String, port: u16, id: u32) {
	let session_data = get_session_data(false);

	match session_data {
		Err(error) => {
			error!("Failed to get session data: {error}");
			return;
		}
		Ok(_) => trace!("Got session data successfully!"),
	}

	let (mut document, session_dir) = session_data.unwrap();

	let mut last_sesstion = host;
	last_sesstion.push_str(":");
	last_sesstion.push_str(&port.to_string());

	document["last_session"] = value(&last_sesstion);
	document["sessions"][&last_sesstion] = value::<i64>(i64::from(id).into());

	match fs::write(session_dir, document.to_string()) {
		Err(error) => error!("Failed to write session data: {}", error),
		Ok(_) => trace!("Saved session data successfully!"),
	}
}

pub fn get(host: Option<String>, port: Option<u16>) -> Option<u32> {
	let session_data = get_session_data(true);

	match session_data {
		Err(error) => {
			error!("Failed to get session data: {error}");
			return None;
		}
		Ok(_) => trace!("Got session data successfully!"),
	}

	let (document, session_dir) = session_data.unwrap();

	if !document.contains_key("last_session")
		|| !document.contains_key("sessions")
		|| !document["last_session"].is_str()
		|| !document["sessions"].is_table()
	{
		argon_error!("Session data file is corrupted!");

		let fix_file = prompt("Would you like to fix this issue by making new session file?", true);

		if fix_file.unwrap_or(false) {
			match write_session_template(&session_dir) {
				Err(error) => error!("Failed to fix corrupted session file: {}", error),
				Ok(_) => trace!("Fixed corrupted session file successfully!"),
			}
		}

		return None;
	}

	let last_session = document["last_session"].as_str().unwrap();
	let sessions = document["sessions"].as_table().unwrap();

	if host.is_none() && port.is_none() {
		if sessions.contains_key(&last_session) && sessions[&last_session].is_integer() {
			let id = sessions[&last_session].as_integer().unwrap();

			return Some(id as u32);
		}
	} else if host.is_some() && port.is_some() {
		let mut session = host.unwrap();
		session.push_str(":");
		session.push_str(&port.unwrap().to_string());

		if sessions.contains_key(&session) && sessions[&session].is_integer() {
			let id = sessions[&session].as_integer().unwrap();

			return Some(id as u32);
		}
	} else {
		let key: String;

		if port.is_some() {
			key = port.unwrap().to_string();
		} else {
			key = host.unwrap();
		}

		let mut sessions_vec: Vec<&str> = vec![];

		for session in sessions.iter() {
			sessions_vec.push(session.0);
		}

		for session in sessions_vec.iter().rev() {
			if session.contains(&key) {
				let id = &sessions[session];

				if id.is_integer() {
					return Some(id.as_integer().unwrap() as u32);
				}

				break;
			}
		}
	}

	return None;
}

pub fn get_all() -> Option<Vec<u32>> {
	let session_data = get_session_data(true);

	match session_data {
		Err(error) => {
			error!("Failed to get session data: {error}");
			return None;
		}
		Ok(_) => trace!("Got session data successfully!"),
	}

	let (document, session_dir) = session_data.unwrap();

	if !document.contains_key("sessions") || !document["sessions"].is_table() {
		argon_error!("Session data file is corrupted!");

		let fix_file = prompt("Would you like to fix this issue by making new session file?", true);

		if fix_file.unwrap_or(false) {
			match write_session_template(&session_dir) {
				Err(error) => error!("Failed to fix corrupted session file: {}", error),
				Ok(_) => trace!("Fixed corrupted session file successfully!"),
			}
		}

		return None;
	}

	let sessions = document["sessions"].as_table().unwrap();

	if sessions.len() == 0 {
		return None;
	}

	let mut sessions_vec: Vec<u32> = vec![];

	for session in sessions.iter() {
		if session.1.is_integer() {
			let id = session.1.as_integer().unwrap();

			sessions_vec.push(id as u32);
		}
	}

	Some(sessions_vec)
}

pub fn remove(id: u32) {
	let session_data = get_session_data(true);

	match session_data {
		Err(error) => {
			error!("Failed to get session data: {error}");
			return;
		}
		Ok(_) => trace!("Got session data successfully!"),
	}

	let (mut document, session_dir) = session_data.unwrap();

	if !document.contains_key("last_session")
		|| !document.contains_key("sessions")
		|| !document["last_session"].is_str()
		|| !document["sessions"].is_table()
	{
		argon_error!("Session data file is corrupted!");

		let fix_file = prompt("Would you like to fix this issue by making new session file?", true);

		if fix_file.unwrap_or(false) {
			match write_session_template(&session_dir) {
				Err(error) => error!("Failed to fix corrupted session file: {}", error),
				Ok(_) => trace!("Fixed corrupted session file successfully!"),
			}
		}

		return;
	}

	let last_session = document["last_session"].as_str().unwrap().to_owned();
	let sessions = document["sessions"].as_table_mut().unwrap();

	let mut last_session_address = "";

	for session in sessions.clone().iter() {
		if session.1.is_integer() {
			let session_id = session.1.as_integer().unwrap();
			let session_address = session.0;

			if id == session_id as u32 {
				sessions.remove(session_address);

				if session_address == last_session {
					document["last_session"] = value(last_session_address);
				}

				break;
			}

			last_session_address = session_address;
		}
	}

	match fs::write(session_dir, document.to_string()) {
		Err(error) => error!("Failed to write session data: {}", error),
		Ok(_) => trace!("Saved session data successfully!"),
	}
}

pub fn remove_all() {
	let session_dir = get_session_dir();

	match session_dir {
		Err(error) => {
			error!("Failed to get session directory: {error}");
			return;
		}
		Ok(_) => trace!("Got session directory successfully!"),
	}

	match write_session_template(&session_dir.unwrap()) {
		Err(error) => error!("Failed to clear session data: {error}"),
		Ok(_) => trace!("Session data cleared successfully!"),
	}
}
