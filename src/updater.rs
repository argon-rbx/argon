use anyhow::{anyhow, Result};
use colored::Colorize;
use log::{debug, trace, warn};
use self_update::{backends::github::Update, cargo_crate_version, version::bump_is_greater, Extract};
use serde::{Deserialize, Serialize};
use std::io;
use std::{fs, sync::Once, time::SystemTime};

use crate::{
	argon_error, argon_info,
	constants::TEMPLATES_VERSION,
	installer::{get_plugin_version, install_templates},
	logger,
	util::{self, get_plugin_path},
};

static UPDATE_FORCED: Once = Once::new();

#[derive(Serialize, Deserialize)]
pub struct UpdateStatus {
	pub last_checked: SystemTime,
	pub plugin_version: String,
	pub templates_version: u8,
	pub vscode_version: String,
}

pub fn get_status() -> Result<UpdateStatus> {
	let path = util::get_argon_dir()?.join("update.toml");

	if path.exists() {
		match toml::from_str(&fs::read_to_string(&path)?) {
			Ok(status) => return Ok(status),
			Err(_) => warn!("Update status file is corrupted! Creating new one.."),
		}
	}

	let status = UpdateStatus {
		last_checked: SystemTime::UNIX_EPOCH,
		plugin_version: get_plugin_version(),
		templates_version: TEMPLATES_VERSION,
		vscode_version: cargo_crate_version!().to_string(),
	};

	fs::write(path, toml::to_string(&status)?)?;

	Ok(status)
}

pub fn set_status(status: &UpdateStatus) -> Result<()> {
	let path = util::get_argon_dir()?.join("update.toml");

	fs::write(path, toml::to_string(status)?)?;

	Ok(())
}

fn update_cli(prompt: bool, force: bool) -> Result<bool> {
	let style = util::get_progress_style();
	let current_version = cargo_crate_version!();

	// Get target-specific asset name for our convention
	let asset_name = {
		#[cfg(target_os = "linux")]
		{
			"argon-{version}-linux-x86_64.zip".to_string()
		}
		#[cfg(all(target_os = "macos", target_arch = "x86_64"))]
		{
			"argon-{version}-macos-x86_64.zip".to_string()
		}
		#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
		{
			"argon-{version}-macos-aarch64.zip".to_string()
		}
		#[cfg(target_os = "windows")]
		{
			"argon-{version}-windows-x86_64.zip".to_string()
		}
	};

	// For M1/M2 Macs, try direct download method first since releases often don't include aarch64 assets
	#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
	{
		trace!("Detected M1/M2 Mac (aarch64), attempting direct download first");
		// Try direct download first for Apple Silicon
		let download_url = format!(
			"https://github.com/LupaHQ/argon/releases/download/{}/{}",
			current_version,
			asset_name.replace("{version}", current_version)
		);

		if !prompt {
			argon_info!(
				"Checking for updates using direct download from {}",
				download_url.bold()
			);
		}
	}

	// Configure the update
	let mut update = Update::configure()
		.repo_owner("LupaHQ")
		.repo_name("argon")
		.bin_name("argon")
		.show_download_progress(true)
		.set_progress_style(style.0, style.1)
		// Use the identifier to match the specific asset name pattern
		.identifier(&asset_name)
		.no_confirm(true);

	// Check the latest release first
	let release = match update.build() {
		Ok(u) => match u.get_latest_release() {
			Ok(release) => release,
			Err(err) => {
				trace!("Failed to get latest release: {}", err);
				// If we can't get the release or there are no assets, we'll use a direct download fallback
				return download_direct_fallback(current_version, prompt, force, &asset_name);
			}
		},
		Err(err) => {
			trace!("Failed to build update: {}", err);
			return download_direct_fallback(current_version, prompt, force, &asset_name);
		}
	};

	if bump_is_greater(current_version, &release.version)? || force {
		if !prompt
			|| logger::prompt(
				&format!(
					"New Argon version: {} is available! Would you like to update?",
					release.version.bold()
				),
				true,
			) {
			if !prompt {
				argon_info!("New Argon version: {} is available! Updating..", release.version.bold());
			}

			// Try to update through normal GitHub release asset
			match update.build()?.update() {
				Ok(_) => {
					argon_info!(
						"CLI updated! Restart the program to apply changes. Visit {} to read the changelog",
						"https://argon.wiki/changelog/argon".bold()
					);
					return Ok(true);
				}
				Err(err) => {
					trace!("Failed to update through asset: {}", err);
					// If update through asset fails, try direct download
					return download_direct_fallback(&release.version, prompt, true, &asset_name);
				}
			}
		} else {
			trace!("Argon is out of date!");
		}
	} else {
		trace!("Argon is up to date!");
	}

	Ok(false)
}

// Fallback download function that directly fetches the release without relying on GitHub release assets
fn download_direct_fallback(version: &str, prompt: bool, force: bool, asset_pattern: &str) -> Result<bool> {
	let temp_dir = tempfile::tempdir()?;
	let download_url = format!(
		"https://github.com/LupaHQ/argon/releases/download/{}/{}",
		version,
		asset_pattern.replace("{version}", version)
	);

	argon_info!("Attempting direct download from {}", download_url);

	// Create a reqwest client
	let client = reqwest::blocking::Client::new();

	// Check if the file exists by sending a HEAD request
	let response = match client.head(&download_url).send() {
		Ok(resp) => {
			if !resp.status().is_success() {
				argon_error!("File does not exist at URL: {}", download_url);
				return Ok(false);
			}
			client.get(&download_url).send()?
		}
		Err(err) => {
			argon_error!("Failed to check file existence: {}", err);
			return Ok(false);
		}
	};

	if !response.status().is_success() {
		argon_error!("Failed to download update: HTTP status {}", response.status());
		return Ok(false);
	}

	// Download the file
	let target_file = temp_dir.path().join(asset_pattern.replace("{version}", version));
	let mut file = std::fs::File::create(&target_file)?;
	io::copy(&mut response.bytes()?.as_ref(), &mut file)?;

	// Extract the binary
	let extract_dir = temp_dir.path().join("extracted");
	std::fs::create_dir_all(&extract_dir)?;

	// Use the self_update extract functionality
	let bin_name = format!("argon{}", if cfg!(windows) { ".exe" } else { "" });

	#[cfg(any(target_os = "windows", target_os = "macos", target_os = "linux"))]
	{
		Extract::from_source(&target_file).extract_file(&extract_dir, &bin_name)?;

		// Get the executable path
		let new_exe = extract_dir.join(&bin_name);

		// Replace the current executable
		self_replace::self_replace(new_exe)?;

		argon_info!(
			"CLI updated! Restart the program to apply changes. Visit {} to read the changelog",
			"https://argon.wiki/changelog/argon".bold()
		);
		Ok(true)
	}

	#[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
	{
		argon_error!("Unsupported platform for direct download");
		Ok(false)
	}
}

fn update_plugin(status: &mut UpdateStatus, prompt: bool, force: bool) -> Result<bool> {
	let style = util::get_progress_style();
	let current_version = &status.plugin_version;
	let plugin_path = get_plugin_path()?;

	let update = Update::configure()
		.repo_owner("LupaHQ")
		.repo_name("argon-roblox")
		.bin_name("Argon.rbxm")
		.target("")
		.show_download_progress(true)
		.set_progress_style(style.0, style.1)
		.bin_install_path(plugin_path)
		.build()?;

	let release = update.get_latest_release()?;

	if bump_is_greater(current_version, &release.version)? || force {
		if !prompt
			|| logger::prompt(
				&format!(
					"New version of Argon plugin: {} is available! Would you like to update?",
					release.version.bold()
				),
				true,
			) {
			if !prompt {
				argon_info!(
					"New version of Argon plugin: {} is available! Updating..",
					release.version.bold()
				);
			}

			match update.download() {
				Ok(_) => {
					argon_info!(
						"Roblox plugin updated! Make sure you have {} setting enabled to see changes. Visit {} to read the changelog",
						"Reload plugins on file changed".bold(),
						"https://argon.wiki/changelog/argon-roblox".bold()
					);

					status.plugin_version = release.version;
					return Ok(true);
				}
				Err(err) => argon_error!("Failed to update Argon plugin: {}", err),
			}
		} else {
			trace!("Argon plugin is out of date!");
		}
	} else {
		trace!("Argon plugin is up to date!");
	}

	Ok(false)
}

fn update_templates(status: &mut UpdateStatus, prompt: bool, force: bool) -> Result<bool> {
	if status.templates_version < TEMPLATES_VERSION || force {
		if !prompt || logger::prompt("Default templates have changed! Would you like to update?", true) {
			if !prompt {
				argon_info!("Default templates have changed! Updating..",);
			}

			install_templates(true)?;

			status.templates_version = TEMPLATES_VERSION;

			return Ok(true);
		} else {
			trace!("Templates are out of date!");
		}
	} else {
		trace!("Project templates are up to date!");
	}

	Ok(false)
}

fn update_vscode(status: &mut UpdateStatus, prompt: bool, force: bool) -> Result<bool> {
	let current_version = &status.vscode_version;

	// Get the latest release from GitHub
	let release = reqwest::blocking::get("https://api.github.com/repos/LupaHQ/argon-vscode/releases/latest")?
		.json::<serde_json::Value>()?;

	let latest_version = release["tag_name"]
		.as_str()
		.ok_or_else(|| anyhow!("Failed to get tag name"))?;

	// Remove leading 'v' if present
	let latest_version = latest_version.trim_start_matches('v');

	if bump_is_greater(current_version, latest_version)? || force {
		if !prompt
			|| logger::prompt(
				&format!(
					"New version of Argon VS Code extension: {} is available! Would you like to update?",
					latest_version.bold()
				),
				true,
			) {
			if !prompt {
				argon_info!(
					"New version of Argon VS Code extension: {} is available! Updating..",
					latest_version.bold()
				);
			}

			// Find the VSIX asset
			let assets = release["assets"]
				.as_array()
				.ok_or_else(|| anyhow!("Failed to get assets"))?;
			let vsix_asset = assets
				.iter()
				.find(|asset| asset["name"].as_str().is_some_and(|name| name.ends_with(".vsix")))
				.ok_or_else(|| anyhow!("Failed to find VSIX asset"))?;

			let download_url = vsix_asset["browser_download_url"]
				.as_str()
				.ok_or_else(|| anyhow!("Failed to get download URL"))?;

			// Download the VSIX file to a temporary location
			let temp_dir = std::env::temp_dir();
			let vsix_path = temp_dir.join(format!("argon-{}.vsix", latest_version));

			argon_info!("Downloading VS Code extension...");

			let mut response = reqwest::blocking::get(download_url)?;
			let mut file = std::fs::File::create(&vsix_path)?;
			std::io::copy(&mut response, &mut file)?;

			// Install the extension using the VS Code CLI
			argon_info!("Installing VS Code extension...");

			let output = std::process::Command::new("code")
				.arg("--install-extension")
				.arg(&vsix_path)
				.arg("--force")
				.output();

			match output {
				Ok(output) if output.status.success() => {
					// Clean up the temporary file
					let _ = std::fs::remove_file(vsix_path);

					argon_info!(
						"VS Code extension updated! Please reload VS Code to apply changes. Visit {} to read the changelog",
						"https://argon.wiki/changelog/argon-vscode".bold()
					);
					status.vscode_version = latest_version.to_string();
					return Ok(true);
				}
				Ok(output) => {
					let stderr = String::from_utf8_lossy(&output.stderr);
					argon_error!("Failed to install VS Code extension: {}", stderr);
				}
				Err(err) => {
					argon_error!("Failed to run VS Code CLI: {}", err);
				}
			}
		} else {
			trace!("Argon VS Code extension is out of date!");
		}
	} else {
		trace!("Argon VS Code extension is up to date!");
	}

	Ok(false)
}

pub fn check_for_updates(plugin: bool, templates: bool, prompt: bool) -> Result<()> {
	let mut status = get_status()?;

	if UPDATE_FORCED.is_completed() {
		return Ok(());
	}

	if status.last_checked.elapsed()?.as_secs() < 3600 {
		debug!("Update check already performed within the last hour");
		return Ok(());
	}

	update_cli(prompt, false)?;

	if plugin {
		update_plugin(&mut status, prompt, false)?;
	}

	if templates {
		update_templates(&mut status, prompt, false)?;
	}

	status.last_checked = SystemTime::now();
	set_status(&status)?;

	Ok(())
}

pub fn manual_update(cli: bool, plugin: bool, templates: bool, vscode: bool, force: bool) -> Result<bool> {
	UPDATE_FORCED.call_once(|| {});

	let mut status = get_status()?;
	let mut updated = false;

	if cli && update_cli(false, force)? {
		updated = true;
	}

	if plugin && update_plugin(&mut status, false, force)? {
		updated = true;
	}

	if templates && update_templates(&mut status, false, force)? {
		updated = true;
	}

	if vscode && update_vscode(&mut status, false, force)? {
		updated = true;
	}

	status.last_checked = SystemTime::now();
	set_status(&status)?;

	Ok(updated)
}
