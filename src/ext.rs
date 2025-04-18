use anyhow::{bail, Result};
use env_logger::WriteStyle;
use path_clean::PathClean;
use rbx_reflection::{PropertyDescriptor, PropertyKind, PropertySerialization};
use std::{
	env,
	fmt::Display,
	path::{Path, PathBuf},
};

/// Collection of extension methods for `Path`
pub trait PathExt {
	fn resolve(&self) -> Result<PathBuf>;
	fn to_string(&self) -> String;
	fn get_name(&self) -> &str;
	fn get_stem(&self) -> &str;
	fn get_ext(&self) -> &str;
	fn get_parent(&self) -> &Path;
	fn len(&self) -> usize;
	fn is_empty(&self) -> bool;
	fn contains(&self, pat: &[&str]) -> bool;
}

impl PathExt for Path {
	fn resolve(&self) -> Result<PathBuf> {
		if self.is_absolute() {
			return Ok(self.to_owned());
		}

		let current_dir = env::current_dir()?;
		let absolute = current_dir.join(self);

		Ok(absolute.clean())
	}

	fn to_string(&self) -> String {
		self.to_str().unwrap_or_default().to_owned()
	}

	fn get_name(&self) -> &str {
		self.file_name().unwrap_or_default().to_str().unwrap_or_default()
	}

	fn get_stem(&self) -> &str {
		if !self.is_dir() {
			self.file_stem().unwrap_or_default().to_str().unwrap_or_default()
		} else {
			self.get_name()
		}
	}

	fn get_ext(&self) -> &str {
		if !self.is_dir() {
			self.extension().unwrap_or_default().to_str().unwrap_or_default()
		} else {
			""
		}
	}

	fn get_parent(&self) -> &Path {
		self.parent().unwrap_or(self)
	}

	fn len(&self) -> usize {
		self.components().count()
	}

	fn is_empty(&self) -> bool {
		self.len() == 0
	}

	fn contains(&self, pattern: &[&str]) -> bool {
		let mut index = 0;

		for comp in self.components() {
			if pattern[index] == comp.as_os_str() {
				index += 1;

				if index == pattern.len() {
					return true;
				}
			} else if index > 0 {
				return false;
			}
		}

		false
	}
}

/// Additional methods for `anyhow::Error`, similar to `context` and `with_context`
pub trait ResultExt<T, E> {
	fn desc<D>(self, desc: D) -> Result<T, anyhow::Error>
	where
		D: Display + Send + Sync + 'static;

	fn with_desc<C, F>(self, f: F) -> Result<T, anyhow::Error>
	where
		C: Display + Send + Sync + 'static,
		F: FnOnce() -> C;
}

impl<T, E> ResultExt<T, E> for Result<T, E>
where
	E: Display + Send + Sync + 'static,
{
	fn desc<D>(self, desc: D) -> Result<T, anyhow::Error>
	where
		D: Display + Send + Sync + 'static,
	{
		match self {
			Ok(ok) => Ok(ok),
			Err(err) => {
				bail!("{}: {}", desc, err);
			}
		}
	}

	fn with_desc<C, F>(self, desc: F) -> Result<T, anyhow::Error>
	where
		C: Display + Send + Sync + 'static,
		F: FnOnce() -> C,
	{
		match self {
			Ok(ok) => Ok(ok),
			Err(err) => {
				bail!("{}: {}", desc(), err);
			}
		}
	}
}

/// `to_string` implementation for `WriteStyle`
pub trait WriteStyleExt {
	fn to_string(&self) -> String;
}

impl WriteStyleExt for WriteStyle {
	fn to_string(&self) -> String {
		let write_style = match self {
			WriteStyle::Always => "always",
			WriteStyle::Auto => "auto",
			WriteStyle::Never => "never",
		};

		String::from(write_style)
	}
}

/// Additional methods for `rbx_reflection::PropertyDescriptor`
pub trait PropertyDescriptorExt {
	fn get_custom_serialization(&self) -> Option<String>;
}

impl PropertyDescriptorExt for PropertyDescriptor<'_> {
	fn get_custom_serialization(&self) -> Option<String> {
		match &self.kind {
			PropertyKind::Canonical {
				serialization: PropertySerialization::SerializesAs(data_type),
			} => Some(data_type.to_string()),
			_ => None,
		}
	}
}
