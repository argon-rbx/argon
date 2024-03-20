use anyhow::Result;
use log::{trace, warn};
use serde::{Deserialize, Serialize};
use std::{
	collections::HashMap,
	fs,
	path::{Path, PathBuf},
	process,
};

#[cfg(target_os = "windows")]
use std::thread;

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

fn get_path() -> Result<PathBuf> {
	let home_dir = util::get_home_dir()?;
	let session_path = home_dir.join(".argon").join("sessions.toml");

	Ok(session_path)
}

fn get_sessions(path: &Path) -> Result<Sessions> {
	fn create_empty(path: &Path) -> Result<Sessions> {
		let sessions = Sessions {
			last_session: String::new(),
			active_sessions: HashMap::new(),
		};

		fs::write(path, toml::to_string(&sessions)?)?;

		Ok(sessions)
	}

	if !path.exists() {
		warn!("Session data file not found! Creating new one..");
		return create_empty(path);
	}

	let sessions_toml = fs::read_to_string(path)?;
	let sessions = toml::from_str::<Sessions>(&sessions_toml);

	match sessions {
		Ok(sessions) => {
			trace!("Session data parsed");
			Ok(sessions)
		}
		Err(_) => {
			warn!("Session data file is corrupted! Creating new one..");
			create_empty(path)
		}
	}
}

pub fn add(id: Option<String>, host: Option<String>, port: Option<u16>, pid: u32, run_async: bool) -> Result<()> {
	let path = get_path()?;
	let mut sessions = get_sessions(&path)?;

	let session = Session { host, port, pid };
	let id = id.unwrap_or(generate_id(&sessions));

	sessions.last_session = id.clone();
	sessions.active_sessions.insert(id, session.clone());

	fs::write(&path, toml::to_string(&sessions)?)?;

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
	// as ctrlc handler does not work on Windows
	#[cfg(target_os = "windows")]
	thread::spawn(move || match cleanup(sessions, &path) {
		Ok(()) => trace!("Session cleanup completed"),
		Err(err) => warn!("Failed to cleanup sessions: {}", err),
	});

	Ok(())
}

pub fn get(id: Option<String>, host: Option<String>, port: Option<u16>) -> Result<Option<Session>> {
	let path = get_path()?;
	let sessions = get_sessions(&path)?;

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

pub fn get_all() -> Result<Option<HashMap<String, Session>>> {
	let path = get_path()?;
	let sessions = get_sessions(&path)?;

	if !sessions.active_sessions.is_empty() {
		return Ok(Some(sessions.active_sessions));
	}

	Ok(None)
}

pub fn remove(session: &Session) -> Result<()> {
	let path = get_path()?;
	let mut sessions = get_sessions(&path)?;

	let id = sessions
		.active_sessions
		.iter()
		.find_map(|(i, s)| if s == session { Some(i.clone()) } else { None })
		.unwrap();

	sessions.active_sessions.remove(&id);

	if sessions.last_session == id {
		if let Some((session_id, _)) = sessions.active_sessions.iter().next() {
			sessions.last_session = session_id.clone();
		} else {
			sessions.last_session = String::new();
		}
	}

	fs::write(path, toml::to_string(&sessions)?)?;

	Ok(())
}

pub fn remove_all() -> Result<()> {
	let path = get_path()?;

	let sessions = Sessions {
		last_session: String::new(),
		active_sessions: HashMap::new(),
	};

	fs::write(path, toml::to_string(&sessions)?)?;

	Ok(())
}

#[allow(dead_code)] // Windows only
fn cleanup(mut sessions: Sessions, path: &Path) -> Result<()> {
	for (id, session) in sessions.active_sessions.clone() {
		if !util::process_exists(session.pid) {
			sessions.active_sessions.remove(&id);
		}
	}

	fs::write(path, toml::to_string(&sessions)?)?;

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
