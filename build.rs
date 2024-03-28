use anyhow::Result;
use self_update::backends::github::Update;
use std::{env, fs::File, path::PathBuf};

fn main() -> Result<()> {
	let out_path = PathBuf::from(env::var("OUT_DIR")?).join("Argon.rbxm");

	if !cfg!(feature = "plugin") {
		File::create(out_path)?;
		return Ok(());
	}

	Update::configure()
		.repo_owner("argon-rbx")
		.repo_name("argon-roblox")
		.bin_name("Argon.rbxm")
		.bin_install_path(out_path)
		.target("")
		.build()?
		.download()?;

	Ok(())
}
