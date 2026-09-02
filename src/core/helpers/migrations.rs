// Temporary module for ContentId to Content migration of selected properties

use rbx_dom_weak::{
	types::{Content, Variant},
	Ustr,
};
use std::{collections::HashMap, sync::OnceLock};

use crate::Properties;

pub type Migrations = HashMap<&'static str, HashMap<&'static str, &'static str>>;

fn get_migrations() -> &'static Migrations {
	static MIGRATIONS: OnceLock<Migrations> = OnceLock::new();

	MIGRATIONS.get_or_init(|| {
		HashMap::from([
			("ImageLabel", HashMap::from([("Image", "ImageContent")])),
			("ImageButton", HashMap::from([("Image", "ImageContent")])),
			(
				"MeshPart",
				HashMap::from([("MeshId", "MeshContent"), ("TextureID", "TextureContent")]),
			),
			("BaseWrap", HashMap::from([("CageMeshId", "CageMeshContent")])),
			(
				"WrapLayer",
				HashMap::from([("ReferenceMeshId", "ReferenceMeshContent")]),
			),
		])
	})
}

pub fn apply<'a>(class: &str, properties: &'a mut Properties) -> Option<&'a mut Properties> {
	let migration = get_migrations().get(class)?;

	for (old, new) in migration {
		let new = Ustr::from(new);

		if properties.contains_key(&new) {
			continue;
		}

		if let Some(Variant::ContentId(value)) = properties.remove(&Ustr::from(old)) {
			properties.insert(new, Content::from(value.as_str()).into());
		}
	}

	Some(properties)
}
