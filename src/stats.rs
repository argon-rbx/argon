use anyhow::Result;
use lazy_static::lazy_static;
use log::{debug, warn};
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};
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
	#[serde(rename(serialize = "hours_used"))]
	minutes_used: u32,
	files_synced: u32,
	lines_synced: u32,
	projects_created: u32,
	projects_built: u32,
	sessions_started: u32,
}

impl ArgonStats {
	fn len(&self) -> u32 {
		self.minutes_used
			+ self.files_synced
			+ self.lines_synced
			+ self.projects_created
			+ self.projects_built
			+ self.sessions_started
	}
}

#[derive(Debug, Serialize, Deserialize)]
struct StatTracker {
	last_synced: SystemTime,
	stats: ArgonStats,
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

	if tracker.last_synced.elapsed()?.as_secs() > 3600 && tracker.stats.len() > 10 {
		if let Some(token) = option_env!("AUTH_TOKEN") {
			let mut stats = tracker.stats.clone();
			let mut hours = stats.minutes_used / 60;

			if stats.minutes_used % 60 >= 30 {
				hours += 1;
			}

			stats.minutes_used = hours;

			Client::new()
				.post(format!("https://api.argon.wiki/push?auth={}", token))
				.json(&stats)
				.send()?;

			tracker.last_synced = SystemTime::now();
			tracker.stats = ArgonStats::default();

			set_tracker(&tracker)?;
		} else {
			warn!("This Argon build has no `AUTH_TOKEN` set, stats will not be synced")
		}
	} else {
		debug!("Stats already synced within the last hour or too few stats to sync");
	}

	*TRACKER.write().unwrap() = tracker;

	thread::spawn(|| loop {
		thread::sleep(Duration::from_secs(300));
		minutes_used(5);
		save().ok();
	});

	sessions_started(1);

	Ok(())
}

pub fn save() -> Result<()> {
	let tracker = TRACKER.read().unwrap();

	set_tracker(&tracker)?;

	Ok(())
}

stat_fn!(minutes_used);
stat_fn!(files_synced);
stat_fn!(lines_synced);
stat_fn!(projects_created);
stat_fn!(projects_built);
stat_fn!(sessions_started);
