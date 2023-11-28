mod debouncer;
mod watcher;

use self::watcher::{ArgonWatcher, WorkspaceEvent};
use anyhow::Result;
use std::{
	path::PathBuf,
	sync::{
		mpsc::{self, Receiver},
		Arc, Mutex,
	},
};

pub struct Fs {
	watcher: ArgonWatcher,
	receiver: Arc<Mutex<Receiver<WorkspaceEvent>>>,
	sync_paths: Vec<PathBuf>,
}

impl Fs {
	pub fn new(root_dir: &PathBuf, sync_paths: &Vec<PathBuf>) -> Result<Self> {
		let (sender, receiver) = mpsc::channel();
		let watcher = ArgonWatcher::new(root_dir, &sender)?;

		let receiver = Arc::new(Mutex::new(receiver));

		let mut fs = Self {
			watcher,
			receiver,
			sync_paths: sync_paths.to_owned(),
		};

		fs.watch()?;

		Ok(fs)
	}

	#[tokio::main]
	pub async fn start(&mut self) -> Result<()> {
		let receiver = self.receiver.clone();
		let receiver = receiver.lock().unwrap();

		self.watcher.start()?;

		for event in receiver.iter() {
			println!("{:?}", event);
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
