use anyhow::Result;
use rbx_dom_weak::{types::Variant, ustr, HashMapExt, UstrMap};
use rmp_serde::Deserializer;
use rmpv::Value;
use std::path::Path;

use crate::{core::snapshot::Snapshot, vfs::Vfs};

#[profiling::function]
pub fn read_msgpack(path: &Path, vfs: &Vfs) -> Result<Snapshot> {
	let msgpack = vfs.read(path)?;

	if msgpack.is_empty() {
		return Ok(Snapshot::new().with_class("ModuleScript"));
	}

	let mut deserializer = Deserializer::from_read_ref(&msgpack).with_human_readable();
	let msgpack: Value = serde::Deserialize::deserialize(&mut deserializer)?;

	let lua = format!("return {}", msgpack_to_lua(&msgpack));

	let mut properties = UstrMap::new();
	properties.insert(ustr("Source"), Variant::String(lua));

	Ok(Snapshot::new().with_class("ModuleScript").with_properties(properties))
}

fn msgpack_to_lua(value: &Value) -> String {
	let mut lua = String::new();

	match value {
		Value::Nil => lua.push_str("nil"),
		Value::Boolean(b) => lua.push_str(&b.to_string()),
		Value::Integer(i) => lua.push_str(&i.to_string()),
		Value::F32(f) => {
			if f.is_infinite() {
				lua.push_str("math.huge")
			} else {
				lua.push_str(&f.to_string())
			}
		}
		Value::F64(f) => {
			if f.is_infinite() {
				lua.push_str("math.huge")
			} else {
				lua.push_str(&f.to_string())
			}
		}
		Value::String(s) => lua.push_str(&format!("\"{}\"", &escape_chars(s.as_str().unwrap_or_default()))),
		Value::Binary(b) => lua.push_str(&String::from_utf8_lossy(b)),
		Value::Array(a) => {
			lua.push('{');

			for v in a {
				lua.push_str(&msgpack_to_lua(v));
				lua.push(',');
			}

			lua.push('}');
		}
		Value::Map(t) => {
			lua.push('{');

			for (k, v) in t {
				lua.push_str(&format!("[{}] = ", &msgpack_to_lua(k)));
				lua.push_str(&msgpack_to_lua(v));
				lua.push(',');
			}

			lua.push('}');
		}
		Value::Ext(_, _) => {}
	}

	lua
}

fn escape_chars(string: &str) -> String {
	let mut validated = String::new();

	for char in string.chars() {
		match char {
			'\n' => validated.push_str("\\n"),
			'\t' => validated.push_str("\\t"),
			'\r' => validated.push_str("\\r"),
			'\\' => validated.push_str("\\\\"),
			'"' => validated.push_str("\\\""),
			_ => validated.push(char),
		}
	}

	validated
}
