use anyhow::Result;
use clap::Parser;
use colored::Colorize;

use crate::argon_info;

const LINK: &str = "https://argon.wiki";

/// Open Argon's documentation in the browser
#[derive(Parser)]
pub struct Doc {}

impl Doc {
	pub fn main(self) -> Result<()> {
		argon_info!("Launched browser. Manually go to: {}", LINK.bold());

		open::that(LINK)?;

		Ok(())
	}
}
