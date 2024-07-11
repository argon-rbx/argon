use anyhow::Result;
use log::{debug, error, trace};
use rbx_dom_weak::{types::Variant, InstanceBuilder, WeakDom};
use roblox_install::RobloxStudio;
use std::{
	collections::HashMap,
	fs::{self, File},
	io::BufWriter,
	path::Path,
	process,
	sync::RwLock,
	thread,
};

use crate::{ext::PathExt, util, Properties};

static INDEX: RwLock<u32> = RwLock::new(0);

pub fn save_mesh(properties: &Properties) -> Option<String> {
	let mut mesh_properties: HashMap<&str, Variant> = HashMap::new();
	mesh_properties.insert("MeshId", properties.get("MeshId")?.clone());

	if let Some(initial_size) = properties.get("InitialSize") {
		mesh_properties.insert("InitialSize", initial_size.clone());
	}

	let dom = WeakDom::new(InstanceBuilder::new("MeshPart").with_properties(mesh_properties));

	trace!("Writing MeshPart temporary binary model");

	let result = || -> Result<String> {
		let pid = process::id().to_string();
		let path = RobloxStudio::locate()?.content_path().join("argon").join(&pid);

		let index = *INDEX.read().unwrap();

		if index == 0 {
			let path = path.clone();

			thread::spawn(move || match clear(&path) {
				Ok(_) => debug!("Cleared temporary mesh models"),
				Err(err) => error!("Failed to clear temporary mesh models: {}", err),
			});
		}

		if !path.exists() {
			fs::create_dir_all(&path)?;
		}

		let name = index.to_string();
		let writer = BufWriter::new(File::create(path.join(&name))?);

		rbx_binary::to_writer(writer, &dom, &[dom.root_ref()])?;

		Ok(pid + "/" + &name)
	}();

	match result {
		Ok(name) => {
			let mut index = INDEX.write().unwrap();
			*index += 1;

			Some(name)
		}
		Err(err) => {
			error!("Failed to write MeshPart temporary model: {}", err);
			None
		}
	}
}

fn clear(path: &Path) -> Result<()> {
	let ignore_name = path.get_name();
	let parent = path.get_parent();

	if !parent.exists() {
		return Ok(());
	}

	for entry in fs::read_dir(parent)? {
		let path = entry?.path();
		let name = path.get_name();

		if name == ignore_name {
			continue;
		}

		if path.is_dir() && !name.parse().is_ok_and(util::process_exists) {
			fs::remove_dir_all(&path)?;
		}
	}

	Ok(())
}
