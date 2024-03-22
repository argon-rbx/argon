#[cfg(test)]
mod tests {
	use config_derive::{Get, Iter, Set, Val};
	use serde::{Deserialize, Serialize};

	#[test]
	fn it_works() {
		#[derive(Debug, Val, Iter, Get, Set)]
		struct Test {
			a: u16,
			b: i32,
			c: String,
			d: bool,
			e: String,
		}

		let mut test = Test {
			a: 1,
			b: 2,
			c: "hello".to_string(),
			d: true,
			e: "world".to_string(),
		};

		println!("{:?}", test.get("c"));

		for (key, value) in &test {
			println!("{}: {:?}", key, value);
		}

		test.set("c", "goodbye").unwrap();
		test.set("d", "false").unwrap();

		println!("{:?}", test);
	}
}
