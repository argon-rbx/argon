use anyhow::Result;
use std::{
	sync::{Arc, Mutex},
	thread,
};

use crate::{
	config::Config,
	fs::{watcher::WorkspaceEventKind, Fs},
	lock,
	project::Project,
};

use self::processor::Processor;

mod processor;
mod queue;

pub struct Core {
	config: Config,
	project: Arc<Mutex<Project>>,
	fs: Arc<Mutex<Fs>>,
	processor: Arc<Processor>,
}

impl Core {
	pub fn new(config: Config, project: Project, mut fs: Fs) -> Result<Self> {
		fs.watch_all(&project.tree_paths)?;

		let project = Arc::new(Mutex::new(project));
		let fs = Arc::new(Mutex::new(fs));

		let processor = Arc::new(Processor::new(project.clone()));

		Ok(Self {
			config,
			project,
			fs,
			processor,
		})
	}

	pub fn name(&self) -> String {
		lock!(self.project).name.clone()
	}

	pub fn host(&self) -> String {
		lock!(self.project).host.clone().unwrap_or(self.config.host.clone())
	}

	pub fn port(&self) -> u16 {
		lock!(self.project).port.unwrap_or(self.config.port)
	}

	pub fn start(&self) {
		let processor = self.processor.clone();
		let project = self.project.clone();
		let fs = self.fs.clone();

		thread::spawn(move || -> Result<()> {
			let receiver = lock!(fs).receiver();

			for event in receiver.iter() {
				if event.root {
					match event.kind {
						WorkspaceEventKind::Write => {
							if event.path == lock!(project).project_path {
								let new_project = Project::load(&lock!(project).project_path)?;
								let mut fs = lock!(fs);

								fs.unwatch_all(&project.lock().unwrap().tree_paths)?;
								*project.lock().unwrap() = new_project;
								fs.watch_all(&project.lock().unwrap().tree_paths)?;
							}
						}
						_ => {
							let project = lock!(project);
							let mut fs = lock!(fs);

							if event.path.is_dir() && project.tree_paths.contains(&event.path) {
								fs.unwatch_all(&project.tree_paths)?;
								fs.watch_all(&project.tree_paths)?;
							}
						}
					}

					continue;
				}

				match event.kind {
					WorkspaceEventKind::Create => processor.create(&event.path),
					WorkspaceEventKind::Delete => processor.delete(&event.path),
					WorkspaceEventKind::Write => processor.write(&event.path),
				}
			}

			Ok(())
		});
	}
}
