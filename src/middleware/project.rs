use std::path::Path;

use crate::{
	core::{meta::Meta, snapshot::Snapshot},
	vfs::Vfs,
};

pub fn main(path: &Path, meta: &Meta, vfs: &Vfs) -> Option<Snapshot> {
	println!("{:?}", path);
	None
}
