use anyhow::Result;
use dirs;
use log::{debug, trace, warn};
use self_update::{backends::github::Update, cargo_crate_version, version::bump_is_greater};
use serde::{Deserialize, Serialize};
use std::env;
use std::{fs, sync::Once, time::SystemTime};
use yansi::Paint;

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

pub fn update_cli(_force: bool, _show_output: bool) -> Result<bool, self_update::errors::Error> {
	// Check if we're running from VS Code
	let exe_path = std::env::current_exe().unwrap_or_default();
	let exe_path_str = exe_path.to_string_lossy();

	// Print debug info to verify execution path
	println!("DEBUG: Current executable path: {}", exe_path_str);

	// Platform-specific VS Code detection
	#[cfg(target_os = "macos")]
	let is_vscode = exe_path_str.contains("/Code.app/");

	#[cfg(target_os = "windows")]
	let is_vscode = exe_path_str.contains("\\Microsoft VS Code\\") || exe_path_str.contains("\\Code\\");

	#[cfg(target_os = "linux")]
	let is_vscode = exe_path_str.contains("/vscode/") || exe_path_str.contains("/code/");

	#[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
	let is_vscode = false;

	println!("DEBUG: Detected VS Code environment: {}", is_vscode);
	trace!("Exe path: {}", exe_path_str);
	trace!("Is VS Code: {}", is_vscode);

	// Get current exe path
	let current_exe = env::current_exe()?;
	trace!("Current executable path: {:?}", current_exe);

	// Get user home directory for better argon binary detection
	let home_dir = dirs::home_dir().expect("Could not determine home directory");
	let argon_bin_path = home_dir.join(".argon").join("bin").join("argon");

	println!("DEBUG: Home directory path: {:?}", home_dir);
	println!("DEBUG: Argon system binary path exists: {}", argon_bin_path.exists());

	// If called from VS Code, use the system binary path
	let install_path = if is_vscode && argon_bin_path.exists() {
		println!(
			"DEBUG: VS Code detected, updating system installation at: {:?}",
			argon_bin_path
		);
		trace!(
			"VS Code detected, updating system installation at: {:?}",
			argon_bin_path
		);
		argon_bin_path.clone()
	} else {
		println!("DEBUG: Updating current executable at: {:?}", current_exe);
		trace!("Updating current executable at: {:?}", current_exe);
		current_exe.clone()
	};

	let style = util::get_progress_style();
	let _current_version = cargo_crate_version!();

	// Simple update configuration without architecture specifics
	let mut update_configure = Update::configure();
	update_configure
		.repo_owner("LupaHQ")
		.repo_name("argon")
		.bin_name("argon")
		.bin_install_path(&install_path)
		.show_download_progress(true)
		.set_progress_style(style.0.clone(), style.1.clone())
		.no_confirm(true);

	// Print debug info about target architecture
	#[cfg(target_arch = "aarch64")]
	println!("DEBUG: Running on aarch64 architecture");
	#[cfg(target_arch = "x86_64")]
	println!("DEBUG: Running on x86_64 architecture");

	// Debug info about target platform
	#[cfg(target_os = "macos")]
	println!("DEBUG: Running on macOS");
	#[cfg(target_os = "windows")]
	println!("DEBUG: Running on Windows");
	#[cfg(target_os = "linux")]
	println!("DEBUG: Running on Linux");

	// Try different targets for Apple Silicon
	#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
	{
		println!("DEBUG: Apple Silicon detected, attempting update with specific targets...");
		let targets_to_try = [
			"aarch64-apple-darwin",
			"arm64-apple-darwin",  // Alias sometimes used
			"x86_64-apple-darwin", // Rosetta fallback
		];

		let mut last_error = None;

		for target in targets_to_try {
			println!("DEBUG: Trying target: {}", target);

			// Re-create the builder for this specific target attempt
			let mut target_configure = Update::configure();
			target_configure
				.repo_owner("LupaHQ")
				.repo_name("argon")
				.bin_name("argon")
				.bin_install_path(&install_path)
				.show_download_progress(true)
				.set_progress_style(style.0.clone(), style.1.clone())
				.no_confirm(true);

			println!("DEBUG: Attempting update for target: {}", target);
			let result = target_configure.target(target).build()?.update();
			println!(
				"DEBUG: Update attempt result for target {}: {:?}",
				target,
				result.is_ok()
			);

			if result.is_ok() {
				argon_info!("{}", Paint::green("Argon CLI updated successfully! 🚀"));
				return Ok(true); // Success, exit early
			}
			println!("DEBUG: Target {} failed: {:?}", target, result.as_ref().err());
			last_error = result.err(); // Store the error from this attempt
		}

		// If all targets failed
		println!("DEBUG: All target options failed.");
		if let Some(err) = last_error {
			let release = update_configure.build()?.get_latest_release().ok();
			let available_assets = release.map_or_else(
				|| "Could not fetch release info".to_string(),
				|r| r.assets.iter().map(|a| a.name.clone()).collect::<Vec<_>>().join(", "),
			);

			argon_error!(
				"Failed to update Argon: {}. No suitable binary found for your architecture. Available assets: {}",
				err,
				available_assets
			);
			Err(err)
		} else {
			// This case should technically be unreachable
			argon_error!("Failed to update Argon: Unknown error occurred after trying all targets.");
			Err(self_update::errors::Error::Update(
				"Update failed after trying all targets, but no specific error was captured.".to_string(),
			))
		}
	}

	// For other architectures, use standard update
	#[cfg(not(all(target_os = "macos", target_arch = "aarch64")))]
	{
		println!("DEBUG: Standard architecture detected, attempting standard update...");
		let build_result = update_configure.build();
		println!("DEBUG: Build result for standard update: {:?}", build_result.is_ok());

		if let Ok(updater) = build_result {
			let update_result = updater.update();
			println!("DEBUG: Update result for standard update: {:?}", update_result.is_ok());
			match update_result {
				Ok(_) => {
					argon_info!("{}", Paint::green("Argon CLI updated successfully! 🚀"));
					Ok(true)
				}
				Err(e) => {
					println!("DEBUG: Standard update failed: {}", e);
					argon_error!("Failed to update Argon: {}", e);
					Err(e)
				}
			}
		} else {
			let err = build_result.err().unwrap(); // We know it failed
			println!("DEBUG: Failed to build standard updater: {}", err);
			argon_error!("Failed to configure Argon update: {}", err);
			Err(err)
		}
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
		.set_progress_style(style.0.clone(), style.1.clone())
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
						Paint::bold(&"Reload plugins on file changed"),
						Paint::bold(&"https://argon.wiki/changelog/argon-roblox")
					);

					status.plugin_version = release.version;
					Ok(true)
				}
				Err(err) => {
					println!("DEBUG: update_plugin failed: {}", err);
					argon_error!("Failed to update Argon plugin: {}", err);
					Ok(false)
				}
			}
		} else {
			trace!("Argon plugin is out of date!");
			Ok(false)
		}
	} else {
		trace!("Argon plugin is up to date!");
		Ok(false)
	}
}

fn update_templates(status: &mut UpdateStatus, prompt: bool, force: bool) -> Result<bool> {
	if status.templates_version < TEMPLATES_VERSION || force {
		if !prompt || logger::prompt("Default templates have changed! Would you like to update?", true) {
			if !prompt {
				argon_info!("Default templates have changed! Updating..",);
			}

			install_templates(true)?;

			status.templates_version = TEMPLATES_VERSION;

			Ok(true)
		} else {
			trace!("Templates are out of date!");
			Ok(false)
		}
	} else {
		trace!("Project templates are up to date!");
		Ok(false)
	}
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

	if let Some(current) = get_vscode_version() {
		println!("DEBUG: Current VS Code extension version detected: {}", current);
		trace!("Current VS Code extension version: {}", current);
		status.vscode_version = current;
	} else {
		println!(
			"DEBUG: Could not detect current VS Code extension version, using stored: {}",
			status.vscode_version
		);
		trace!(
			"Could not detect current VS Code extension version, using stored: {}",
			status.vscode_version
		);
	}

	let current_version = &status.vscode_version;
	println!("DEBUG: Current version to compare against: {}", current_version);

	println!("DEBUG: Fetching latest release from GitHub");
	trace!("Fetching latest VS Code extension release from GitHub");
	let client = reqwest::blocking::Client::builder().user_agent("argon-cli").build()?;

	let response = match client
		.get("https://api.github.com/repos/LupaHQ/argon-vscode/releases/latest")
		.send()
	{
		Ok(resp) => resp,
		Err(err) => {
			println!("DEBUG: Failed to send request for latest release: {}", err);
			trace!("Failed to get latest release information: {}", err);
			return Ok(false); // Early exit if network request fails
		}
	};

	if !response.status().is_success() {
		println!("DEBUG: GitHub API request failed with status: {}", response.status());
		trace!("GitHub API request failed with status: {}", response.status());
		return Ok(false); // Early exit on bad status
	}

	let release: serde_json::Value = match response.json() {
		Ok(json) => {
			println!("DEBUG: Successfully parsed GitHub API response");
			json
		}
		Err(err) => {
			println!("DEBUG: Failed to parse GitHub API response: {}", err);
			trace!("Failed to parse GitHub API response: {}", err);
			return Ok(false); // Early exit on parse error
		}
	};

	let latest_version_str = match release["tag_name"].as_str() {
		Some(tag) => tag.trim_start_matches('v').to_string(),
		None => {
			println!("DEBUG: Failed to get tag name from release");
			trace!("Failed to get tag name from release");
			return Ok(false);
		}
	};
	let latest_version = &latest_version_str; // Borrow for comparison

	println!(
		"DEBUG: Comparing versions - current: {}, latest: {}",
		current_version, latest_version
	);
	trace!("Latest VS Code extension version: {}", latest_version);

	let is_greater = bump_is_greater(current_version, latest_version);
	println!("DEBUG: Is latest version greater? {:?}", is_greater);

	let update_needed = match is_greater {
		Ok(result) => result || force,
		Err(err) => {
			println!("DEBUG: Failed to compare versions: {}", err);
			trace!("Failed to compare versions: {}", err);
			force // If comparison fails, only update if forced
		}
	};
	println!("DEBUG: Update needed? {} (force={})", update_needed, force);

	if update_needed {
		if !prompt
			|| logger::prompt(
				&format!(
					"New version of Argon VS Code extension: {} is available! Would you like to update?",
					Paint::bold(latest_version)
				),
				true,
			) {
			if !prompt {
				argon_info!(
					"New version of Argon VS Code extension: {} is available! Updating..",
					Paint::bold(latest_version)
				);
			}

			let assets = match release["assets"].as_array() {
				Some(assets) => assets,
				None => {
					trace!("Failed to get assets from release");
					return Ok(false);
				}
			};

			let vsix_asset = match assets.iter().find(|asset| {
				asset
					.get("name")
					.and_then(|n| n.as_str())
					.is_some_and(|name| name.ends_with(".vsix"))
			}) {
				Some(asset) => asset,
				None => {
					trace!("Failed to find VSIX asset in release");
					return Ok(false);
				}
			};

			let download_url = match vsix_asset.get("browser_download_url").and_then(|url| url.as_str()) {
				Some(url) => url.to_string(),
				None => {
					trace!("Failed to get download URL from asset");
					return Ok(false);
				}
			};

			let temp_dir = std::env::temp_dir();
			let vsix_path = temp_dir.join(format!("argon-{}.vsix", latest_version));

			if vsix_path.exists() {
				if let Err(err) = std::fs::remove_file(&vsix_path) {
					trace!("Failed to remove existing VSIX file: {}", err);
				}
			}

			argon_info!("Downloading VS Code extension...");
			println!("DEBUG: Downloading from URL: {}", download_url); // Debug URL
			trace!("Downloading from URL: {}", download_url);

			// Use a closure for download logic to handle intermediate errors cleanly
			let download_result = || -> Result<()> {
				let mut response = client.get(&download_url).send()?;
				if !response.status().is_success() {
					anyhow::bail!("Failed to download: HTTP status {}", response.status());
				}
				let mut file = std::fs::File::create(&vsix_path)?;
				std::io::copy(&mut response, &mut file)?;
				Ok(())
			};

			if let Err(err) = download_result() {
				println!("DEBUG: Download failed: {}", err);
				argon_error!("Failed to download VS Code extension: {}", err);
				return Ok(false);
			}

			argon_info!("Installing VS Code extension...");
			trace!("Running: code --install-extension {} --force", vsix_path.display());

			if !vsix_path.exists() {
				println!("DEBUG: VSIX file not found after download: {}", vsix_path.display());
				argon_error!("VSIX file not found at {}", vsix_path.display());
				return Ok(false);
			}

			let metadata = match std::fs::metadata(&vsix_path) {
				Ok(meta) => meta,
				Err(err) => {
					println!("DEBUG: Failed to get VSIX file metadata: {}", err);
					argon_error!("Failed to get VSIX file metadata: {}", err);
					return Ok(false);
				}
			};

			if metadata.len() == 0 {
				println!("DEBUG: Downloaded VSIX file is empty");
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
					let stdout = String::from_utf8_lossy(&output.stdout);
					let stderr = String::from_utf8_lossy(&output.stderr);
					println!("DEBUG: VS Code install stdout: {}", stdout);
					println!("DEBUG: VS Code install stderr: {}", stderr);
					trace!("VS Code stdout: {}", stdout);
					trace!("VS Code stderr: {}", stderr);

					if output.status.success() {
						let _ = std::fs::remove_file(vsix_path);
						argon_info!(
							"VS Code extension updated! Please reload VS Code to apply changes. Visit {} to read the changelog",
							Paint::bold(&"https://argon.wiki/changelog/argon-vscode")
						);
						status.vscode_version = latest_version_str; // Update status with owned String
						return Ok(true);
					} else {
						argon_error!("Failed to install VS Code extension: {}", stderr);
					}
				}
				Err(err) => {
					println!("DEBUG: Failed to run VS Code CLI: {}", err);
					argon_error!(
						"Failed to run VS Code CLI: {}. Ensure 'code' command is in your system PATH.",
						err
					);
				}
			}
		} else {
			println!("DEBUG: User declined VS Code update.");
			trace!("User declined update.");
		}
	} else {
		println!("DEBUG: VS Code extension already up to date.");
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

	// Update CLI first, as it might contain fixes for other update processes
	if cli {
		argon_info!("Checking for CLI updates...");
		if update_cli(force, false)? {
			updated = true;
		}
	}

	// Then update other components
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
