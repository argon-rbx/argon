use env_logger::{fmt::Color, Builder, WriteStyle};
use log::{Level, LevelFilter};
use std::io::Write;

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
				"{}: {}:{} - {:?}",
				style.value(record.level().to_string()),
				record.module_path().unwrap_or("error").replace("::", "."),
				record.line().unwrap_or(0),
				record.args()
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
