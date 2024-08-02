mod unresolved_value {
	use argon::resolution::UnresolvedValue;

	use rbx_dom_weak::types::{
		Attributes, Axes, BinaryString, BrickColor, CFrame, Color3, Color3uint8, ColorSequence, ColorSequenceKeypoint,
		Content, CustomPhysicalProperties, Enum, Faces, Font, FontStyle, FontWeight, Matrix3, NumberRange,
		NumberSequence, NumberSequenceKeypoint, PhysicalProperties, Ray, Rect, Region3int16, Tags, UDim, UDim2,
		Variant, Vector2, Vector3, Vector3int16,
	};

	// Based on Rojo's resolution::test (https://github.com/rojo-rbx/rojo/blob/master/src/resolution.rs#L249)
	fn resolve(class: &str, property: &str, value: &str) -> Variant {
		let unresolved: UnresolvedValue = serde_json::from_str(value).unwrap();
		unresolved.resolve(class, property).unwrap()
	}

	fn resolve_unambiguous(value: &str) -> Variant {
		let unresolved: UnresolvedValue = serde_json::from_str(value).unwrap();
		unresolved.resolve_unambiguous().unwrap()
	}

	#[test]
	fn attributes() {
		let mut attributes = Attributes::new();
		attributes.insert("String".into(), Variant::String("Hello, world!".into()));
		attributes.insert("Number".into(), Variant::Float64(4.2));
		attributes.insert("Bool".into(), Variant::Bool(true));

		assert_eq!(
			resolve(
				"Instance",
				"Attributes",
				r#"{"String": "Hello, world!", "Number": 4.2, "Bool": true}"#
			),
			attributes.into()
		);
	}

	#[test]
	fn axes() {
		assert_eq!(resolve("ArcHandles", "Axes", "[]"), Axes::empty().into());
		assert_eq!(resolve("ArcHandles", "Axes", r#"["X"]"#), Axes::X.into());
		assert_eq!(resolve("ArcHandles", "Axes", r#"["X", "Y", "Z"]"#), Axes::all().into());
	}

	#[test]
	fn binary_string() {
		assert_eq!(
			resolve("BinaryStringValue", "Value", r#""Hello, world!""#),
			BinaryString::from("Hello, world!".as_bytes().to_vec()).into()
		);
	}

	#[test]
	fn bool() {
		assert_eq!(resolve("Part", "Anchored", "true"), true.into());
		assert_eq!(resolve_unambiguous("false"), false.into());
	}

	#[test]
	fn brick_color() {
		assert_eq!(
			resolve("Part", "BrickColor", "1032"),
			BrickColor::from_number(1032).unwrap().into()
		);
		assert_eq!(
			resolve("Part", "BrickColor", r#""Electric blue""#),
			BrickColor::from_name("Electric blue").unwrap().into()
		);
	}

	#[test]
	fn cframe() {
		assert_eq!(
			resolve("Part", "CFrame", "[1, 2, 3, 1, 0, 0, 0, 1, 0, 0, 0, 1]"),
			CFrame::new(Vector3::new(1.0, 2.0, 3.0), Matrix3::identity()).into()
		);
	}

	#[test]
	fn color3() {
		assert_eq!(
			resolve("Lighting", "Ambient", "[1.2, 3.4, 5.6]"),
			Color3::new(1.2, 3.4, 5.6).into()
		);
	}

	#[test]
	fn color3_uint8() {
		assert_eq!(
			resolve("Part", "Color3uint8", "[0, 100, 200]"),
			Color3uint8::new(0, 100, 200).into()
		);
	}

	#[test]
	fn color_sequence() {
		let keypoints = vec![
			ColorSequenceKeypoint::new(0.0, Color3::new(1.0, 0.0, 0.0)),
			ColorSequenceKeypoint::new(0.5, Color3::new(0.0, 1.0, 0.0)),
			ColorSequenceKeypoint::new(1.0, Color3::new(0.0, 0.0, 1.0)),
		];

		assert_eq!(
			resolve(
				"Beam",
				"Color",
				r#"[{"time": 0, "color": [1, 0, 0]}, {"time": 0.5, "color": [0, 1, 0]}, {"time": 1, "color": [0, 0, 1]}]"#
			),
			ColorSequence { keypoints }.into()
		);
	}

	#[test]
	fn content() {
		assert_eq!(
			resolve("Decal", "Texture", r#""rbxassetid://1234567890""#),
			Content::from("rbxassetid://1234567890").into(),
		);
	}

	#[test]
	fn enums() {
		assert_eq!(resolve("Part", "Shape", r#""Ball""#), Enum::from_u32(0).into());
		assert_eq!(resolve("Part", "Shape", r#""Block""#), Enum::from_u32(1).into());
		assert_eq!(resolve("Part", "Shape", r#""Cylinder""#), Enum::from_u32(2).into());
	}

	#[test]
	fn faces() {
		assert_eq!(resolve("Handles", "Faces", "[]"), Faces::empty().into());
		assert_eq!(resolve("Handles", "Faces", r#"["Right"]"#), Faces::RIGHT.into());
		assert_eq!(
			resolve(
				"Handles",
				"Faces",
				r#"["Right", "Top", "Back", "Left", "Bottom", "Front"]"#
			),
			Faces::all().into()
		);
	}

	#[test]
	fn float32() {
		assert_eq!(resolve("Players", "RespawnTime", "0.5"), Variant::Float32(0.5));
		assert_eq!(resolve("Players", "RespawnTime", "123.456"), Variant::Float32(123.456));
	}

	#[test]
	fn float64() {
		assert_eq!(resolve("Sound", "PlaybackLoudness", "0.5"), Variant::Float64(0.5));
		assert_eq!(
			resolve("Sound", "PlaybackLoudness", "123.456"),
			Variant::Float64(123.456)
		);
		assert_eq!(resolve_unambiguous("4.2"), Variant::Float64(4.2));
	}

	#[test]
	fn font() {
		let font = Font::new(
			"rbxasset://fonts/families/Ubuntu.json",
			FontWeight::Bold,
			FontStyle::Italic,
		);

		assert_eq!(
			resolve(
				"TextLabel",
				"FontFace",
				r#"{"family": "rbxasset://fonts/families/SourceSansPro.json", "weight": "Regular", "style": "Normal"}"#
			),
			Font::default().into()
		);
		assert_eq!(
			resolve(
				"TextLabel",
				"FontFace",
				r#"{"family": "rbxasset://fonts/families/Ubuntu.json", "weight": "Bold", "style": "Italic"}"#
			),
			font.into()
		);
	}

	#[test]
	fn int32() {
		assert_eq!(resolve("Frame", "ZIndex", "8"), Variant::Int32(8));
		assert_eq!(resolve("Frame", "ZIndex", "999999999"), Variant::Int32(999999999));
	}

	#[test]
	fn int64() {
		assert_eq!(resolve("Player", "UserId", "8"), Variant::Int64(8));
		assert_eq!(resolve("Player", "UserId", "999999999"), Variant::Int64(999999999));
	}

	#[test]
	fn number_range() {
		assert_eq!(
			resolve("ParticleEmitter", "Lifetime", "[5, 10]"),
			NumberRange::new(5.0, 10.0).into()
		);
	}

	#[test]
	fn number_sequence() {
		let keypoints = vec![
			NumberSequenceKeypoint::new(0.0, 0.0, 0.0),
			NumberSequenceKeypoint::new(0.5, 0.3, 0.0),
			NumberSequenceKeypoint::new(1.0, 0.8, 0.0),
		];

		assert_eq!(
			resolve(
				"Beam",
				"Transparency",
				r#"[{"time": 0, "value": 0, "envelope": 0}, {"time": 0.5, "value": 0.3, "envelope": 0}, {"time": 1, "value": 0.8, "envelope": 0}]"#
			),
			NumberSequence { keypoints }.into()
		);
	}

	#[test]
	fn optional_cframe() {
		assert_eq!(
			resolve("Model", "WorldPivotData", "[0, 0, 0, 1, 0, 0, 0, 1, 0, 0, 0, 1]"),
			CFrame::new(Vector3::new(0.0, 0.0, 0.0), Matrix3::identity()).into()
		);
	}

	#[test]
	fn physical_properties() {
		let properties = PhysicalProperties::Custom(CustomPhysicalProperties {
			density: 1.0,
			friction: 2.0,
			elasticity: 3.0,
			friction_weight: 4.0,
			elasticity_weight: 5.0,
		});

		assert_eq!(
			resolve(
				"Part",
				"CustomPhysicalProperties",
				r#"{"density": 1, "friction": 2, "elasticity": 3, "frictionWeight": 4, "elasticityWeight": 5}"#
			),
			properties.into()
		);
		assert_eq!(
			resolve("Part", "CustomPhysicalProperties", r#""Default""#),
			PhysicalProperties::Default.into()
		);
	}

	#[test]
	fn ray() {
		assert_eq!(
			resolve("RayValue", "Value", r#"{"origin": [1, 2, 3], "direction": [4, 5, 6]}"#),
			Ray::new(Vector3::new(1.0, 2.0, 3.0), Vector3::new(4.0, 5.0, 6.0)).into()
		);
	}

	#[test]
	fn rect() {
		assert_eq!(
			resolve("ImageButton", "SliceCenter", "[[1, 2], [3, 4]]"),
			Rect::new(Vector2::new(1.0, 2.0), Vector2::new(3.0, 4.0)).into()
		);
	}

	#[test]
	fn refs() {
		// TODO: Implement Ref
		// assert_eq!(resolve("Model", "PrimaryPart", ""), Ref::none().into());
	}

	#[test]
	fn region3() {
		// Currently Region3 property does not exist

		// assert_eq!(
		// 	resolve("Not", "Available", "[[1, 2, 3], [4, 5, 6]]"),
		// 	Region3::new(Vector3::new(1.0, 2.0, 3.0), Vector3::new(4.0, 5.0, 6.0)).into()
		// );
	}

	#[test]
	fn region3_int16() {
		assert_eq!(
			resolve("Terrain", "MaxExtents", "[[1, 2, 3], [4, 5, 6]]"),
			Region3int16::new(Vector3int16::new(1, 2, 3), Vector3int16::new(4, 5, 6)).into()
		);
	}

	#[test]
	fn shared_string() {
		// Currently there is not valid SharedString property to test

		// assert_eq!(
		// 	resolve("Not", "Available", r#""Hello, world!""#),
		// 	SharedString::new("Hello, world!".as_bytes().to_vec()).into()
		// );
	}

	#[test]
	fn string() {
		assert_eq!(resolve("Instance", "Name", r#""Argon""#), "Argon".into());
		assert_eq!(resolve_unambiguous(r#""Cool!""#), "Cool!".into());
	}

	#[test]
	fn tags() {
		let mut tags = Tags::new();
		tags.push("foo");
		tags.push("bar");

		assert_eq!(resolve("Instance", "Tags", r#"["foo", "bar"]"#), tags.into());
	}

	#[test]
	fn udim() {
		assert_eq!(
			resolve("UIListLayout", "Padding", "[0.5, 500]"),
			UDim::new(0.5, 500).into()
		);
	}

	#[test]
	fn udim2() {
		assert_eq!(
			resolve("Frame", "Size", "[[0.5, 500], [1, 1000]]"),
			UDim2::new(UDim::new(0.5, 500), UDim::new(1.0, 1000)).into()
		);
	}

	#[test]
	fn vector2() {
		assert_eq!(
			resolve("ImageLabel", "ImageRectSize", "[1.2, 3.4]"),
			Vector2::new(1.2, 3.4).into()
		);
	}

	#[test]
	fn vector2_int16() {
		// Currently Vector2int16 property does not exist

		// assert_eq!(
		// 	resolve("Not", "Available", "[1, 2]"),
		// 	Vector2int16::new(1, 2).into()
		// );
	}

	#[test]
	fn vector3() {
		assert_eq!(
			resolve("Part", "Size", "[1.2, 3.4, 5.6]"),
			Vector3::new(1.2, 3.4, 5.6).into()
		);
	}

	#[test]
	fn vector3_int16() {
		assert_eq!(
			resolve("TerrainRegion", "ExtentsMax", "[1, 2, 3]"),
			Vector3int16::new(1, 2, 3).into()
		);
	}
}
