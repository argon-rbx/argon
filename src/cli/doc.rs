use anyhow::Result;
use clap::Parser;

use crate::argon_info;

/// Open Argon's documentation in the browser
#[derive(Parser)]
pub struct Doc {}

impl Doc {
	pub fn main(self) -> Result<()> {
		argon_info!("Opening browser");

		open::that("https://argon.wiki/docs")?;

		Ok(())
	}
}
