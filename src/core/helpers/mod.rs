use crate::Properties;

mod migrations;

pub mod syncback;

#[inline]
pub fn apply_migrations(class: &str, properties: &mut Properties) {
	migrations::apply(class, properties);
}
