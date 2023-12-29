use anyhow::Result;
use csv::ReaderBuilder;
use serde::Serialize;
use std::{collections::HashMap, path::Path};

#[derive(Default, Serialize, Debug)]
struct LocalizationEntry {
	key: Option<String>,
	context: Option<String>,
	example: Option<String>,
	source: Option<String>,
	values: HashMap<String, String>,
}

pub fn read_localization_table(path: &Path) -> Result<String> {
	let mut reader = ReaderBuilder::new().has_headers(true).flexible(true).from_path(path)?;
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

	Ok(serde_json::to_string(&entries)?)
}
