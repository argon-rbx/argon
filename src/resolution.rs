// Based on Rojo's resolution.rs (https://github.com/rojo-rbx/rojo/blob/master/src/resolution.rs)

use anyhow::{bail, format_err};
use rbx_dom_weak::types::{
	Attributes, CFrame, Color3, Content, Enum, Font, MaterialColors, Matrix3, Tags, Variant, VariantType, Vector2,
	Vector3,
};
use rbx_reflection::{DataType, PropertyDescriptor};
use serde::{Deserialize, Serialize};
use std::{borrow::Borrow, fmt::Write};

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
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum AmbiguousValue {
	Bool(bool),
	String(String),
	StringArray(Vec<String>),
	Number(f64),
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
			get_descriptor(class, property).ok_or_else(|| format_err!("Unknown property {}.{}", class, property))?;

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
				(VariantType::Bool, AmbiguousValue::Bool(value)) => Ok(value.into()),

				(VariantType::Float32, AmbiguousValue::Number(value)) => Ok((value as f32).into()),
				(VariantType::Float64, AmbiguousValue::Number(value)) => Ok(value.into()),
				(VariantType::Int32, AmbiguousValue::Number(value)) => Ok((value as i32).into()),
				(VariantType::Int64, AmbiguousValue::Number(value)) => Ok((value as i64).into()),

				(VariantType::String, AmbiguousValue::String(value)) => Ok(value.into()),
				(VariantType::Tags, AmbiguousValue::StringArray(value)) => Ok(Tags::from(value).into()),
				(VariantType::Content, AmbiguousValue::String(value)) => Ok(Content::from(value).into()),

				(VariantType::Vector2, AmbiguousValue::Array2(value)) => {
					Ok(Vector2::new(value[0] as f32, value[1] as f32).into())
				}

				(VariantType::Vector3, AmbiguousValue::Array3(value)) => {
					Ok(Vector3::new(value[0] as f32, value[1] as f32, value[2] as f32).into())
				}

				(VariantType::Color3, AmbiguousValue::Array3(value)) => {
					Ok(Color3::new(value[0] as f32, value[1] as f32, value[2] as f32).into())
				}

				(VariantType::CFrame, AmbiguousValue::Array12(value)) => {
					let value = value.map(|v| v as f32);
					let pos = Vector3::new(value[0], value[1], value[2]);
					let orientation = Matrix3::new(
						Vector3::new(value[3], value[4], value[5]),
						Vector3::new(value[6], value[7], value[8]),
						Vector3::new(value[9], value[10], value[11]),
					);

					Ok(CFrame::new(pos, orientation).into())
				}

				(VariantType::Attributes, AmbiguousValue::Attributes(value)) => Ok(value.into()),

				(VariantType::Font, AmbiguousValue::Font(value)) => Ok(value.into()),

				(VariantType::MaterialColors, AmbiguousValue::MaterialColors(value)) => Ok(value.into()),

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

fn get_descriptor(class: &str, property: &str) -> Option<&'static PropertyDescriptor<'static>> {
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
