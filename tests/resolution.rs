mod unresolved_value {
	use argon::resolution::UnresolvedValue;

	use rbx_dom_weak::types::{
		Attributes, Axes, BinaryString, BrickColor, CFrame, Color3, Color3uint8, Matrix3, Variant, Vector3,
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
		attributes.insert(String::from("String"), Variant::String(String::from("Hello, world!")));
		attributes.insert(String::from("Number"), Variant::Float64(4.2));
		attributes.insert(String::from("Bool"), Variant::Bool(true));

		assert_eq!(
			resolve(
				"Instance",
				"Attributes",
				r#"{"String": "Hello, world!", "Number": 4.2, "Bool": true}"#
			),
			Variant::Attributes(attributes)
		);
	}

	#[test]
	fn axes() {
		assert_eq!(resolve("ArcHandles", "Axes", r#"[]"#), Variant::Axes(Axes::empty()));
		assert_eq!(
			resolve("ArcHandles", "Axes", r#"["X", "Y", "Z"]"#),
			Variant::Axes(Axes::all())
		);
	}

	#[test]
	fn binary_string() {
		assert_eq!(
			resolve("BinaryStringValue", "Value", r#""Hello, world!""#),
			Variant::BinaryString(BinaryString::from(vec![
				72, 101, 108, 108, 111, 44, 32, 119, 111, 114, 108, 100, 33
			]))
		);
	}

	#[test]
	fn bool() {
		assert_eq!(resolve("Part", "Anchored", "true"), Variant::Bool(true));
		assert_eq!(resolve_unambiguous("false"), Variant::Bool(false));
	}

	#[test]
	fn brick_color() {
		assert_eq!(
			resolve("Part", "BrickColor", "1032"),
			Variant::BrickColor(BrickColor::from_number(1032).unwrap())
		);
		assert_eq!(
			resolve("Part", "BrickColor", r#""Electric blue""#),
			Variant::BrickColor(BrickColor::from_name("Electric blue").unwrap())
		);
	}

	#[test]
	fn cframe() {
		assert_eq!(
			resolve("Part", "CFrame", r#"[1, 2, 3, 1, 0, 0, 0, 1, 0, 0, 0, 1]"#),
			Variant::CFrame(CFrame::new(Vector3::new(1.0, 2.0, 3.0), Matrix3::identity()))
		);
	}

	#[test]
	fn color3() {
		assert_eq!(
			resolve("Lighting", "Ambient", r#"[0.5, 0.5, 0.5]"#),
			Variant::Color3(Color3::new(0.5, 0.5, 0.5))
		);
	}

	#[test]
	fn color3_uint8() {
		assert_eq!(
			resolve("Part", "Color3uint8", r#"[0, 100, 200]"#),
			Variant::Color3uint8(Color3uint8::new(0, 100, 200))
		);
	}
}
