use colored::{Color, Colorize};
use dialoguer::console::{style, Style, StyledObject};
use dialoguer::theme::Theme;
use dialoguer::Confirm;
use env_logger::{Builder, WriteStyle};
use log::{Level, LevelFilter};
use std::fmt::{Display, Formatter};
use std::{fmt, io::Write};

use crate::util;

// These Argon logs ignore verbosity level, aside of `Off`
#[macro_export]
macro_rules! argon_error {
    ($($arg:tt)+) => (log::log!(target: "argon_log", log::Level::Error, $($arg)+))
}

#[macro_export]
macro_rules! argon_warn {
    ($($arg:tt)+) => (log::log!(target: "argon_log", log::Level::Warn, $($arg)+))
}

#[macro_export]
macro_rules! argon_info {
    ($($arg:tt)+) => (log::log!(target: "argon_log", log::Level::Info, $($arg)+))
}

pub fn init(verbosity: LevelFilter, log_style: WriteStyle) {
	let mut builder = Builder::new();

	builder.format(move |buffer, record| {
		if record.level() > verbosity && record.target() != "argon_log" {
			return Ok(());
		}

		let color = match record.level() {
			Level::Error => Color::Red,
			Level::Warn => Color::Yellow,
			Level::Info => Color::Green,
			Level::Debug => Color::Cyan,
			Level::Trace => Color::White,
		};

		if record.target() == "argon_log" {
			writeln!(
				buffer,
				"{}: {:?}",
				record.level().to_string().color(color).bold(),
				record.args()
			)
		} else {
			writeln!(
				buffer,
				"{}: {:?} [{}:{}]",
				record.level().to_string().color(color).bold(),
				record.args(),
				record.module_path().unwrap(),
				record.line().unwrap()
			)
		}
	});

	if verbosity == LevelFilter::Off {
		builder.filter_level(LevelFilter::Off);
	} else if verbosity <= LevelFilter::Info {
		builder.filter_level(LevelFilter::Info);
	} else {
		builder.filter_level(verbosity);
	}

	builder.write_style(log_style);

	// We want to see only important logs from these crates
	builder.filter_module("notify_debouncer_full", LevelFilter::Warn);
	builder.filter_module("notify", LevelFilter::Warn);

	builder.filter_module("actix_server", LevelFilter::Warn);
	builder.filter_module("actix_http", LevelFilter::Warn);
	builder.filter_module("reqwest", LevelFilter::Warn);
	builder.filter_module("rustls", LevelFilter::Warn);
	builder.filter_module("hyper", LevelFilter::Warn);
	builder.filter_module("mio", LevelFilter::Warn);

	builder.filter_module("rbx_binary", LevelFilter::Warn);

	builder.init();
}

pub fn prompt(prompt: &str, default: bool) -> bool {
	if util::env_yes() {
		return default;
	}

	let theme = match util::env_log_style() {
		WriteStyle::Always => PromptTheme::color(),
		_ => PromptTheme::no_color(),
	};

	let result = Confirm::with_theme(&theme)
		.with_prompt(prompt)
		.default(default)
		.interact();

	result.unwrap_or(default)
}

pub struct Table {
	rows: Vec<Vec<String>>,
	columns: Vec<usize>,
}

impl Table {
	pub fn new() -> Self {
		Self {
			rows: Vec::new(),
			columns: Vec::new(),
		}
	}

	pub fn add_row(&mut self, row: Vec<String>) {
		for (i, column) in row.iter().enumerate() {
			if self.columns.len() <= i {
				self.columns.push(column.len());
			} else if self.columns[i] < column.len() {
				self.columns[i] = column.len();
			}
		}

		self.rows.push(row);
	}

	pub fn set_header(&mut self, row: Vec<&str>) {
		self.add_row(row.iter().map(|s| s.to_string()).collect());
	}
}

impl Display for Table {
	fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
		let mut header = String::new();
		let mut separator = String::new();

		for (i, row) in self.rows[0].iter().enumerate() {
			header.push_str(&format!("| {0: <1$} ", row.bold(), self.columns[i]));
		}

		for column in &self.columns {
			separator.push_str(&format!("|{0:-<1$}", "", column + 2));
		}

		write!(f, "{}|\n{}|\n", header, separator)?;

		for row in self.rows.iter().skip(1) {
			for (i, column) in row.iter().enumerate() {
				write!(f, "| {0: <1$} ", column, self.columns[i])?;
			}

			writeln!(f, "|")?;
		}

		Ok(())
	}
}

pub struct PromptTheme {
	pub prompt_style: Style,
	pub prompt_prefix: StyledObject<String>,
	pub prompt_suffix: StyledObject<String>,
	pub yes_style: Style,
	pub no_style: Style,
	pub none_style: Style,
	pub hint_style: Style,
}

impl Theme for PromptTheme {
	fn format_confirm_prompt(&self, f: &mut dyn fmt::Write, prompt: &str, _: Option<bool>) -> fmt::Result {
		if !prompt.is_empty() {
			write!(f, "{}: {} ", &self.prompt_prefix, self.prompt_style.apply_to(prompt))?;
		}

		write!(f, "{}", self.hint_style.apply_to("(y/n)"))
	}

	fn format_confirm_prompt_selection(
		&self,
		f: &mut dyn fmt::Write,
		prompt: &str,
		selection: Option<bool>,
	) -> fmt::Result {
		if !prompt.is_empty() {
			write!(f, "{}: {} ", &self.prompt_prefix, self.prompt_style.apply_to(prompt))?;
		}

		let selection = selection.map(|s| if s { "yes" } else { "no" });

		match selection {
			Some(selection) => match selection {
				"yes" => write!(f, "{} {}", &self.prompt_suffix, self.yes_style.apply_to(selection)),
				"no" => write!(f, "{} {}", &self.prompt_suffix, self.no_style.apply_to(selection)),
				_ => write!(f, "{} {}", &self.prompt_suffix, self.none_style.apply_to(selection)),
			},
			None => {
				write!(f, "{} {}", &self.prompt_suffix, self.none_style.apply_to("none"))
			}
		}
	}
}

impl PromptTheme {
	fn color() -> Self {
		Self {
			prompt_style: Style::new().for_stderr(),
			prompt_prefix: style("PROMPT".into()).for_stderr().blue().bold(),
			prompt_suffix: style("·".into()).for_stderr().black().bright(),
			yes_style: Style::new().for_stderr().green(),
			no_style: Style::new().for_stderr().red(),
			none_style: Style::new().for_stderr().cyan(),
			hint_style: Style::new().for_stderr().black().bright(),
		}
	}

	fn no_color() -> Self {
		Self {
			prompt_style: Style::new().for_stderr(),
			prompt_prefix: style("PROMPT".into()).for_stderr(),
			prompt_suffix: style("·".into()).for_stderr(),
			yes_style: Style::new().for_stderr(),
			no_style: Style::new().for_stderr(),
			none_style: Style::new().for_stderr(),
			hint_style: Style::new().for_stderr(),
		}
	}
}
