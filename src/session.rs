use anyhow::Result;
use log::{error, trace, warn};
use std::{fs, path::PathBuf};
use toml_edit::{table, value, Document};

use crate::utils;

fn get_session_path() -> Result<PathBuf> {
	let home_dir = utils::get_home_dir()?;
	let session_path = home_dir.join(".argon").join("session.toml");

	Ok(session_path)
}

fn create_session_doc(session_path: &PathBuf) -> Result<Document> {
	let mut document = Document::new();

	document["last_session"] = value("");
	document["active_sessions"] = table();

	fs::write(session_path, document.to_string())?;

	Ok(document)
}

fn get_session_data() -> Result<(Document, PathBuf)> {
	let session_path = get_session_path()?;

	if session_path.exists() {
		let session = fs::read_to_string(&session_path)?;
		let document = session.parse::<Document>()?;

		if document.contains_key("last_session")
			|| document.contains_key("active_sessions")
			|| document["last_session"].is_str()
			|| document["active_sessions"].is_table()
		{
			return Ok((document, session_path));
		}

		warn!("Session data file is corrupted! Creating new one.");

		let document = create_session_doc(&session_path)?;

		return Ok((document, session_path));
	}

	let document = create_session_doc(&session_path)?;

	Ok((document, session_path))
}

pub fn add(host: String, port: u16, id: u32) -> Result<()> {
	let (mut document, session_path) = get_session_data()?;

	let mut session = host;
	session.push(':');
	session.push_str(&port.to_string());

	document["last_session"] = value(&session);
	document["active_sessions"][&session] = value::<i64>(id as i64);

	fs::write(session_path, document.to_string())?;

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
		if active_sessions.contains_key(last_session) {
			let id = active_sessions[&last_session].as_integer()?;

			return Some((last_session.to_string(), id as u32));
		}
	} else if host.is_some() && port.is_some() {
		let mut session = host.unwrap();
		session.push(':');
		session.push_str(&port.unwrap().to_string());

		if active_sessions.contains_key(&session) {
			let id = active_sessions[&session].as_integer()?;

			return Some((session, id as u32));
		}
	} else {
		let key = host.unwrap_or_else(|| port.unwrap().to_string());

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

	if active_sessions.is_empty() {
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

pub fn remove(address: &String) -> Result<()> {
	let (mut document, session_path) = get_session_data()?;

	let last_session = document["last_session"].as_str().unwrap().to_string();
	let active_sessions = document["active_sessions"].as_table_mut().unwrap();

	active_sessions.remove(address);

	if last_session == *address {
		let mut last_address = "";

		for session in active_sessions.iter() {
			last_address = session.0;
		}

		document["last_session"] = value(last_address);
	}

	fs::write(session_path, document.to_string())?;

	Ok(())
}

pub fn remove_all() -> Result<()> {
	let session_path = get_session_path()?;
	create_session_doc(&session_path)?;

	Ok(())
}
