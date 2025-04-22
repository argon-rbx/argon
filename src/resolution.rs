// Based on Rojo's resolution.rs (https://github.com/rojo-rbx/rojo/blob/master/src/resolution.rs)

use anyhow::{bail, format_err, Context};
use rbx_dom_weak::types::{
	Attributes, Axes, BinaryString, BrickColor, CFrame, Color3, Color3uint8, ColorSequence, ColorSequenceKeypoint,
	Content, ContentId, ContentType, CustomPhysicalProperties, Enum, Faces, Font, MaterialColors, Matrix3, NumberRange,
	NumberSequence, NumberSequenceKeypoint, PhysicalProperties, Ray, Rect, Region3, Region3int16, Tags, UDim, UDim2,
	Variant, VariantType, Vector2, Vector2int16, Vector3, Vector3int16,
};
use rbx_reflection::{DataType, PropertyDescriptor};
use serde::{ser::SerializeSeq, Deserialize, Serialize, Serializer};
use std::{borrow::Borrow, collections::HashMap, fmt::Write};

use crate::ext::PropertyDescriptorExt;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum UnresolvedValue {
	FullyQualified(Variant),
	Ambiguous(AmbiguousValue),
}

impl UnresolvedValue {
	pub fn resolve(self, class: &str, property: &str) -> anyhow::Result<Variant> {
		match self {
			UnresolvedValue::FullyQualified(full) => Ok(full),
			UnresolvedValue::Ambiguous(partial) => partial.resolve(class, property),
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
	pub fn from_variant(variant: Variant, class: &str, property: &str) -> Self {
		Self::Ambiguous(match variant {
			Variant::Attributes(attr) => {
				let mut object = HashMap::new();

				for (key, value) in attr {
					object.insert(
						key,
						match value {
							Variant::Bool(bool) => UnresolvedValue::Ambiguous(AmbiguousValue::Bool(bool)),
							Variant::Float64(num) => UnresolvedValue::Ambiguous(AmbiguousValue::Number(num)),
							Variant::String(str) => UnresolvedValue::Ambiguous(AmbiguousValue::String(str)),
							_ => UnresolvedValue::FullyQualified(value),
						},
					);
				}

				AmbiguousValue::Object(object)
			}

			Variant::Axes(axes) => {
				let mut array = Vec::new();

				if axes.contains(Axes::X) {
					array.push("X".into());
				}

				if axes.contains(Axes::Y) {
					array.push("Y".into());
				}

				if axes.contains(Axes::Z) {
					array.push("Z".into());
				}

				AmbiguousValue::StringArray(array)
			}

			Variant::BinaryString(binary) => {
				AmbiguousValue::String(String::from_utf8(binary.into_vec()).unwrap_or_default())
			}

			Variant::Bool(bool) => AmbiguousValue::Bool(bool),

			Variant::BrickColor(color) => AmbiguousValue::String(color.to_string()),

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

			Variant::Color3(color) => AmbiguousValue::Array3([color.r as f64, color.g as f64, color.b as f64]),
			Variant::Color3uint8(color) => AmbiguousValue::Array3([color.r as f64, color.g as f64, color.b as f64]),

			Variant::ColorSequence(sequence) => AmbiguousValue::ColorSequence(sequence.keypoints),

			Variant::Content(content) => AmbiguousValue::String(match content.value() {
				ContentType::Object(referent) => referent.to_string(),
				ContentType::Uri(uri) => uri.to_owned(),
				_ => String::new(),
			}),
			Variant::ContentId(content) => AmbiguousValue::String(content.into_string()),

			Variant::Enum(rbx_enum) => {
				if let Some(property) = find_descriptor(class, property) {
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

			Variant::Faces(faces) => {
				let mut array = Vec::new();

				if faces.contains(Faces::RIGHT) {
					array.push("Right".into());
				}

				if faces.contains(Faces::TOP) {
					array.push("Top".into());
				}

				if faces.contains(Faces::BACK) {
					array.push("Back".into());
				}

				if faces.contains(Faces::LEFT) {
					array.push("Left".into());
				}

				if faces.contains(Faces::BOTTOM) {
					array.push("Bottom".into());
				}

				if faces.contains(Faces::FRONT) {
					array.push("Front".into());
				}

				AmbiguousValue::StringArray(array)
			}

			Variant::Float32(num) => AmbiguousValue::Number(num as f64),
			Variant::Float64(num) => AmbiguousValue::Number(num),

			Variant::Font(font) => AmbiguousValue::Font(font),

			Variant::Int32(num) => AmbiguousValue::Number(num as f64),
			Variant::Int64(num) => AmbiguousValue::Number(num as f64),

			Variant::MaterialColors(colors) => AmbiguousValue::MaterialColors(colors),

			Variant::NumberRange(range) => AmbiguousValue::Array2([range.min as f64, range.max as f64]),

			Variant::NumberSequence(sequence) => AmbiguousValue::NumberSequence(sequence.keypoints),

			Variant::OptionalCFrame(cf) => {
				if let Some(cf) = cf {
					AmbiguousValue::Array12([
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
					])
				} else {
					AmbiguousValue::String("null".into())
				}
			}

			Variant::PhysicalProperties(PhysicalProperties::Custom(custom)) => {
				AmbiguousValue::PhysicalProperties(custom)
			}
			Variant::PhysicalProperties(PhysicalProperties::Default) => AmbiguousValue::String("Default".into()),

			Variant::Ray(ray) => AmbiguousValue::Array3Array2([
				[ray.origin.x as f64, ray.origin.y as f64, ray.origin.z as f64],
				[ray.direction.x as f64, ray.direction.y as f64, ray.direction.z as f64],
			]),

			Variant::Rect(rect) => AmbiguousValue::Array4([
				rect.min.x as f64,
				rect.min.y as f64,
				rect.max.x as f64,
				rect.max.y as f64,
			]),
			// TODO: Implement Ref
			// Variant::Ref(reference) => AmbiguousValue::
			//
			Variant::Region3(region) => AmbiguousValue::Array3Array2([
				[region.min.x as f64, region.min.y as f64, region.min.z as f64],
				[region.max.x as f64, region.max.y as f64, region.max.z as f64],
			]),
			Variant::Region3int16(region) => AmbiguousValue::Array3Array2([
				[region.min.x as f64, region.min.y as f64, region.min.z as f64],
				[region.max.x as f64, region.max.y as f64, region.max.z as f64],
			]),

			Variant::SharedString(shared) => {
				AmbiguousValue::String(String::from_utf8(shared.data().to_vec()).unwrap_or_default())
			}
			Variant::String(str) => AmbiguousValue::String(str),

			Variant::Tags(tags) => AmbiguousValue::StringArray(tags.iter().map(|s| s.into()).collect()),

			Variant::UDim(udim) => AmbiguousValue::Array2([udim.scale as f64, udim.offset as f64]),

			Variant::UDim2(udim) => AmbiguousValue::Array2Array2([
				[udim.x.scale as f64, udim.x.offset as f64],
				[udim.y.scale as f64, udim.y.offset as f64],
			]),

			Variant::Vector2(vector) => AmbiguousValue::Array2([vector.x as f64, vector.y as f64]),
			Variant::Vector2int16(vector) => AmbiguousValue::Array2([vector.x as f64, vector.y as f64]),

			Variant::Vector3(vector) => AmbiguousValue::Array3([vector.x as f64, vector.y as f64, vector.z as f64]),
			Variant::Vector3int16(vector) => {
				AmbiguousValue::Array3([vector.x as f64, vector.y as f64, vector.z as f64])
			}

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
	#[serde(serialize_with = "serialize_number")]
	Number(f64),
	#[serde(serialize_with = "serialize_array")]
	Array2([f64; 2]),
	#[serde(serialize_with = "serialize_array")]
	Array3([f64; 3]),
	#[serde(serialize_with = "serialize_array")]
	Array4([f64; 4]),
	#[serde(serialize_with = "serialize_array")]
	Array12([f64; 12]),
	#[serde(serialize_with = "serialize_nested_array")]
	Array2Array2([[f64; 2]; 2]),
	#[serde(serialize_with = "serialize_nested_array")]
	Array3Array2([[f64; 3]; 2]),
	Attributes(Attributes),
	MaterialColors(MaterialColors),
	ColorSequence(Vec<ColorSequenceKeypoint>),
	NumberSequence(Vec<NumberSequenceKeypoint>),
	Font(Font),
	PhysicalProperties(CustomPhysicalProperties),
	Object(HashMap<String, UnresolvedValue>),
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

					format_err!(
						"Invalid value for property {}.{}. Got {} but expected a member of the {} enum such as {}",
						class,
						property,
						value,
						enum_name,
						list_examples(&examples),
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
				(VariantType::Attributes, AmbiguousValue::Attributes(attr)) => Ok(attr.into()),
				(VariantType::Attributes, AmbiguousValue::Object(value)) => {
					let mut attributes = Attributes::new();

					for (key, unresolved) in value {
						attributes.insert(key, unresolved.resolve_unambiguous()?);
					}

					Ok(attributes.into())
				}

				(VariantType::Axes, AmbiguousValue::StringArray(axes)) => {
					let mut bits = 0;

					for axis in axes {
						match axis.as_ref() {
							"X" => bits |= 1,
							"Y" => bits |= 2,
							"Z" => bits |= 4,
							_ => {
								bail!("invalid axis '{}'", axis);
							}
						}
					}

					Ok(Axes::from_bits(bits).unwrap_or_else(Axes::empty).into())
				}

				(VariantType::BinaryString, AmbiguousValue::String(str)) => {
					Ok(BinaryString::from(str.as_bytes()).into())
				}

				(VariantType::Bool, AmbiguousValue::Bool(bool)) => Ok(bool.into()),

				(VariantType::BrickColor, AmbiguousValue::Number(num)) => Ok(BrickColor::from_number(num as u16)
					.context(format!("{} is not valid BrickColor number", num))?
					.into()),
				(VariantType::BrickColor, AmbiguousValue::String(name)) => Ok(BrickColor::from_name(&name)
					.context(format!("{} is not valid BrickColor name", name))?
					.into()),

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

				(VariantType::Color3, AmbiguousValue::Array3(color)) => {
					let (r, g, b) = (color[0] as f32, color[1] as f32, color[2] as f32);

					// Fix for the custom BasePart.Color serialization (https://github.com/argon-rbx/rbx-dom/blob/master/patches/parts.yml#L18)
					if let Some(data_type) = descriptor.get_custom_serialization() {
						if data_type == "Color3uint8" && (r > 1.0 || g > 1.0 || b > 1.0) {
							return Ok(Color3::new(r / 255.0, g / 255.0, b / 255.0).into());
						}
					}

					Ok(Color3::new(r, g, b).into())
				}
				(VariantType::Color3uint8, AmbiguousValue::Array3(color)) => {
					Ok(Color3uint8::new(color[0] as u8, color[1] as u8, color[2] as u8).into())
				}

				(VariantType::ColorSequence, AmbiguousValue::ColorSequence(keypoints)) => {
					Ok(ColorSequence { keypoints }.into())
				}

				(VariantType::Content, AmbiguousValue::String(content)) => Ok(Content::from(content).into()),
				(VariantType::ContentId, AmbiguousValue::String(content)) => Ok(ContentId::from(content).into()),

				(VariantType::Faces, AmbiguousValue::StringArray(faces)) => {
					let mut bits = 0;

					for face in faces {
						match face.as_ref() {
							"Right" => bits |= 1,
							"Top" => bits |= 2,
							"Back" => bits |= 4,
							"Left" => bits |= 8,
							"Bottom" => bits |= 16,
							"Front" => bits |= 32,
							_ => {
								bail!("invalid face '{}'", face);
							}
						}
					}

					Ok(Faces::from_bits(bits).unwrap_or_else(Faces::empty).into())
				}

				(VariantType::Float32, AmbiguousValue::Number(num)) => Ok((num as f32).into()),
				(VariantType::Float64, AmbiguousValue::Number(num)) => Ok(num.into()),

				(VariantType::Font, AmbiguousValue::Font(font)) => Ok(font.into()),

				(VariantType::Int32, AmbiguousValue::Number(num)) => Ok((num as i32).into()),
				(VariantType::Int64, AmbiguousValue::Number(num)) => Ok((num as i64).into()),

				(VariantType::MaterialColors, AmbiguousValue::MaterialColors(colors)) => Ok(colors.into()),

				(VariantType::NumberRange, AmbiguousValue::Array2(range)) => {
					Ok(NumberRange::new(range[0] as f32, range[1] as f32).into())
				}

				(VariantType::NumberSequence, AmbiguousValue::NumberSequence(keypoints)) => {
					Ok(NumberSequence { keypoints }.into())
				}

				(VariantType::OptionalCFrame, AmbiguousValue::Array12(cf)) => {
					let cf = cf.map(|v| v as f32);

					let pos = Vector3::new(cf[0], cf[1], cf[2]);
					let orientation = Matrix3::new(
						Vector3::new(cf[3], cf[4], cf[5]),
						Vector3::new(cf[6], cf[7], cf[8]),
						Vector3::new(cf[9], cf[10], cf[11]),
					);

					Ok(CFrame::new(pos, orientation).into())
				}

				(VariantType::PhysicalProperties, AmbiguousValue::PhysicalProperties(custom)) => {
					Ok(PhysicalProperties::Custom(custom).into())
				}
				(VariantType::PhysicalProperties, AmbiguousValue::String(default)) => {
					if default != "Default" {
						bail!("string is not 'Default'");
					}

					Ok(PhysicalProperties::Default.into())
				}

				(VariantType::Ray, AmbiguousValue::Array3Array2(ray)) => Ok(Ray::new(
					Vector3::new(ray[0][0] as f32, ray[0][1] as f32, ray[0][2] as f32),
					Vector3::new(ray[1][0] as f32, ray[1][1] as f32, ray[1][2] as f32),
				)
				.into()),

				(VariantType::Rect, AmbiguousValue::Array4(rect)) => Ok(Rect::new(
					Vector2::new(rect[0] as f32, rect[1] as f32),
					Vector2::new(rect[2] as f32, rect[3] as f32),
				)
				.into()),
				// TODO: Implement Ref
				// (VariantType::Ref, AmbiguousValue::String(path)) => Ok(),
				//
				(VariantType::Region3, AmbiguousValue::Array3Array2(region)) => Ok(Region3::new(
					Vector3::new(region[0][0] as f32, region[0][1] as f32, region[0][2] as f32),
					Vector3::new(region[1][0] as f32, region[1][1] as f32, region[1][2] as f32),
				)
				.into()),
				(VariantType::Region3int16, AmbiguousValue::Array3Array2(region)) => Ok(Region3int16::new(
					Vector3int16::new(region[0][0] as i16, region[0][1] as i16, region[0][2] as i16),
					Vector3int16::new(region[1][0] as i16, region[1][1] as i16, region[1][2] as i16),
				)
				.into()),

				(VariantType::SharedString, AmbiguousValue::String(str)) => Ok(str.into()),
				(VariantType::String, AmbiguousValue::String(str)) => Ok(str.into()),

				(VariantType::Tags, AmbiguousValue::StringArray(tags)) => Ok(Tags::from(tags).into()),

				(VariantType::UDim, AmbiguousValue::Array2(udim)) => {
					Ok(rbx_dom_weak::types::UDim::new(udim[0] as f32, udim[1] as i32).into())
				}

				(VariantType::UDim2, AmbiguousValue::Array2Array2(udim)) => Ok(UDim2::new(
					UDim::new(udim[0][0] as f32, udim[0][1] as i32),
					UDim::new(udim[1][0] as f32, udim[1][1] as i32),
				)
				.into()),

				(VariantType::Vector2, AmbiguousValue::Array2(vector)) => {
					Ok(Vector2::new(vector[0] as f32, vector[1] as f32).into())
				}
				(VariantType::Vector2int16, AmbiguousValue::Array2(vector)) => {
					Ok(Vector2int16::new(vector[0] as i16, vector[1] as i16).into())
				}

				(VariantType::Vector3, AmbiguousValue::Array3(vector)) => {
					Ok(Vector3::new(vector[0] as f32, vector[1] as f32, vector[2] as f32).into())
				}
				(VariantType::Vector3int16, AmbiguousValue::Array3(vector)) => {
					Ok(Vector3int16::new(vector[0] as i16, vector[1] as i16, vector[2] as i16).into())
				}

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
			AmbiguousValue::Array2Array2(_) => "an array of two arrays of two numbers",
			AmbiguousValue::Array3Array2(_) => "an array of two arrays of three numbers",
			AmbiguousValue::Attributes(_) => "an object containing attributes",
			AmbiguousValue::MaterialColors(_) => "an object describing MaterialColors",
			AmbiguousValue::ColorSequence(_) => "an object describing a ColorSequence",
			AmbiguousValue::NumberSequence(_) => "an object describing a NumberSequence",
			AmbiguousValue::Font(_) => "an object describing a Font",
			AmbiguousValue::PhysicalProperties(_) => "an object describing PhysicalProperties",
			AmbiguousValue::Object(_) => "a generic object",
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

#[inline]
fn truncate_number(number: &f64) -> f64 {
	// Temporary solution to avoid saving `null` values in JSON files
	if number.is_infinite() {
		999_999_999.0 * number.signum()
	} else {
		(*number * 1_000_000.0).trunc() / 1_000_000.0
	}
}

fn serialize_number<S>(number: &f64, serializer: S) -> Result<S::Ok, S::Error>
where
	S: Serializer,
{
	let number = truncate_number(number);

	if number.fract() == 0.0 {
		serializer.serialize_i64(number as i64)
	} else {
		serializer.serialize_f64(number)
	}
}

fn serialize_array<S>(array: &[f64], serializer: S) -> Result<S::Ok, S::Error>
where
	S: Serializer,
{
	let mut seq = serializer.serialize_seq(Some(array.len()))?;

	for number in array {
		let number = truncate_number(number);

		if number.fract() == 0.0 {
			seq.serialize_element(&(number as i64))?;
		} else {
			seq.serialize_element(&number)?;
		}
	}

	seq.end()
}

fn serialize_nested_array<S, const N: usize>(array: &[[f64; N]; 2], serializer: S) -> Result<S::Ok, S::Error>
where
	S: Serializer,
{
	let mut seq = serializer.serialize_seq(Some(2))?;

	for array in array {
		let mut new: Vec<Number> = Vec::with_capacity(array.len());

		for number in array {
			let number = truncate_number(number);

			if number.fract() == 0.0 {
				new.push(Number::Int(number as i64));
			} else {
				new.push(Number::Float(number));
			}
		}

		seq.serialize_element(array.as_slice())?;
	}

	seq.end()
}

#[derive(Serialize)]
#[serde(untagged)]
enum Number {
	Int(i64),
	Float(f64),
}
