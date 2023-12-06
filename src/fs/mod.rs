mod debouncer;
pub mod watcher;

use self::watcher::{FsWatcher, WorkspaceEvent};
use anyhow::Result;
use crossbeam_channel::Receiver;
use std::path::PathBuf;

pub struct Fs {
	watcher: FsWatcher,
	receiver: Receiver<WorkspaceEvent>,
}

impl Fs {
	pub fn new(root: &PathBuf) -> Result<Self> {
		let (sender, receiver) = crossbeam_channel::unbounded();
		let watcher = FsWatcher::new(root, &sender)?;

		watcher.start()?;

		Ok(Self { watcher, receiver })
	}

	pub fn watch(&mut self, path: &PathBuf) -> Result<()> {
		self.watcher.watch(path)
	}

	pub fn unwatch(&mut self, path: &PathBuf) -> Result<()> {
		self.watcher.unwatch(path)
	}

	pub fn watch_all(&mut self, paths: &Vec<PathBuf>) -> Result<()> {
		for path in paths {
			self.watch(path)?;
		}

		Ok(())
	}

	pub fn unwatch_all(&mut self, paths: &Vec<PathBuf>) -> Result<()> {
		for path in paths {
			self.unwatch(path)?;
		}

		Ok(())
	}

	pub fn receiver(&self) -> Receiver<WorkspaceEvent> {
		self.receiver.clone()
	}
}
