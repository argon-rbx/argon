mod debouncer;
mod watcher;

use anyhow::Result;

use std::{
	path::PathBuf,
	sync::mpsc::{self},
};
use watcher::WorkspaceWatcher;

pub struct Fs {
	watcher: WorkspaceWatcher,
	sync_paths: Vec<PathBuf>,
}

impl Fs {
	pub fn new(root_dir: &PathBuf, sync_paths: &Vec<PathBuf>) -> Result<Self> {
		let watcher = WorkspaceWatcher::new(root_dir.to_owned())?;

		let mut fs = Self {
			watcher,
			sync_paths: sync_paths.to_owned(),
		};

		fs.watch()?;

		Ok(fs)
	}

	#[tokio::main]
	pub async fn start(&mut self) -> Result<()> {
		let (sender, receiver) = mpsc::channel();
		self.watcher.start(sender)?;

		for event in receiver {
			// println!("{:?}", event);
		}

		Ok(())
	}

	fn watch(&mut self) -> Result<()> {
		for path in &self.sync_paths {
			self.watcher.watch(path)?;
		}

		Ok(())
	}

	fn unwatch(&mut self) -> Result<()> {
		for path in &self.sync_paths {
			self.watcher.unwatch(path)?;
		}

		Ok(())
	}
}
