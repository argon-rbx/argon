use anyhow::{Context, Result};
use clap::Parser;

#[derive(Parser)]
pub struct Test {}

impl Test {
	fn test() -> Option<u32> {
		None
	}

	pub fn main(self) -> Result<()> {
		log::error!("1");
		log::warn!("2");
		log::info!("3");
		log::debug!("4");
		log::trace!("5");

		crate::argon_error!("argon_1");
		crate::argon_warn!("argon_2");
		crate::argon_info!("argon_3");

		Test::test().context("blah blah blah")?;

		Ok(())
	}
}
