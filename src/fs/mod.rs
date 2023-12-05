mod debouncer;
mod watcher;

use crate::project::Project;

use self::watcher::{ArgonWatcher, WorkspaceEvent, WorkspaceEventKind};
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
	project: PathBuf,
}

impl Fs {
	pub fn new(project: &Project) -> Result<Self> {
		let (sender, receiver) = mpsc::channel();
		let watcher = ArgonWatcher::new(&project.workspace, &sender)?;

		let receiver = Arc::new(Mutex::new(receiver));

		let mut fs = Self {
			watcher,
			receiver,
			sync_paths: project.get_sync_paths().to_owned(),
			project: project.project.to_owned(),
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
			if event.root {
				match event.kind {
					WorkspaceEventKind::Write => {
						if event.path == self.project {
							self.update(true)?;
						}
					}
					_ => {
						if event.path.is_dir() && self.sync_paths.contains(&event.path) {
							self.update(false)?;
						}
					}
				}

				continue;
			}

			// TODO: Add to queue here
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

	fn update(&mut self, is_project: bool) -> Result<()> {
		self.unwatch()?;

		if is_project {
			let project = Project::load(&self.project)?;
			let project_paths = project.get_sync_paths();

			self.sync_paths = project_paths;
		}

		self.watch()?;

		Ok(())
	}
}
