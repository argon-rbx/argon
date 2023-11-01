use directories::UserDirs;
use log::{error, trace, warn};
use std::{
	fs,
	path::{Path, PathBuf},
};
use toml_edit::{table, value, Document};

use crate::{unwrap_or_return, DynResult};

fn get_data_dir() -> DynResult<PathBuf> {
	let user_dirs = unwrap_or_return!(UserDirs::new(), Err("Failed to get user directory!".into()));
	let home_dir = user_dirs.home_dir();
	let data_dir = home_dir.join(Path::new(".argon/session.toml"));

	Ok(data_dir)
}

fn create_session_document(data_dir: &PathBuf) -> DynResult<Document> {
	let mut document = Document::new();

	document["last_session"] = value("");
	document["active_sessions"] = table();

	fs::write(data_dir, document.to_string())?;

	Ok(document)
}

fn get_session_data() -> DynResult<(Document, PathBuf)> {
	let data_dir = get_data_dir()?;

	if data_dir.exists() {
		let session = fs::read_to_string(&data_dir)?;
		let document = session.parse::<Document>()?;

		if document.contains_key("last_session")
			|| document.contains_key("active_sessions")
			|| document["last_session"].is_str()
			|| document["active_sessions"].is_table()
		{
			return Ok((document, data_dir));
		}

		warn!("Session data file is corrupted! Creating new one.");

		let document = create_session_document(&data_dir)?;

		return Ok((document, data_dir));
	}

	let document = create_session_document(&data_dir)?;

	Ok((document, data_dir))
}

pub fn add(host: String, port: u16, id: u32) -> DynResult<()> {
	let (mut document, data_dir) = get_session_data()?;

	let mut session = host;
	session.push_str(":");
	session.push_str(&port.to_string());

	document["last_session"] = value(&session);
	document["active_sessions"][&session] = value::<i64>(id as i64);

	fs::write(data_dir, document.to_string())?;

	Ok(())
}

pub fn get(host: Option<String>, port: Option<u16>) -> Option<(String, u32)> {
	let session_data = get_session_data();

	match session_data {
		Err(error) => {
			error!("Failed to get session data: {}", error);
			return None;
		}
		Ok(_) => trace!("Session data parsed"),
	}

	let (document, _) = session_data.unwrap();

	let last_session = document["last_session"].as_str().unwrap();
	let active_sessions = document["active_sessions"].as_table().unwrap();

	if host.is_none() && port.is_none() {
		if active_sessions.contains_key(&last_session) {
			let id = active_sessions[&last_session].as_integer()?;

			return Some((last_session.to_string(), id as u32));
		}
	} else if host.is_some() && port.is_some() {
		let mut session = host.unwrap();
		session.push_str(":");
		session.push_str(&port.unwrap().to_string());

		if active_sessions.contains_key(&session) {
			let id = active_sessions[&session].as_integer()?;

			return Some((session, id as u32));
		}
	} else {
		let key: String;

		if port.is_some() {
			key = port.unwrap().to_string();
		} else {
			key = host.unwrap();
		}

		let mut sessions_vec: Vec<&str> = vec![];

		for session in active_sessions.iter() {
			sessions_vec.push(session.0);
		}

		for session in sessions_vec.iter().rev() {
			if session.contains(&key) {
				let id = &active_sessions[session].as_integer()?;

				return Some((session.to_string(), *id as u32));
			}
		}
	}

	None
}

pub fn get_all() -> Option<Vec<(String, u32)>> {
	let session_data = get_session_data();

	match session_data {
		Err(error) => {
			error!("Failed to get session data: {}", error);
			return None;
		}
		Ok(_) => trace!("Session data parsed"),
	}

	let (document, _) = session_data.unwrap();
	let active_sessions = document["active_sessions"].as_table().unwrap();

	if active_sessions.len() == 0 {
		return None;
	}

	let mut all_sessions: Vec<(String, u32)> = vec![];

	for session in active_sessions.iter() {
		let address = session.0.to_string();
		let id = session.1.as_integer()?;

		all_sessions.push((address, id as u32));
	}

	Some(all_sessions)
}

pub fn remove(address: &String) -> DynResult<()> {
	let (mut document, data_dir) = get_session_data()?;

	let last_session = document["last_session"].as_str().unwrap().to_string();
	let active_sessions = document["active_sessions"].as_table_mut().unwrap();

	active_sessions.remove(&address);

	if last_session == *address {
		let mut last_address = "";

		for session in active_sessions.iter() {
			last_address = session.0;
		}

		document["last_session"] = value(last_address);
	}

	fs::write(data_dir, document.to_string())?;

	Ok(())
}

pub fn remove_all() -> DynResult<()> {
	let data_dir = get_data_dir()?;
	create_session_document(&data_dir)?;

	Ok(())
}
