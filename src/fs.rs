use anyhow::Result;
use notify::{poll::ScanEvent, Config, PollWatcher, RecommendedWatcher, RecursiveMode, Watcher};
use notify_debouncer_full::new_debouncer;
use std::{
	path::{Path, PathBuf},
	sync::mpsc::{self},
	thread::{self, JoinHandle},
	time::Duration,
};

pub struct Fs {
	thread: JoinHandle<()>,
	watcher: PollWatcher,
	watched_paths: Vec<PathBuf>,
	sync_paths: Vec<PathBuf>,
	root_dir: PathBuf,
	source_dir: PathBuf,
}

impl Fs {
	pub fn new(root_dir: PathBuf, source: String) -> Result<Self> {
		let source_dir = root_dir.join(source);

		// let (sender, receiver) = mpsc::channel();
		// let (workpsace_sender, workspace_receiver) = mpsc::channel();

		// let config = Config::default().with_compare_contents(true);
		// let watcher = PollWatcher::new(sender, Config::default())?;
		// let mut debouncer = new_debouncer(Duration::from_millis(100), None, sender)?;
		// let watcher = RecommendedWatcher::new(sender, config).unwrap();
		// let workspace_watcher = RecommendedWatcher::new(workpsace_sender, Config::default());

		// debouncer.watcher().watch(
		// 	Path::new("/Users/dervex/Desktop/argon/test/src/ReplicatedStorage"),
		// 	RecursiveMode::Recursive,
		// )?;

		// debouncer.watcher().watch(
		// 	Path::new("/Users/dervex/Desktop/argon/test/src/ReplicatedFirst"),
		// 	RecursiveMode::Recursive,
		// )?;

		// debouncer.cache().add_root(
		// 	Path::new("/Users/dervex/Desktop/argon/test/src/ReplicatedStorage"),
		// 	RecursiveMode::Recursive,
		// );

		// debouncer.cache().add_root(
		// 	Path::new("/Users/dervex/Desktop/argon/test/src/ReplicatedFirst"),
		// 	RecursiveMode::Recursive,
		// );

		let (tx, rx) = mpsc::channel();
		// use the PollWatcher and disable automatic polling
		let mut watcher = PollWatcher::new(tx, Config::default().with_poll_interval(Duration::from_millis(100)))?;

		// Add a path to be watched. All files and directories at that path and
		// below will be monitored for changes.
		watcher.watch(
			Path::new("/Users/dervex/Desktop/argon/test/src/ReplicatedStorage"),
			RecursiveMode::Recursive,
		)?;
		watcher.watch(
			Path::new("/Users/dervex/Desktop/argon/test/src/ReplicatedFirst"),
			RecursiveMode::Recursive,
		)?;

		// run event receiver on a different thread, we want this one for user input
		let thread = std::thread::spawn(move || {
			for res in rx {
				match res {
					Ok(event) => println!("changed: {:?}", event),
					Err(e) => println!("watch error: {:?}", e),
				}
			}
		});

		let mut fs = Self {
			thread,
			watcher,
			watched_paths: vec![],
			sync_paths: vec![
				PathBuf::from("/Users/dervex/Desktop/argon/test/src/ReplicatedStorage"),
				PathBuf::from("/Users/dervex/Desktop/argon/test/src/ReplicatedFirst"),
			],
			root_dir,
			source_dir,
		};

		fs.watch()?;
		Ok(fs)
	}

	pub async fn start(&mut self, sync_paths: &Vec<PathBuf>) -> Result<()> {
		self.sync_paths = sync_paths.to_owned();

		self.watch()?;
		// self.main().await?;

		Ok(())
	}

	fn watch(&mut self) -> Result<()> {
		for path in &self.sync_paths {
			// if path.exists() {
			self.watcher.watch(path, RecursiveMode::Recursive)?;
			self.watched_paths.push(path.to_owned());
			// }
		}

		Ok(())
	}

	// fn unwatch(&mut self) -> Result<()> {
	// 	for path in &self.watched_paths {
	// 		self.watcher.unwatch(path)?;
	// 	}

	// 	self.watched_paths.clear();

	// 	Ok(())
	// }

	fn update_watched_paths() -> Result<()> {
		Ok(())
	}
}

unsafe impl Sync for Fs {}
