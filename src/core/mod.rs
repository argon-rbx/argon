use anyhow::Result;
use log::warn;
use std::{
	sync::{Arc, Mutex},
	thread,
};

use crate::{
	config::Config,
	fs::{Fs, FsEventKind},
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
	processor: Arc<Mutex<Processor>>,
}

impl Core {
	pub fn new(config: Config, project: Project, mut fs: Fs) -> Result<Self> {
		fs.watch_all(&project.sync_paths)?;

		let processor = Processor::new(project.ignore_globs.clone());

		let project = Arc::new(Mutex::new(project));
		let fs = Arc::new(Mutex::new(fs));
		let processor = Arc::new(Mutex::new(processor));

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
						FsEventKind::Write => {
							if event.path == lock!(project).project_path {
								let new_project = Project::load(&lock!(project).project_path);

								if new_project.is_err() {
									warn!("Failed to reload project: {:?}", new_project);
									continue;
								}

								let mut processor = lock!(processor);
								let mut fs = lock!(fs);

								fs.unwatch_all(&lock!(project).sync_paths)?;

								*lock!(project) = new_project.unwrap();

								let project = lock!(project);

								fs.watch_all(&project.sync_paths)?;
								processor.set_ignore_globs(project.ignore_globs.clone());
							}
						}
						_ => {
							let project = lock!(project);
							let mut fs = lock!(fs);

							if event.path.is_dir() && project.sync_paths.contains(&event.path) {
								fs.unwatch_all(&project.sync_paths)?;
								fs.watch_all(&project.sync_paths)?;
							}
						}
					}

					continue;
				}

				let processor = lock!(processor);

				match event.kind {
					FsEventKind::Create => processor.create(&event.path),
					FsEventKind::Delete => processor.delete(&event.path),
					FsEventKind::Write => processor.write(&event.path),
				}
			}

			Ok(())
		});
	}
}
