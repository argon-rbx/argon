use clap::Parser;

//use crate::fs;
use crate::{argon_info, server};

/// Serve Argon session
#[derive(Parser)]
#[clap(hide(true))]
pub struct Command {
	/// Server host name
	#[arg(short = 'H', long)]
	host: String,

	/// Server port
	#[arg(short, long)]
	port: u16,
}

impl Command {
	pub fn run(self) {
		argon_info!("Serving on: {}:{}", self.host, self.port);
		//fs::watch().ok();
		server::start(self.host, self.port).ok();
	}
}
