pub mod cli;
pub mod config;
pub mod confirm;
pub mod crash_handler;
pub mod fs;
pub mod installer;
pub mod logger;
pub mod server;
pub mod session;

pub type DynResult<T> = Result<T, Box<dyn std::error::Error>>;

// TODO: needs improvements, Result support

#[macro_export]
macro_rules! unwrap_or_return {
	($e:expr) => {
		match $e {
			Some(x) => x,
			None => return,
		}
	};

	($e:expr, $r:expr) => {
		match $e {
			Some(x) => x,
			None => return $r,
		}
	};

	($e:expr) => {
		match $e {
			Ok(x) => x,
			Err(e) => e,
		}
	};
}

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
