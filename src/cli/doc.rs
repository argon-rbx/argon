use anyhow::Result;
use clap::Parser;
use log::trace;

/// Open Argon's documentation in the browser
#[derive(Parser)]
pub struct Doc {}

impl Doc {
	pub fn main(self) -> Result<()> {
		trace!("Opening browser!");

		open::that("https://argon.wiki/docs")?;

		Ok(())
	}
}
