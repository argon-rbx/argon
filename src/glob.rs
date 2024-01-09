use glob::{glob, Paths, Pattern, PatternError};
use serde::{de::Error, Deserialize, Deserializer, Serialize, Serializer};
use std::{
	fmt::{self, Debug, Formatter},
	path::PathBuf,
};

#[derive(Clone)]
pub struct Glob {
	pattern: Pattern,
}

impl Glob {
	pub fn new(glob: &str) -> Result<Self, PatternError> {
		Ok(Self {
			pattern: Pattern::new(glob)?,
		})
	}

	pub fn matches(&self, path: &str) -> bool {
		self.pattern.matches(path)
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
