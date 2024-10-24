use glob::{glob, Paths, Pattern, PatternError};
use serde::{de::Error, Deserialize, Deserializer, Serialize, Serializer};
use std::{
	fmt::{self, Debug, Formatter},
	path::{Path, PathBuf},
};

#[derive(Clone, PartialEq)]
pub struct Glob {
	pattern: Pattern,
}

impl Glob {
	pub fn new(pattern: &str) -> Result<Self, PatternError> {
		#[cfg(not(target_os = "windows"))]
		let pattern = pattern.replace('\\', "/");

		#[cfg(target_os = "windows")]
		let pattern = pattern.replace('/', "\\");

		Ok(Self {
			pattern: Pattern::new(&pattern)?,
		})
	}

	pub fn from_path(path: &Path) -> Result<Self, PatternError> {
		Self::new(path.to_str().unwrap_or_default())
	}

	pub fn matches(&self, str: &str) -> bool {
		self.pattern.matches(str)
	}

	pub fn matches_path(&self, path: &Path) -> bool {
		self.pattern.matches_path(path)
	}

	pub fn matches_path_with_dir(&self, path: &Path) -> bool {
		let matches = self.pattern.matches_path(path);

		if !matches && self.pattern.as_str().ends_with("/**") {
			if let Ok(pattern) = Pattern::new(self.pattern.as_str().strip_suffix("/**").unwrap()) {
				return pattern.matches_path(path);
			} else {
				return false;
			}
		}

		matches
	}

	pub fn first(&self) -> Option<PathBuf> {
		let path = glob(self.pattern.as_str()).unwrap().next();

		if let Some(path) = path {
			return Some(path.unwrap_or_default());
		}

		None
	}

	pub fn iter(&self) -> Paths {
		glob(self.pattern.as_str()).unwrap()
	}

	pub fn as_str(&self) -> &str {
		self.pattern.as_str()
	}
}

impl Serialize for Glob {
	fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
		serializer.serialize_str(self.pattern.as_str())
	}
}

impl<'de> Deserialize<'de> for Glob {
	fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
		let pattern = String::deserialize(deserializer)?;
		Self::new(&pattern).map_err(Error::custom)
	}
}

impl Debug for Glob {
	fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
		write!(f, "{}", self.pattern.as_str())
	}
}
