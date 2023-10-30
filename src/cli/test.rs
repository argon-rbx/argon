use clap::Parser;

#[derive(Parser)]
pub struct Command {}

impl Command {
	pub fn run(self) {
		log::error!("1");
		log::warn!("2");
		log::info!("3");
		log::debug!("4");
		log::trace!("5");

		crate::argon_error!("argon_1");
		crate::argon_warn!("argon_2");
		crate::argon_info!("argon_3");

		let _ = crate::confirm::prompt("test prompt", false);
	}
}
