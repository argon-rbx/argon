use anyhow::Result;
use colored::Colorize;
use log::{debug, trace, warn};
use self_update::{backends::github::Update, cargo_crate_version, version::bump_is_greater, Extract};
use serde::{Deserialize, Serialize};
use std::io;
use std::{fs, sync::Once, time::SystemTime};
use std::env;
use dirs;

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

	// Try to get installed VS Code extension version
	let vscode_version = match get_vscode_version() {
		Some(version) => {
			trace!("Using detected VS Code extension version: {}", version);
			version
		}
		None => {
			trace!("Could not detect VS Code extension version, using default version");
			"0.0.0".to_string() // Use a baseline version if not detected
		}
	};

	let status = UpdateStatus {
		last_checked: SystemTime::UNIX_EPOCH,
		plugin_version: get_plugin_version(),
		templates_version: TEMPLATES_VERSION,
		vscode_version,
	};

	fs::write(path, toml::to_string(&status)?)?;

	Ok(status)
}

pub fn set_status(status: &UpdateStatus) -> Result<()> {
	let path = util::get_argon_dir()?.join("update.toml");

	fs::write(path, toml::to_string(status)?)?;

	Ok(())
}

pub fn update_cli(
	force: bool,
	show_output: bool,
) -> Result<bool, self_update::errors::Error> {
	// Check if we're running from VS Code
	let exe_path = std::env::current_exe().unwrap_or_default();
	let exe_path_str = exe_path.to_string_lossy();
	
	// Platform-specific VS Code detection
	#[cfg(target_os = "macos")]
	let is_vscode = exe_path_str.contains("/Code.app/");
	
	#[cfg(target_os = "windows")]
	let is_vscode = exe_path_str.contains("\\Microsoft VS Code\\") || 
					exe_path_str.contains("\\Code\\");
	
	#[cfg(target_os = "linux")]
	let is_vscode = exe_path_str.contains("/vscode/") || 
					exe_path_str.contains("/code/");
	
	#[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
	let is_vscode = false;
	
	trace!("Exe path: {}", exe_path_str);
	trace!("Is VS Code: {}", is_vscode);

	// Get current exe path
	let current_exe = env::current_exe()?;
	trace!("Current executable path: {:?}", current_exe);
	
	// Get user home directory for better argon binary detection
	let home_dir = dirs::home_dir().expect("Could not determine home directory");
	let argon_bin_path = home_dir.join(".argon").join("bin").join("argon");
	
	// If called from VS Code, use the system binary path
	let install_path = if is_vscode && argon_bin_path.exists() {
		trace!("VS Code detected, updating system installation at: {:?}", argon_bin_path);
		argon_bin_path.clone()
	} else {
		trace!("Updating current executable at: {:?}", current_exe);
		current_exe.clone()
	};
	
	let style = util::get_progress_style();
	let current_version = cargo_crate_version!();
	
	// Simple update configuration without architecture specifics
	let update = Update::configure()
		.repo_owner("LupaHQ")
		.repo_name("argon")
		.bin_name("argon")
		.bin_install_path(install_path)
		.show_download_progress(true)
		.set_progress_style(style.0, style.1)
		.no_confirm(true)
		.build()?;

	let release = match update.get_latest_release() {
		Ok(release) => release,
		Err(err) => {
			trace!("Failed to get latest release: {}", err);
			return Ok(false);
		}
	};

	if bump_is_greater(&current_version, &release.version)? || force {
		if !show_output
			|| logger::prompt(
				&format!(
					"New Argon version: {} is available! Would you like to update?",
					release.version.bold()
				),
				true,
			) {
			if !show_output {
				argon_info!("New Argon version: {} is available! Updating..", release.version.bold());
			}

			match update.update() {
				Ok(_) => {
					argon_info!(
						"CLI updated! Restart the program to apply changes. Visit {} to read the changelog",
						"https://argon.wiki/changelog/argon".bold()
					);
					return Ok(true);
				}
				Err(err) => {
					trace!("Failed to update through asset: {}", err);
					argon_error!("Failed to update Argon: {}", err);
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

// Get the currently installed VS Code extension version
fn get_vscode_version() -> Option<String> {
	// Try to get version using VS Code CLI
	let output = std::process::Command::new("code")
		.arg("--list-extensions")
		.arg("--show-versions")
		.output();

	match output {
		Ok(output) if output.status.success() => {
			let stdout = String::from_utf8_lossy(&output.stdout);
			trace!("VS Code extensions list: {}", stdout);
			// Look for the extension - might be "lemonade-labs.argon@x.y.z" or "argon@x.y.z"
			for line in stdout.lines() {
				if line.contains("lemonade-labs.argon@") || line.contains("argon@") {
					if let Some(version) = line.split('@').nth(1) {
						trace!("Found VS Code extension version: {}", version.trim());
						return Some(version.trim().to_string());
					}
				}
			}
			trace!("VS Code extension not found in installed extensions");
			None
		}
		Ok(output) => {
			trace!(
				"VS Code CLI returned error: {}",
				String::from_utf8_lossy(&output.stderr)
			);
			None
		}
		Err(err) => {
			trace!("Could not run VS Code CLI: {}", err);
			None
		}
	}
}

fn update_vscode(status: &mut UpdateStatus, prompt: bool, force: bool) -> Result<bool> {
	println!("DEBUG: Starting VS Code extension update process");
	trace!("Checking for VS Code extension updates");
	
	// Refresh our current version from installed extensions
	if let Some(current) = get_vscode_version() {
		println!("DEBUG: Current VS Code extension version detected: {}", current);
		trace!("Current VS Code extension version: {}", current);
		status.vscode_version = current;
	} else {
		println!("DEBUG: Could not detect current VS Code extension version, using stored: {}", status.vscode_version);
		trace!("Could not detect current VS Code extension version, using stored: {}", status.vscode_version);
	}
	
	let current_version = &status.vscode_version;
	println!("DEBUG: Current version to compare against: {}", current_version);

	// Get the latest release from GitHub
	println!("DEBUG: Fetching latest release from GitHub");
	trace!("Fetching latest VS Code extension release from GitHub");
	let client = reqwest::blocking::Client::builder()
		.user_agent("argon-cli")
		.build()?;
	
	let release = match client.get("https://api.github.com/repos/LupaHQ/argon-vscode/releases/latest").send() {
		Ok(response) => {
			println!("DEBUG: GitHub API status: {}", response.status());
			if !response.status().is_success() {
				println!("DEBUG: GitHub API request failed with status: {}", response.status());
				trace!("GitHub API request failed with status: {}", response.status());
				return Ok(false);
			}
			
			match response.json::<serde_json::Value>() {
				Ok(json) => {
					println!("DEBUG: Successfully parsed GitHub API response");
					json
				},
				Err(err) => {
					println!("DEBUG: Failed to parse GitHub API response: {}", err);
					trace!("Failed to parse GitHub API response: {}", err);
					return Ok(false);
				}
			}
		},
		Err(err) => {
			println!("DEBUG: Failed to get latest release information: {}", err);
			trace!("Failed to get latest release information: {}", err);
			return Ok(false);
		}
	};

	let latest_version = match release["tag_name"].as_str() {
		Some(tag) => {
			let version = tag.trim_start_matches('v');
			println!("DEBUG: Latest VS Code extension version: {}", version);
			version
		},
		None => {
			println!("DEBUG: Failed to get tag name from release");
			trace!("Failed to get tag name from release");
			return Ok(false);
		}
	};
	
	println!("DEBUG: Comparing versions - current: {}, latest: {}", current_version, latest_version);
	trace!("Latest VS Code extension version: {}", latest_version);

	// Compare versions and update if needed
	let is_greater = bump_is_greater(current_version, latest_version);
	println!("DEBUG: Is latest version greater? {:?}", is_greater);
	
	let update_needed = match is_greater {
		Ok(result) => {
			let needed = result || force;
			println!("DEBUG: Update needed? {} (force={})", needed, force);
			needed
		},
		Err(err) => {
			println!("DEBUG: Failed to compare versions: {}", err);
			trace!("Failed to compare versions: {}", err);
			force // If comparison fails, only update if forced
		}
	};
	
	if update_needed {
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
			let assets = match release["assets"].as_array() {
				Some(assets) => assets,
				None => {
					trace!("Failed to get assets from release");
					return Ok(false);
				}
			};

			let vsix_asset = match assets
				.iter()
				.find(|asset| asset["name"].as_str().is_some_and(|name| name.ends_with(".vsix")))
			{
				Some(asset) => asset,
				None => {
					trace!("Failed to find VSIX asset in release");
					return Ok(false);
				}
			};

			let download_url = match vsix_asset["browser_download_url"].as_str() {
				Some(url) => url,
				None => {
					trace!("Failed to get download URL from asset");
					return Ok(false);
				}
			};

			// Download the VSIX file to a temporary location
			let temp_dir = std::env::temp_dir();
			let vsix_path = temp_dir.join(format!("argon-{}.vsix", latest_version));

			// Delete existing file if it exists
			if vsix_path.exists() {
				if let Err(err) = std::fs::remove_file(&vsix_path) {
					trace!("Failed to remove existing VSIX file: {}", err);
				}
			}

			argon_info!("Downloading VS Code extension...");
			trace!("Downloading from URL: {}", download_url);

			let mut response = match client.get(download_url).send() {
				Ok(response) => {
					if !response.status().is_success() {
						argon_error!("Failed to download: HTTP status {}", response.status());
						return Ok(false);
					}
					response
				}
				Err(err) => {
					argon_error!("Failed to download VS Code extension: {}", err);
					return Ok(false);
				}
			};

			let mut file = match std::fs::File::create(&vsix_path) {
				Ok(file) => file,
				Err(err) => {
					argon_error!("Failed to create temporary file: {}", err);
					return Ok(false);
				}
			};

			match std::io::copy(&mut response, &mut file) {
				Ok(size) => trace!("Downloaded {} bytes to {}", size, vsix_path.display()),
				Err(err) => {
					argon_error!("Failed to save VS Code extension: {}", err);
					return Ok(false);
				}
			}

			// Install the extension using the VS Code CLI
			argon_info!("Installing VS Code extension...");
			trace!("Running: code --install-extension {} --force", vsix_path.display());

			// Make sure file exists and has size
			if !vsix_path.exists() {
				argon_error!("VSIX file not found at {}", vsix_path.display());
				return Ok(false);
			}

			let metadata = match std::fs::metadata(&vsix_path) {
				Ok(meta) => meta,
				Err(err) => {
					argon_error!("Failed to get VSIX file metadata: {}", err);
					return Ok(false);
				}
			};

			if metadata.len() == 0 {
				argon_error!("Downloaded VSIX file is empty");
				return Ok(false);
			}

			let output = std::process::Command::new("code")
				.arg("--install-extension")
				.arg(&vsix_path)
				.arg("--force")
				.output();

			match output {
				Ok(output) => {
					trace!("VS Code stdout: {}", String::from_utf8_lossy(&output.stdout));
					trace!("VS Code stderr: {}", String::from_utf8_lossy(&output.stderr));

					if output.status.success() {
						// Clean up the temporary file
						let _ = std::fs::remove_file(vsix_path);

						argon_info!(
							"VS Code extension updated! Please reload VS Code to apply changes. Visit {} to read the changelog",
							"https://argon.wiki/changelog/argon-vscode".bold()
						);
						status.vscode_version = latest_version.to_string();
						return Ok(true);
					} else {
						let stderr = String::from_utf8_lossy(&output.stderr);
						argon_error!("Failed to install VS Code extension: {}", stderr);
					}
				}
				Err(err) => {
					argon_error!("Failed to run VS Code CLI: {}", err);
				}
			}
		} else {
			println!("DEBUG: No update needed");
			trace!("Argon VS Code extension is up to date!");
		}
	} else {
		println!("DEBUG: No update needed");
		trace!("Argon VS Code extension is up to date!");
	}

	Ok(false)
}

pub fn check_for_updates(plugin: bool, templates: bool, prompt: bool) -> Result<()> {
	let mut status = get_status()?;

	// If we've already checked within the last hour, skip
	let now = SystemTime::now();
	let one_hour = std::time::Duration::from_secs(60 * 60);
	if now.duration_since(status.last_checked).unwrap_or(one_hour) < one_hour {
		debug!("Update check already performed within the last hour");
		return Ok(());
	}

	update_cli(false, false)?;

	if plugin {
		update_plugin(&mut status, prompt, false)?;
	}

	if templates {
		update_templates(&mut status, prompt, false)?;
	}

	// Also check for VS Code extension updates
	let _ = update_vscode(&mut status, prompt, false);

	status.last_checked = SystemTime::now();
	set_status(&status)?;

	Ok(())
}

pub fn manual_update(cli: bool, plugin: bool, templates: bool, vscode: bool, force: bool) -> Result<bool> {
	UPDATE_FORCED.call_once(|| {});

	let mut status = get_status()?;
	let mut updated = false;

	if cli {
		argon_info!("Checking for CLI updates...");
		if update_cli(force, false)? {
			updated = true;
		}
	}

	if plugin {
		argon_info!("Checking for Plugin updates...");
		if update_plugin(&mut status, false, force)? {
			updated = true;
		}
	}

	if templates {
		argon_info!("Checking for Template updates...");
		if update_templates(&mut status, false, force)? {
			updated = true;
		}
	}

	if vscode {
		argon_info!("Checking for VS Code extension updates...");
		if update_vscode(&mut status, false, force)? {
			updated = true;
		} else {
			trace!("No VS Code extension updates found or update failed");
		}
	}

	status.last_checked = SystemTime::now();
	set_status(&status)?;

	if !updated {
		argon_info!("All components are up to date!");
	}

	Ok(updated)
}
