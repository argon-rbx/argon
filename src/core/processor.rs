use anyhow::Result;
use rbx_xml::DecodeOptions;
use std::{
	collections::HashMap,
	fs::{self, File},
	io::BufReader,
	path::Path,
	sync::{Arc, Mutex},
};

use super::{
	dom::Dom,
	instance::Instance,
	middleware::{FileKind, ModelKind},
	queue::Queue,
};
use crate::{
	lock,
	messages::{Create, Delete, Message, Update},
	project::Project,
	rbx_path::RbxPath,
	util,
};

pub struct Processor {
	pub dom: Arc<Mutex<Dom>>,
	pub queue: Arc<Mutex<Queue>>,
	pub project: Arc<Mutex<Project>>,
}

impl Processor {
	pub fn new(dom: Arc<Mutex<Dom>>, queue: Arc<Mutex<Queue>>, project: Arc<Mutex<Project>>) -> Self {
		Self { dom, queue, project }
	}

	pub fn init(&self, path: &Path) -> Result<()> {
		let ext = util::get_file_ext(path);
		let is_dir = ext.is_empty();

		if !self.is_valid(path, ext, is_dir) {
			return Ok(());
		}

		let rbx_paths = lock!(self.project).path_map.get_vec(path).unwrap().clone();

		for rbx_path in rbx_paths {
			let mut dom = lock!(self.dom);
			let mut cur_rbx_path = RbxPath::new();

			for (index, comp) in rbx_path.iter().enumerate() {
				cur_rbx_path.push(comp);

				if dom.contains(&cur_rbx_path) {
					continue;
				} else if index != rbx_path.len() - 1 {
					let class = self.get_class(&FileKind::Dir, None, Some(&cur_rbx_path))?;
					let parent = self.get_parent(&cur_rbx_path);

					let properties = if path.exists() {
						self.get_properties(&FileKind::Dir, path)?
					} else {
						HashMap::new()
					};

					let instance = Instance::new(comp)
						.with_class(&class)
						.with_properties(properties)
						.with_rbx_path(&cur_rbx_path)
						.with_path(path);

					dom.insert(instance, &parent);
				} else {
					let kind = self.get_file_kind(comp, ext, is_dir)?;
					let class = self.get_class(&kind, None, Some(&cur_rbx_path))?;
					let parent = self.get_parent(&cur_rbx_path);

					let properties = if path.is_file() {
						self.get_properties(&kind, path)?
					} else {
						HashMap::new()
					};

					let instance = Instance::new(comp)
						.with_class(&class)
						.with_properties(properties)
						.with_rbx_path(&cur_rbx_path)
						.with_path(path);

					dom.insert(instance, &parent);
				}
			}
		}

		Ok(())
	}

	pub fn create(&self, path: &Path, dom_only: bool) -> Result<()> {
		let ext = util::get_file_ext(path);
		let is_dir = path.is_dir();

		if !self.is_valid(path, ext, is_dir) {
			return Ok(());
		}

		let file_stem = util::get_file_stem(path);
		let rbx_paths = self.get_rbx_paths(path, file_stem, ext)?;
		let file_kind = self.get_file_kind(file_stem, ext, is_dir)?;

		let mut dom = lock!(self.dom);
		let mut queue = lock!(self.queue);

		for rbx_path in rbx_paths {
			match file_kind {
				FileKind::ChildScript(_)
				| FileKind::Script(_)
				| FileKind::JsonModule
				| FileKind::StringValue
				| FileKind::LocalizationTable
				| FileKind::Dir => {
					let class = self.get_class(&file_kind, Some(path), None)?;
					let properties = self.get_properties(&file_kind, path)?;
					let name = self.get_name(&file_kind, &rbx_path, file_stem);
					let parent = self.get_parent(&rbx_path);

					let instance = Instance::new(&name)
						.with_class(&class)
						.with_properties(properties.clone())
						.with_rbx_path(&rbx_path)
						.with_path(path);

					dom.insert(instance, &parent);

					if !dom_only {
						queue.push(
							Message::Create(Create {
								class,
								path: rbx_path,
								properties,
							}),
							None,
						);
					}
				}
				FileKind::InstanceData => {
					let instance = dom.get_mut(&rbx_path).unwrap();
					let properties = self.get_properties(&file_kind, path)?;

					instance.properties = properties.clone();

					if !dom_only {
						queue.push(
							Message::Update(Update {
								path: rbx_path,
								properties,
							}),
							None,
						);
					}
				}
				FileKind::Model(ref kind) => {
					let reader = BufReader::new(File::open(path)?);
					let mut model = if *kind == ModelKind::Binary {
						rbx_binary::from_reader(reader)?
					} else {
						rbx_xml::from_reader(reader, DecodeOptions::default())?
					};

					model.root_mut().name = file_stem.to_owned();

					let new_instances = dom.append(&mut model, &rbx_path, path);

					if !dom_only {
						for (path, instances) in new_instances {
							for instance in instances {
								queue.push(
									Message::Create(Create {
										class: instance.class.clone(),
										path: path.to_owned(),
										properties: instance.properties.clone(),
									}),
									None,
								);
							}
						}
					}
				}
			}
		}

		drop(dom);
		drop(queue);

		if path.is_dir() {
			for entry in fs::read_dir(path)? {
				self.create(&entry?.path(), dom_only)?;
			}
		}

		Ok(())
	}

	pub fn delete(&self, path: &Path) -> Result<()> {
		let ext = util::get_file_ext(path);

		if !self.is_valid(path, ext, true) {
			return Ok(());
		}

		let file_stem = util::get_file_stem(path);
		let rbx_paths = self.get_rbx_paths(path, file_stem, ext)?;

		let mut dom = lock!(self.dom);
		let mut queue = lock!(self.queue);

		for rbx_path in rbx_paths {
			if ext == "json" && self.is_instance_data(file_stem) {
				let instance = dom.get_mut(&rbx_path).unwrap();
				instance.properties = HashMap::new();

				queue.push(
					Message::Update(Update {
						path: rbx_path.clone(),
						properties: HashMap::new(),
					}),
					None,
				);
			} else if dom.remove(&rbx_path) {
				queue.push(Message::Delete(Delete { path: rbx_path }), None);
			}
		}

		Ok(())
	}

	pub fn write(&self, path: &Path) -> Result<()> {
		let ext = util::get_file_ext(path);

		if !self.is_valid(path, ext, false) {
			return Ok(());
		}

		let file_stem = util::get_file_stem(path);
		let rbx_paths = self.get_rbx_paths(path, file_stem, ext)?;
		let file_kind = self.get_file_kind(file_stem, ext, false)?;

		let mut dom = lock!(self.dom);
		let mut queue = lock!(self.queue);

		for rbx_path in rbx_paths {
			match file_kind {
				FileKind::Model(ref kind) => {
					if dom.remove(&rbx_path) {
						queue.push(Message::Delete(Delete { path: rbx_path.clone() }), None);

						let reader = BufReader::new(File::open(path)?);
						let mut model = if *kind == ModelKind::Binary {
							rbx_binary::from_reader(reader)?
						} else {
							rbx_xml::from_reader(reader, DecodeOptions::default())?
						};

						model.root_mut().name = file_stem.to_owned();

						let new_instances = dom.append(&mut model, &rbx_path, path);

						for (path, instances) in new_instances {
							for instance in instances {
								queue.push(
									Message::Create(Create {
										class: instance.class.clone(),
										path: path.to_owned(),
										properties: instance.properties.clone(),
									}),
									None,
								);
							}
						}
					}
				}
				_ => {
					let instance = dom.get_mut(&rbx_path).unwrap();
					let properties = self.get_properties(&file_kind, path)?;

					instance.properties = properties.clone();

					queue.push(
						Message::Update(Update {
							path: rbx_path,
							properties,
						}),
						None,
					);
				}
			}
		}

		Ok(())
	}
}
