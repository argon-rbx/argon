use anyhow::Result;
use notify::{Config, RecommendedWatcher, RecursiveMode, Watcher};
use std::{path::PathBuf, sync::mpsc};

pub struct Fs {
	watcher: RecommendedWatcher,
	receiver: mpsc::Receiver<notify::Result<notify::Event>>,
	sync_paths: Vec<PathBuf>,
	project_root: PathBuf,
}

impl Fs {
	pub fn new(project_root: PathBuf) -> Self {
		let (sender, receiver) = mpsc::channel();
		let config = Config::default().with_compare_contents(true);
		let watcher = RecommendedWatcher::new(sender, config).unwrap();

		Self {
			watcher,
			receiver,
			sync_paths: vec![],
			project_root,
		}
	}

	pub async fn start(&mut self, sync_paths: &Vec<PathBuf>) -> Result<()> {
		self.sync_paths = sync_paths.to_owned();

		self.watch(sync_paths)?;
		self.main().await?;

		Ok(())
	}

	fn watch(&mut self, paths: &Vec<PathBuf>) -> Result<()> {
		self.watcher.watch(&self.project_root, RecursiveMode::NonRecursive)?;

		for path in paths {
			if path.exists() {
				self.watcher.watch(path, RecursiveMode::Recursive)?;
			}
		}

		Ok(())
	}

	async fn main(&self) -> Result<()> {
		for response in &self.receiver {
			match response {
				Ok(event) => println!("{event:?}"),
				Err(error) => println!("error: {:?}", error),
			}
		}

		Ok(())
	}
}

unsafe impl Sync for Fs {}
