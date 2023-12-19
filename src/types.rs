use serde::{Serialize, Serializer};
use std::fmt::{self, Debug, Display};

use crate::ROBLOX_SEPARATOR;

#[derive(Clone)]
pub struct RobloxPath {
	components: Vec<String>,
}

impl RobloxPath {
	pub fn new() -> Self {
		Self { components: vec![] }
	}

	pub fn from(path: &str) -> Self {
		let mut components = vec![];

		for component in path.split(ROBLOX_SEPARATOR) {
			components.push(component.to_owned());
		}

		Self { components }
	}

	pub fn push(&mut self, path: &str) {
		if path.is_empty() {
			return;
		}

		self.components.push(path.to_owned());
	}

	pub fn pop(&mut self) -> Option<String> {
		self.components.pop()
	}

	pub fn len(&self) -> usize {
		self.components.len()
	}

	pub fn is_empty(&self) -> bool {
		self.components.is_empty()
	}
}

impl Serialize for RobloxPath {
	fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
		serializer.serialize_str(&self.to_string())
	}
}

impl Display for RobloxPath {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "{}", self.components.join(&ROBLOX_SEPARATOR.to_string()))
	}
}

impl Debug for RobloxPath {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "{}", self.components.join(&ROBLOX_SEPARATOR.to_string()))
	}
}

#[derive(Debug, Clone, Serialize)]
pub enum RobloxKind {
	ServerScript,
	ClientScript,
	ModuleScript,
	Other,
}
