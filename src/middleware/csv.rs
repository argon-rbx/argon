use anyhow::Result;
use csv::ReaderBuilder;
use rbx_dom_weak::types::Variant;
use serde::Serialize;
use std::{collections::HashMap, path::Path};

use crate::{core::snapshot::Snapshot, vfs::Vfs};

#[derive(Default, Serialize, Debug)]
struct LocalizationEntry {
	key: Option<String>,
	context: Option<String>,
	example: Option<String>,
	source: Option<String>,
	values: HashMap<String, String>,
}

#[profiling::function]
pub fn snapshot_csv(path: &Path, vfs: &Vfs) -> Result<Snapshot> {
	let mut reader = ReaderBuilder::new()
		.has_headers(true)
		.flexible(true)
		.from_reader(vfs.reader(path)?);

	let headers = reader.headers()?.clone();
	let mut entries = vec![];

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
					"Context" => entry.context = Some(field.to_owned()),
					"Example" => entry.example = Some(field.to_owned()),
					"Source" => entry.source = Some(field.to_owned()),
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

	let mut properties = HashMap::new();
	properties.insert(String::from("Contents"), Variant::String(contents));

	Ok(Snapshot::new()
		.with_class("LocalizationTable")
		.with_properties(properties))
}
