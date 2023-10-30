use notify::{Config, RecommendedWatcher, RecursiveMode, Result, Watcher};
//use std::net::{TcpListener, TcpStream};
use std::path::Path;

use crate::argon_info;

#[tokio::main]
pub async fn watch() -> Result<()> {
	argon_info!("Started watching file changes!");

	let (sender, receiver) = std::sync::mpsc::channel();

	let mut watcher = RecommendedWatcher::new(sender, Config::default())?;

	watcher.watch(Path::new("."), RecursiveMode::Recursive)?;

	for response in receiver {
		match response {
			Ok(event) => println!("{event:?}"),
			Err(error) => println!("error: {:?}", error),
		}
	}

	println!("end!");

	Ok(())
}
