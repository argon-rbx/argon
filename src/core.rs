use anyhow::Result;
use std::{
	path::PathBuf,
	sync::{Arc, Mutex},
	thread,
};

use crate::{
	config::Config,
	fs::{watcher::WorkspaceEventKind, Fs},
	project::Project,
};

pub struct Core {
	config: Config,
	project: Arc<Mutex<Project>>,
	fs: Arc<Mutex<Fs>>,
}

impl Core {
	pub fn new(config: Config, project_path: &PathBuf) -> Result<Self> {
		let project = Project::load(project_path)?;
		let mut fs = Fs::new(&project.workspace_dir)?;

		fs.watch_all(&project.tree_paths)?;

		let project = Arc::new(Mutex::new(project));
		let fs = Arc::new(Mutex::new(fs));

		Ok(Self { config, project, fs })
	}

	pub fn name(&self) -> String {
		self.project.lock().unwrap().name.clone()
	}

	pub fn host(&self) -> String {
		self.project
			.lock()
			.unwrap()
			.host
			.clone()
			.unwrap_or(self.config.host.clone())
	}

	pub fn port(&self) -> u16 {
		self.project.lock().unwrap().port.unwrap_or(self.config.port)
	}

	pub fn start(&self) {
		let project = self.project.clone();
		let fs = self.fs.clone();

		thread::spawn(move || -> Result<()> {
			let receiver = fs.lock().unwrap().receiver();

			for event in receiver.iter() {
				if event.root {
					match event.kind {
						WorkspaceEventKind::Write => {
							if event.path == project.lock().unwrap().project_path {
								let new_project = Project::load(&project.lock().unwrap().project_path)?;
								let mut fs = fs.lock().unwrap();

								fs.unwatch_all(&project.lock().unwrap().tree_paths)?;
								*project.lock().unwrap() = new_project;
								fs.watch_all(&project.lock().unwrap().tree_paths)?;
							}
						}
						_ => {
							let project = project.lock().unwrap();

							if event.path.is_dir() && project.tree_paths.contains(&event.path) {
								let mut fs = fs.lock().unwrap();

								fs.unwatch_all(&project.tree_paths)?;
								fs.watch_all(&project.tree_paths)?;
							}
						}
					}

					continue;
				}

				// TODO: Add to queue here
				println!("{:?}", event);
			}

			Ok(())
		});
	}
}
