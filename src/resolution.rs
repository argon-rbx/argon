// Based on Rojo's resolution.rs (https://github.com/rojo-rbx/rojo/blob/master/src/resolution.rs)

use anyhow::{bail, format_err};
use rbx_dom_weak::types::{
	Attributes, CFrame, Color3, Content, Enum, Font, MaterialColors, Matrix3, Tags, Variant, VariantType, Vector2,
	Vector3,
};
use rbx_reflection::{DataType, PropertyDescriptor};
use serde::{Deserialize, Serialize};
use std::{borrow::Borrow, collections::HashMap, fmt::Write};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum UnresolvedValue {
	FullyQualified(Variant),
	Ambiguous(AmbiguousValue),
}

impl UnresolvedValue {
	pub fn resolve(self, class_name: &str, prop_name: &str) -> anyhow::Result<Variant> {
		match self {
			UnresolvedValue::FullyQualified(full) => Ok(full),
			UnresolvedValue::Ambiguous(partial) => partial.resolve(class_name, prop_name),
		}
	}

	pub fn resolve_unambiguous(self) -> anyhow::Result<Variant> {
		match self {
			UnresolvedValue::FullyQualified(full) => Ok(full),
			UnresolvedValue::Ambiguous(partial) => partial.resolve_unambiguous(),
		}
	}

	pub fn as_str(&self) -> Option<&str> {
		match self {
			UnresolvedValue::Ambiguous(AmbiguousValue::String(s)) => Some(s.as_str()),
			_ => None,
		}
	}

	// Based on Uplift Games' Rojo fork (https://github.com/UpliftGames/rojo/blob/syncback-incremental/src/resolution.rs#L43)
	pub fn from_variant(variant: Variant, class_name: &str, prop_name: &str) -> Self {
		Self::Ambiguous(match variant {
			Variant::Enum(rbx_enum) => {
				if let Some(property) = find_descriptor(class_name, prop_name) {
					if let DataType::Enum(enum_name) = &property.data_type {
						let database = rbx_reflection_database::get();

						if let Some(enum_descriptor) = database.enums.get(enum_name) {
							for (variant_name, id) in &enum_descriptor.items {
								if *id == rbx_enum.to_u32() {
									return Self::Ambiguous(AmbiguousValue::String(variant_name.to_string()));
								}
							}
						}
					}
				}

				return Self::FullyQualified(variant);
			}

			Variant::Bool(bool) => AmbiguousValue::Bool(bool),

			Variant::Float32(n) => AmbiguousValue::Number(n as f64),
			Variant::Float64(n) => AmbiguousValue::Number(n),
			Variant::Int32(n) => AmbiguousValue::Number(truncate_float(n as f64)),
			Variant::Int64(n) => AmbiguousValue::Number(n as f64),

			Variant::String(str) => AmbiguousValue::String(str),
			Variant::Tags(tags) => AmbiguousValue::StringArray(tags.iter().map(|s| s.to_string()).collect()),
			Variant::Content(content) => AmbiguousValue::String(content.into_string()),

			Variant::Vector2(vector) => {
				AmbiguousValue::Array2([truncate_float(vector.x as f64), truncate_float(vector.y as f64)])
			}
			Variant::Vector3(vector) => AmbiguousValue::Array3([vector.x as f64, vector.y as f64, vector.z as f64]),
			Variant::Color3(color) => AmbiguousValue::Array3([color.r as f64, color.g as f64, color.b as f64]),

			Variant::CFrame(cf) => AmbiguousValue::Array12([
				cf.position.x as f64,
				cf.position.y as f64,
				cf.position.z as f64,
				cf.orientation.x.x as f64,
				cf.orientation.x.y as f64,
				cf.orientation.x.z as f64,
				cf.orientation.y.x as f64,
				cf.orientation.y.y as f64,
				cf.orientation.y.z as f64,
				cf.orientation.z.x as f64,
				cf.orientation.z.y as f64,
				cf.orientation.z.z as f64,
			]),

			Variant::Attributes(attr) => AmbiguousValue::Attributes(attr),
			Variant::Font(font) => AmbiguousValue::Font(font),
			Variant::MaterialColors(colors) => AmbiguousValue::MaterialColors(colors),

			_ => {
				return Self::FullyQualified(variant);
			}
		})
	}
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum AmbiguousValue {
	Bool(bool),
	String(String),
	StringArray(Vec<String>),
	Number(f64),
	Object(HashMap<String, UnresolvedValue>),
	Array2([f64; 2]),
	Array3([f64; 3]),
	Array4([f64; 4]),
	Array12([f64; 12]),
	Attributes(Attributes),
	Font(Font),
	MaterialColors(MaterialColors),
}

impl AmbiguousValue {
	pub fn resolve(self, class: &str, property: &str) -> anyhow::Result<Variant> {
		let descriptor =
			find_descriptor(class, property).ok_or_else(|| format_err!("Unknown property {}.{}", class, property))?;

		match &descriptor.data_type {
			DataType::Enum(enum_name) => {
				let descriptor = rbx_reflection_database::get()
					.enums
					.get(enum_name)
					.ok_or_else(|| format_err!("Unknown enum {}. Probably not implemented yet!", enum_name))?;

				let error = |value: &str| {
					let mut examples = descriptor
						.items
						.keys()
						.map(|value| value.borrow())
						.collect::<Vec<&str>>();
					examples.sort();

					let examples = list_examples(&examples);

					format_err!(
						"Invalid value for property {}.{}. Got {} but expected a member of the {} enum such as {}",
						class,
						property,
						value,
						enum_name,
						examples,
					)
				};

				let value = match self {
					AmbiguousValue::String(value) => value,
					unresolved => return Err(error(unresolved.describe())),
				};

				let resolved = descriptor
					.items
					.get(value.as_str())
					.ok_or_else(|| error(value.as_str()))?;

				Ok(Enum::from_u32(*resolved).into())
			}
			DataType::Value(variant) => match (variant, self) {
				(VariantType::Bool, AmbiguousValue::Bool(bool)) => Ok(bool.into()),

				(VariantType::Float32, AmbiguousValue::Number(num)) => Ok((num as f32).into()),
				(VariantType::Float64, AmbiguousValue::Number(num)) => Ok(num.into()),
				(VariantType::Int32, AmbiguousValue::Number(num)) => Ok((num as i32).into()),
				(VariantType::Int64, AmbiguousValue::Number(num)) => Ok((num as i64).into()),

				(VariantType::String, AmbiguousValue::String(str)) => Ok(str.into()),
				(VariantType::Tags, AmbiguousValue::StringArray(tags)) => Ok(Tags::from(tags).into()),
				(VariantType::Content, AmbiguousValue::String(content)) => Ok(Content::from(content).into()),

				(VariantType::Vector2, AmbiguousValue::Array2(vector)) => {
					Ok(Vector2::new(vector[0] as f32, vector[1] as f32).into())
				}

				(VariantType::Vector3, AmbiguousValue::Array3(vector)) => {
					Ok(Vector3::new(vector[0] as f32, vector[1] as f32, vector[2] as f32).into())
				}

				(VariantType::Color3, AmbiguousValue::Array3(color)) => {
					Ok(Color3::new(color[0] as f32, color[1] as f32, color[2] as f32).into())
				}

				(VariantType::CFrame, AmbiguousValue::Array12(cf)) => {
					let cf = cf.map(|v| v as f32);

					let pos = Vector3::new(cf[0], cf[1], cf[2]);
					let orientation = Matrix3::new(
						Vector3::new(cf[3], cf[4], cf[5]),
						Vector3::new(cf[6], cf[7], cf[8]),
						Vector3::new(cf[9], cf[10], cf[11]),
					);

					Ok(CFrame::new(pos, orientation).into())
				}

				(VariantType::Attributes, AmbiguousValue::Attributes(attr)) => Ok(attr.into()),
				(VariantType::Attributes, AmbiguousValue::Object(value)) => {
					let mut attributes = Attributes::new();

					for (key, unresolved) in value {
						attributes.insert(key, unresolved.resolve_unambiguous()?);
					}

					Ok(attributes.into())
				}

				(VariantType::Font, AmbiguousValue::Font(font)) => Ok(font.into()),

				(VariantType::MaterialColors, AmbiguousValue::MaterialColors(colors)) => Ok(colors.into()),

				(_, unresolved) => Err(format_err!(
					"Wrong type of value for property {}.{}. Expected {:?}, got {}",
					class,
					property,
					variant,
					unresolved.describe(),
				)),
			},
			_ => Err(format_err!("Unknown data type for property {}.{}", class, property)),
		}
	}

	pub fn resolve_unambiguous(self) -> anyhow::Result<Variant> {
		match self {
			AmbiguousValue::Bool(value) => Ok(value.into()),
			AmbiguousValue::Number(value) => Ok(value.into()),
			AmbiguousValue::String(value) => Ok(value.into()),
			other => bail!("Cannot unambiguously resolve the value {other:?}"),
		}
	}

	fn describe(&self) -> &'static str {
		match self {
			AmbiguousValue::Bool(_) => "a bool",
			AmbiguousValue::String(_) => "a string",
			AmbiguousValue::StringArray(_) => "an array of strings",
			AmbiguousValue::Number(_) => "a number",
			AmbiguousValue::Object(_) => "a generic object",
			AmbiguousValue::Array2(_) => "an array of two numbers",
			AmbiguousValue::Array3(_) => "an array of three numbers",
			AmbiguousValue::Array4(_) => "an array of four numbers",
			AmbiguousValue::Array12(_) => "an array of twelve numbers",
			AmbiguousValue::Attributes(_) => "an object containing attributes",
			AmbiguousValue::Font(_) => "an object describing a Font",
			AmbiguousValue::MaterialColors(_) => "an object describing MaterialColors",
		}
	}
}

fn find_descriptor(class: &str, property: &str) -> Option<&'static PropertyDescriptor<'static>> {
	let database = rbx_reflection_database::get();
	let mut current_class = class;

	loop {
		let class = database.classes.get(current_class)?;

		if let Some(descriptor) = class.properties.get(property) {
			return Some(descriptor);
		}

		current_class = class.superclass.as_deref()?;
	}
}

fn list_examples(values: &[&str]) -> String {
	let mut output = String::new();
	let length = (values.len() - 1).min(5);

	for value in &values[..length] {
		output.push_str(value);
		output.push_str(", ");
	}

	if values.len() > 5 {
		write!(output, "or {} more", values.len() - length).unwrap();
	} else {
		output.push_str("or ");
		output.push_str(values[values.len() - 1]);
	}

	output
}

// Temporary solution to avoid saving `null` values in JSON files
fn truncate_float(float: f64) -> f64 {
	if float.is_infinite() {
		999_999_999.0 * float.signum()
	} else {
		float
	}
}
