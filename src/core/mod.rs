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
	messages::{Message, UpdateMeta},
	project::Project,
};

use self::{processor::Processor, queue::Queue};

mod processor;
mod queue;

pub struct Core {
	config: Arc<Config>,
	project: Arc<Mutex<Project>>,
	fs: Arc<Mutex<Fs>>,
	processor: Arc<Mutex<Processor>>,
	queue: Arc<Mutex<Queue>>,
}

impl Core {
	pub fn new(config: Config, project: Project) -> Result<Self> {
		let mut fs = Fs::new(&project.workspace_dir)?;
		fs.watch_all(&project.local_paths)?;

		let config = Arc::new(config);
		let project = Arc::new(Mutex::new(project));
		let fs = Arc::new(Mutex::new(fs));

		let queue = Arc::new(Mutex::new(Queue::new()));
		let processor = Arc::new(Mutex::new(Processor::new(
			queue.clone(),
			project.clone(),
			config.clone(),
		)));

		Ok(Self {
			config,
			project,
			fs,
			processor,
			queue,
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
		let queue = self.queue.clone();
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

								let mut queue = lock!(queue);
								let mut fs = lock!(fs);

								fs.unwatch_all(&lock!(project).local_paths)?;

								*lock!(project) = new_project.unwrap();

								let project = lock!(project);

								fs.watch_all(&project.local_paths)?;

								queue.push(Message::UpdateMeta(UpdateMeta {
									name: project.name.clone(),
									game_id: project.game_id,
									place_ids: project.place_ids.clone(),
								}));
							}
						}
						_ => {
							let project = lock!(project);
							let mut fs = lock!(fs);

							if event.path.is_dir() && project.local_paths.contains(&event.path) {
								fs.unwatch_all(&project.local_paths)?;
								fs.watch_all(&project.local_paths)?;
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
