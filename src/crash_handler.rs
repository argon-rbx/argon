use backtrace::Backtrace;
use colored::Colorize;
use log::{error, trace, LevelFilter};
use open;
use panic_message::get_panic_info_message;
use std::{env, panic, process};

use crate::{argon_error, logger, util};

const MAX_BACKTRACE_LEN: usize = 6500;

pub fn hook() {
	panic::set_hook(Box::new(|panic_info| {
		let message = get_panic_info_message(panic_info).unwrap_or("Failed to get panic info message");
		let backtrace_enabled = util::env_backtrace();
		let verbosity = util::env_verbosity();

		let mut report = String::from("> This crash report was automatically generated by Argon");
		report.push_str("\n\nDetails:\n--------\n");
		report.push_str("*Provide extra details about your problem or what happened here*");

		argon_error!("{}", "Argon has crashed!".bold());

		report.push_str("\n\nMessage:\n----------\n");
		report.push_str(message);

		argon_error!("{}: {}", "Message".bold(), message);

		report.push_str("\n\nLocation:\n----------\n");

		if let Some(location) = panic_info.location() {
			report.push_str(location.file());
			report.push_str(": ");
			report.push_str(&location.line().to_string());

			argon_error!("{}: {}: {}", "Location".bold(), location.file(), location.line());
		} else {
			report.push_str("Failed to get panic info location");
		}

		report.push_str("\n\nVerbose output:\n----------------\n");

		if verbosity != LevelFilter::Trace {
			report.push_str("Verbose output disabled, run Argon with `-vvvv` flag to enable");

			argon_error!(
				"{}: Run Argon with {} flag to show detailed tracing",
				"Verbosity".bold(),
				"-vvvv".bold()
			);
		} else {
			report.push_str("```\nPast your verbose output here\n```");
		}

		report.push_str("\n\nBacktrace:\n-----------\n");

		if backtrace_enabled {
			let backtrace = Backtrace::new();

			argon_error!("{}:\n{:?}", "Backtrace".bold(), backtrace);

			// Temporary solution for broken OsString parser
			let mut backtrace = format!("{backtrace:?}");
			backtrace = backtrace.replace("             ", "\t\t");
			backtrace = backtrace.replace("    ", "\t");
			backtrace = backtrace.replace("   ", "");
			backtrace = backtrace.replace("  ", "");
			backtrace = backtrace.replace('&', "ptr");

			if backtrace.len() > MAX_BACKTRACE_LEN {
				backtrace.drain(..MAX_BACKTRACE_LEN);
				backtrace.push_str("\n...\n");
			}

			report.push_str("```\n");
			report.push_str(&backtrace);
			report.push_str("```");
		} else {
			report.push_str("Backtrace disabled, run Argon with `--backtrace` flag to enable");

			argon_error!(
				"{}: Run Argon with {} flag to show full backtrace\n",
				"Backtrace".bold(),
				"--backtrace".bold()
			);
		}

		let report_issue = logger::prompt(
			"Would you like to create new issue on GitHub with current report?",
			false,
		);

		if report_issue {
			let mut url = env!("CARGO_PKG_REPOSITORY").to_owned();
			url.push_str(&format!("/issues/new?title=Argon crashed: {message}&body="));

			#[cfg(not(target_os = "windows"))]
			url.push_str(&report);

			#[cfg(target_os = "windows")]
			url.push_str(&report.replace('\n', "%0A").replace('\t', "%09"));

			match open::that(url) {
				Err(err) => error!("Failed to launch system browser: {err}"),
				Ok(()) => trace!("Browser launched successfully"),
			}
		}

		process::exit(1)
	}));
}
