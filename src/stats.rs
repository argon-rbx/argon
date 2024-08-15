use anyhow::Result;
use lazy_static::lazy_static;
use log::{debug, warn};
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::{
	fs,
	sync::RwLock,
	thread,
	time::{Duration, SystemTime},
};

use crate::util;

lazy_static! {
	static ref TRACKER: RwLock<StatTracker> = RwLock::new(StatTracker::default());
}

macro_rules! stat_fn {
	($name:ident) => {
		pub fn $name($name: u32) {
			TRACKER.write().unwrap().stats.$name += $name;
		}
	};
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
struct ArgonStats {
	minutes_used: u32,
	files_synced: u32,
	lines_synced: u32,
	projects_created: u32,
	projects_built: u32,
	sessions_started: u32,
}

impl ArgonStats {
	fn total(&self) -> u32 {
		self.minutes_used / 60
			+ self.files_synced
			+ self.lines_synced
			+ self.projects_created
			+ self.projects_built
			+ self.sessions_started
	}

	fn extend(&mut self, other: &ArgonStats) {
		self.minutes_used += other.minutes_used;
		self.files_synced += other.files_synced;
		self.lines_synced += other.lines_synced;
		self.projects_created += other.projects_created;
		self.projects_built += other.projects_built;
		self.sessions_started += other.sessions_started;
	}
}

#[derive(Debug, Serialize, Deserialize)]
struct StatTracker {
	last_synced: SystemTime,
	stats: ArgonStats,
}

impl StatTracker {
	fn reset(&mut self) {
		self.stats = ArgonStats::default();
	}

	fn merge(&mut self, other: Self) {
		if other.last_synced > self.last_synced {
			self.last_synced = other.last_synced;
		}

		self.stats.extend(&other.stats);
	}
}

impl Default for StatTracker {
	fn default() -> Self {
		Self {
			last_synced: SystemTime::UNIX_EPOCH,
			stats: ArgonStats::default(),
		}
	}
}

fn get_tracker() -> Result<StatTracker> {
	let path = util::get_argon_dir()?.join("stats.toml");

	if path.exists() {
		match toml::from_str(&fs::read_to_string(&path)?) {
			Ok(tracker) => return Ok(tracker),
			Err(_) => warn!("Stat tracker file is corrupted! Creating new one.."),
		}
	}

	let tracker = StatTracker::default();

	fs::write(path, toml::to_string(&tracker)?)?;

	Ok(tracker)
}

fn set_tracker(tracker: &StatTracker) -> Result<()> {
	let path = util::get_argon_dir()?.join("stats.toml");

	fs::write(path, toml::to_string(tracker)?)?;

	Ok(())
}

pub fn track() -> Result<()> {
	let mut tracker = get_tracker()?;

	if tracker.last_synced.elapsed()?.as_secs() > 3600 && tracker.stats.total() > 10 {
		if let Some(token) = option_env!("ARGON_TOKEN") {
			let stats = tracker.stats;
			let remainder = stats.minutes_used % 60;

			let stats = json!({
				"hours_used": stats.minutes_used / 60,
				"files_synced": stats.files_synced,
				"lines_synced": stats.lines_synced,
				"projects_created": stats.projects_created,
				"projects_built": stats.projects_built,
				"sessions_started": stats.sessions_started,
			});

			Client::new()
				.post(format!("https://api.argon.wiki/push?auth={}", token))
				.json(&stats)
				.send()?;

			tracker.last_synced = SystemTime::now();
			tracker.stats = ArgonStats::default();

			tracker.stats.minutes_used = remainder;

			set_tracker(&tracker)?;
		} else {
			warn!("This Argon build has no `ARGON_TOKEN` set, stats will not be uploaded")
		}
	} else {
		debug!("Stats already synced within the last hour or too few stats to sync");
	}

	thread::spawn(|| loop {
		thread::sleep(Duration::from_secs(300));
		minutes_used(5);

		match save() {
			Ok(_) => debug!("Stats saved successfully"),
			Err(err) => warn!("Failed to save stats: {}", err),
		}
	});

	sessions_started(1);

	Ok(())
}

pub fn save() -> Result<()> {
	let mut tracker = TRACKER.write().unwrap();

	if let Ok(old) = get_tracker() {
		tracker.merge(old);
	}

	set_tracker(&tracker)?;
	tracker.reset();

	Ok(())
}

stat_fn!(minutes_used);
stat_fn!(files_synced);
stat_fn!(lines_synced);
stat_fn!(projects_created);
stat_fn!(projects_built);
stat_fn!(sessions_started);
