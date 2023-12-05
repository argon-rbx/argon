use crate::{fs::Fs, project::Project};

pub struct Core {
	pub project: Project,
	pub fs: Fs,
}

impl Core {
	pub fn new(project: Project, fs: Fs) -> Self {
		Self { project, fs }
	}
}
