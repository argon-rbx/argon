use anyhow::{Context, Result};
use log::{debug, trace, warn};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fs, process, thread};

use crate::util;

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct Session {
	pub pid: u32,
	pub host: Option<String>,
	pub port: Option<u16>,
}

impl Session {
	pub fn get_address(&self) -> Option<String> {
		if let Some(host) = &self.host {
			if let Some(port) = self.port {
				return Some(format!("http://{}:{}", host, port));
			}
		}

		None
	}
}

#[derive(Serialize, Deserialize, Debug)]
struct Sessions {
	last_session: String,
	active_sessions: HashMap<String, Session>,
}

fn get_sessions() -> Result<Sessions> {
	let path = util::get_argon_dir()?.join("sessions.toml");

	if path.exists() {
		match toml::from_str(&fs::read_to_string(&path)?) {
			Ok(sessions) => return Ok(sessions),
			Err(_) => warn!("Session data file is corrupted! Creating new one.."),
		}
	}

	let sessions = Sessions {
		last_session: String::new(),
		active_sessions: HashMap::new(),
	};

	fs::write(path, toml::to_string(&sessions)?)?;

	Ok(sessions)
}

fn set_sessions(sessions: &Sessions) -> Result<()> {
	let path = util::get_argon_dir()?.join("sessions.toml");

	fs::write(path, toml::to_string(sessions)?)?;

	Ok(())
}

pub fn add(id: Option<String>, host: Option<String>, port: Option<u16>, pid: u32, run_async: bool) -> Result<()> {
	let mut sessions = get_sessions()?;

	let session = Session { host, port, pid };
	let id = id.unwrap_or(generate_id(&sessions));

	sessions.last_session.clone_from(&id);
	sessions.active_sessions.insert(id, session.clone());

	set_sessions(&sessions)?;

	if !run_async {
		ctrlc::set_handler(move || {
			match remove(&session) {
				Ok(()) => trace!("Session entry removed"),
				Err(err) => warn!("Failed to remove session entry: {}", err),
			}

			process::exit(0);
		})?;
	}

	// Schedule manual cleanup of old sessions
	// as ctrlc handler does not work on Windows,
	// on UNIX cleanup will remove crashed sessions
	thread::spawn(move || match cleanup(sessions) {
		Ok(()) => debug!("Session cleanup completed"),
		Err(err) => warn!("Failed to cleanup sessions: {}", err),
	});

	Ok(())
}

pub fn get(id: Option<String>, host: Option<String>, port: Option<u16>) -> Result<Option<Session>> {
	let sessions = get_sessions()?;

	if id.is_none() && host.is_none() && port.is_none() {
		return Ok(sessions.active_sessions.get(&sessions.last_session).cloned());
	} else if let Some(id) = id {
		return Ok(sessions.active_sessions.get(&id).cloned());
	}

	for (_, session) in sessions.active_sessions {
		if session.host == host || session.port == port {
			return Ok(Some(session));
		}
	}

	Ok(None)
}

pub fn get_multiple(ids: &Vec<String>) -> Result<HashMap<String, Session>> {
	let sessions = get_sessions()?;

	let mut result = HashMap::new();

	for id in ids {
		if let Some(session) = sessions.active_sessions.get(id) {
			result.insert(id.to_owned(), session.to_owned());
		}
	}

	Ok(result)
}

pub fn get_all() -> Result<HashMap<String, Session>> {
	Ok(get_sessions()?.active_sessions)
}

pub fn remove(session: &Session) -> Result<()> {
	let mut sessions = get_sessions()?;

	let id = sessions
		.active_sessions
		.iter()
		.find_map(|(i, s)| if s == session { Some(i.clone()) } else { None })
		.context("Session not found")?;

	sessions.active_sessions.remove(&id);

	if sessions.last_session == id {
		if let Some((session_id, _)) = sessions.active_sessions.iter().next() {
			sessions.last_session.clone_from(session_id);
		} else {
			sessions.last_session = String::new();
		}
	}

	set_sessions(&sessions)?;

	Ok(())
}

pub fn remove_multiple(ids: &Vec<String>) -> Result<()> {
	let mut sessions = get_sessions()?;

	for id in ids {
		sessions.active_sessions.remove(id);
	}

	sessions.last_session = sessions.active_sessions.keys().next().cloned().unwrap_or_default();

	set_sessions(&sessions)?;

	Ok(())
}

pub fn remove_all() -> Result<()> {
	let sessions = Sessions {
		last_session: String::new(),
		active_sessions: HashMap::new(),
	};

	set_sessions(&sessions)?;

	Ok(())
}

fn cleanup(mut sessions: Sessions) -> Result<()> {
	let mut did_remove = false;

	for (id, session) in sessions.active_sessions.clone() {
		if !util::process_exists(session.pid) {
			sessions.active_sessions.remove(&id);
			did_remove = true;
		}
	}

	if did_remove {
		set_sessions(&sessions)?;
	}

	Ok(())
}

fn generate_id(sessions: &Sessions) -> String {
	let mut index = 0;

	loop {
		let id = index.to_string();

		if !sessions.active_sessions.contains_key(&id) {
			return id;
		}

		index += 1;
	}
}
