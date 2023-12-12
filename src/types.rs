#[derive(Debug, Clone)]
pub struct RobloxPath {
	components: Vec<String>,
}

impl RobloxPath {
	pub fn new() -> Self {
		Self { components: vec![] }
	}

	// pub fn from(path: &str) -> Self {
	// 	let mut components = vec![];

	// 	for component in path.split(ROBLOX_SEPARATOR) {
	// 		components.push(component.to_owned());
	// 	}

	// 	Self { components }
	// }

	// pub fn to_string(&self) -> String {
	// 	self.components.join(ROBLOX_SEPARATOR)
	// }

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
