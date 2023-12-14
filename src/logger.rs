use dialoguer::console::{style, Style, StyledObject};
use dialoguer::theme::Theme;
use dialoguer::Confirm;
use env_logger::{fmt::Color, Builder, WriteStyle};
use log::{Level, LevelFilter};
use std::{env, fmt, io::Write};

pub fn init(level_filter: LevelFilter, color_choice: WriteStyle) {
	let mut builder = Builder::new();

	builder.format(move |buffer, record| {
		if record.level() > level_filter && record.target() != "argon_log" {
			return Ok(());
		}

		let color = match record.level() {
			Level::Error => Color::Red,
			Level::Warn => Color::Yellow,
			Level::Info => Color::Green,
			Level::Debug => Color::Cyan,
			Level::Trace => Color::White,
		};

		let mut style = buffer.style();
		style.set_color(color).set_bold(true);

		if record.target() == "argon_log" {
			writeln!(
				buffer,
				"{}: {:?}",
				style.value(record.level().to_string()),
				record.args()
			)
		} else {
			writeln!(
				buffer,
				"{}: {:?} [{}:{}]",
				style.value(record.level().to_string()),
				record.args(),
				record.module_path().unwrap_or("error").replace("::", "."),
				record.line().unwrap_or(0)
			)
		}
	});

	if level_filter == LevelFilter::Off {
		builder.filter_level(LevelFilter::Off);
	} else if level_filter <= LevelFilter::Info {
		builder.filter_level(LevelFilter::Info);
	} else {
		builder.filter_level(level_filter);
	}

	builder.write_style(color_choice);

	builder.init();
}

pub fn prompt(prompt: &str, default: bool) -> bool {
	let log_style = env::var("RUST_LOG_STYLE").unwrap_or("always".to_string());

	let theme = match log_style.as_str() {
		"always" => PromptTheme::color(),
		_ => PromptTheme::no_color(),
	};

	let result = Confirm::with_theme(&theme)
		.with_prompt(prompt)
		.default(default)
		.interact();

	result.unwrap_or(default)
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
			prompt_prefix: style("PROMPT".to_string()).for_stderr().blue().bold(),
			prompt_suffix: style("·".to_string()).for_stderr().black().bright(),
			yes_style: Style::new().for_stderr().green(),
			no_style: Style::new().for_stderr().red(),
			none_style: Style::new().for_stderr().cyan(),
			hint_style: Style::new().for_stderr().black().bright(),
		}
	}

	fn no_color() -> Self {
		Self {
			prompt_style: Style::new().for_stderr(),
			prompt_prefix: style("PROMPT".to_string()).for_stderr(),
			prompt_suffix: style("·".to_string()).for_stderr(),
			yes_style: Style::new().for_stderr(),
			no_style: Style::new().for_stderr(),
			none_style: Style::new().for_stderr(),
			hint_style: Style::new().for_stderr(),
		}
	}
}
