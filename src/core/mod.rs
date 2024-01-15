use anyhow::Result;
use log::{error, trace, warn};
use rbx_dom_weak::types::Ref;
use rbx_xml::EncodeOptions;
use std::{
	fs::{self, File},
	io::BufWriter,
	path::Path,
	sync::{mpsc::Sender, Arc, Mutex, MutexGuard},
	thread,
};

use self::{dom::Dom, processor::Processor, queue::Queue};
use crate::{
	argon_warn,
	config::Config,
	fs::{Fs, FsEventKind},
	lock,
	messages::{Create, Message, SyncMeta},
	project::Project,
};

mod dom;
mod instance;
mod processor;
mod queue;

pub struct Core {
	config: Arc<Config>,
	project: Arc<Mutex<Project>>,
	fs: Arc<Mutex<Fs>>,
	processor: Arc<Mutex<Processor>>,
	queue: Arc<Mutex<Queue>>,
	dom: Arc<Mutex<Dom>>,
}

impl Core {
	pub fn new(config: Config, project: Project) -> Result<Self> {
		let dom = Dom::new(&project);
		let fs = Fs::new(&project.workspace_dir)?;

		let config = Arc::new(config);
		let project = Arc::new(Mutex::new(project));
		let fs = Arc::new(Mutex::new(fs));
		let dom = Arc::new(Mutex::new(dom));

		let queue = Arc::new(Mutex::new(Queue::new()));
		let processor = Arc::new(Mutex::new(Processor::new(
			dom.clone(),
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
			dom,
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

	pub fn game_id(&self) -> Option<u64> {
		lock!(self.project).game_id
	}

	pub fn place_ids(&self) -> Option<Vec<u64>> {
		lock!(self.project).place_ids.clone()
	}

	pub fn queue(&self) -> MutexGuard<'_, Queue> {
		lock!(self.queue)
	}

	pub fn load_dom(&mut self) -> Result<()> {
		let processor = lock!(self.processor);
		let project_paths = lock!(self.project).get_paths();

		for path in &project_paths {
			processor.init(path)?;

			if let Ok(read_dir) = fs::read_dir(path) {
				for entry in read_dir {
					let entry = entry?;

					match processor.create(&entry.path(), true) {
						Ok(_) => {
							trace!("Processed path: {:?}", entry.path());
						}
						Err(err) => {
							error!("Failed to process path: {:?}, due to: {}", entry.path(), err);
						}
					}
				}
			}
		}

		Ok(())
	}

	pub fn watch(&self, sender: Option<Sender<()>>) {
		let processor = self.processor.clone();
		let project = self.project.clone();
		let queue = self.queue.clone();
		let dom = self.dom.clone();
		let fs = self.fs.clone();

		thread::spawn(move || -> Result<()> {
			let receiver = lock!(fs).receiver();

			// Start watching for the file changes
			lock!(fs).watch_all(&lock!(project).get_paths())?;

			for event in receiver {
				if event.root {
					match event.kind {
						FsEventKind::Write => {
							if event.path == lock!(project).project_path {
								let result = lock!(project).reload();

								match result {
									Ok(changes) => {
										if changes.address {
											argon_warn!(
												"The project address has changed! Restart Argon to apply changes."
											)
										}

										if changes.paths {
											warn!("Rebuilding DOM - project paths changed! This might take a while..");

											lock!(dom).reload(&lock!(project));

											let mut fs = lock!(fs);
											let processor = lock!(processor);
											let project_paths = lock!(project).get_paths();

											for path in &project_paths {
												processor.init(path)?;

												if let Ok(read_dir) = fs::read_dir(path) {
													for entry in read_dir {
														let entry = entry?;

														match processor.create(&entry.path(), true) {
															Ok(_) => {
																trace!("Reloaded path: {:?}", entry.path());
															}
															Err(err) => {
																error!(
																	"Failed to reload path: {:?}, due to: {}",
																	entry.path(),
																	err
																);
															}
														}
													}
												}
											}

											fs.unwatch_all(&project_paths)?;
											fs.watch_all(&project_paths)?;

											if let Some(sender) = sender.clone() {
												sender.send(()).unwrap();
											}

											println!("{:?}", "DONE!");
										}

										if changes.meta {
											let mut queue = lock!(queue);
											let project = lock!(project);

											queue.push(
												Message::SyncMeta(SyncMeta {
													name: project.name.clone(),
													game_id: project.game_id,
													place_ids: project.place_ids.clone(),
												}),
												None,
											);
										}
									}
									Err(err) => {
										warn!("Failed to reload the project: {}", err);
										continue;
									}
								}
							}
						}
						_ => {
							let project_paths = lock!(project).get_paths();
							let mut fs = lock!(fs);

							if event.path.is_dir() && project_paths.contains(&event.path) {
								fs.unwatch_all(&project_paths)?;
								fs.watch_all(&project_paths)?;
							}
						}
					}

					continue;
				}

				let result = || -> Result<()> {
					let processor = lock!(processor);

					match event.kind {
						FsEventKind::Create => processor.create(&event.path, false)?,
						FsEventKind::Delete => processor.delete(&event.path)?,
						FsEventKind::Write => processor.write(&event.path)?,
					}

					if let Some(sender) = sender.clone() {
						sender.send(()).unwrap();
					}

					Ok(())
				};

				match result() {
					Ok(_) => {
						trace!("Processed event: {:?}", event);
					}
					Err(err) => {
						error!("Failed to process event: {:?}, due to: {}", event, err);
					}
				}
			}

			Ok(())
		});
	}

	pub fn build(&self, path: &Path, xml: bool) -> Result<()> {
		let writer = BufWriter::new(File::create(path)?);

		let project = lock!(self.project);
		let dom = lock!(self.dom);

		let root_refs = if project.is_place() {
			dom.place_root_refs().to_vec()
		} else {
			vec![dom.root_ref()]
		};

		if xml {
			rbx_xml::to_writer(writer, dom.inner(), &root_refs, EncodeOptions::default())?;
		} else {
			rbx_binary::to_writer(writer, dom.inner(), &root_refs)?;
		}

		Ok(())
	}

	pub fn sync_dom(&self, id: u64) {
		let dom = lock!(self.dom);
		let mut queue = lock!(self.queue);

		fn walk(children: &[Ref], dom: &Dom, queue: &mut MutexGuard<'_, Queue>, id: &u64) {
			for child in children {
				let child = dom.get_by_ref(*child).unwrap();
				let path = dom.get_rbx_path(child.referent()).unwrap();

				queue.push(
					Message::Create(Create {
						class: child.class.clone(),
						path: path.clone(),
						properties: child.properties.clone(),
					}),
					Some(id),
				);

				walk(child.children(), dom, queue, id);
			}
		}

		walk(dom.root().children(), &dom, &mut queue, &id);
	}
}
