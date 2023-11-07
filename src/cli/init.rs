use anyhow::Result;
use clap::Parser;

#[derive(Parser)]
pub struct Init {}

impl Init {
	pub fn main(self) -> Result<()> {
		Ok(())
	}
}
