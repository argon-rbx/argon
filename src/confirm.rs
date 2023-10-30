use dialoguer::console::{style, Style, StyledObject};
use dialoguer::theme::Theme;
use dialoguer::{Confirm, Error};
use std::{env, fmt};

pub fn prompt(prompt: &str, default: bool) -> Result<bool, Error> {
	let log_style = env::var("RUST_LOG_STYLE").unwrap_or("always".to_string());

	let theme = match log_style.as_str() {
		"always" => ConfirmTheme::color(),
		_ => ConfirmTheme::no_color(),
	};

	Confirm::with_theme(&theme)
		.with_prompt(prompt)
		.default(default)
		.interact()
}

pub struct ConfirmTheme {
	pub prompt_style: Style,
	pub prompt_prefix: StyledObject<String>,
	pub prompt_suffix: StyledObject<String>,
	pub yes_style: Style,
	pub no_style: Style,
	pub none_style: Style,
	pub hint_style: Style,
}

impl Theme for ConfirmTheme {
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

		let selection = selection.map(|x| if x { "yes" } else { "no" });

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

impl ConfirmTheme {
	fn color() -> ConfirmTheme {
		ConfirmTheme {
			prompt_style: Style::new().for_stderr(),
			prompt_prefix: style("PROMPT".to_string()).for_stderr().blue().bold(),
			prompt_suffix: style("·".to_string()).for_stderr().black().bright(),
			yes_style: Style::new().for_stderr().green(),
			no_style: Style::new().for_stderr().red(),
			none_style: Style::new().for_stderr().cyan(),
			hint_style: Style::new().for_stderr().black().bright(),
		}
	}

	fn no_color() -> ConfirmTheme {
		ConfirmTheme {
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
