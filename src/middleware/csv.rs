use anyhow::Result;
use csv::{ReaderBuilder, WriterBuilder};
use rbx_dom_weak::{types::Variant, ustr, HashMapExt, UstrMap};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, path::Path};

use crate::{core::snapshot::Snapshot, vfs::Vfs, Properties};

#[derive(Debug, Default, Serialize, Deserialize)]
struct LocalizationEntry {
	key: Option<String>,
	context: Option<String>,
	example: Option<String>,
	source: Option<String>,
	values: HashMap<String, String>,
}

#[profiling::function]
pub fn read_csv(path: &Path, vfs: &Vfs) -> Result<Snapshot> {
	let contents = vfs.read(path)?;

	if contents.is_empty() {
		return Ok(Snapshot::new().with_class("LocalizationTable"));
	}

	let mut reader = ReaderBuilder::new()
		.has_headers(true)
		.flexible(true)
		.from_reader(contents.as_slice());

	let headers = reader.headers()?.clone();
	let mut entries = Vec::new();

	for record in reader.records() {
		let mut entry = LocalizationEntry::default();

		for (index, field) in record?.iter().enumerate() {
			if field.is_empty() {
				continue;
			}

			let header = headers.get(index);

			if let Some(header) = header {
				match header {
					"Key" => entry.key = Some(field.to_owned()),
					"Source" => entry.source = Some(field.to_owned()),
					"Context" => entry.context = Some(field.to_owned()),
					"Example" => entry.example = Some(field.to_owned()),
					_ => {
						entry.values.insert(header.to_owned(), field.to_owned());
					}
				}
			}
		}

		if entry.key.is_some() || entry.source.is_some() {
			entries.push(entry);
		}
	}

	let contents = serde_json::to_string(&entries)?;

	let mut properties = UstrMap::new();
	properties.insert(ustr("Contents"), Variant::String(contents));

	Ok(Snapshot::new()
		.with_class("LocalizationTable")
		.with_properties(properties))
}

#[profiling::function]
pub fn write_csv(mut properties: Properties, path: &Path, vfs: &Vfs) -> Result<Properties> {
	if let Some(Variant::String(contents)) = properties.remove(&ustr("Contents")) {
		let entries: Vec<LocalizationEntry> = serde_json::from_str(&contents)?;
		let mut contents = Vec::new();

		let mut writer = WriterBuilder::new()
			.has_headers(true)
			.flexible(true)
			.from_writer(&mut contents);

		writer.write_record(["Key", "Source", "Context", "Example"])?;

		for entry in entries {
			let mut record = vec![
				entry.key.unwrap_or_default(),
				entry.source.unwrap_or_default(),
				entry.context.unwrap_or_default(),
				entry.example.unwrap_or_default(),
			];

			for value in entry.values.values() {
				record.push(value.to_owned());
			}

			writer.write_record(&record)?;
		}

		writer.flush()?;
		drop(writer);

		vfs.write(path, &contents)?;
	}

	Ok(properties)
}
