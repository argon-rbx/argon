#[cfg(test)]
mod tests {

	#[test]
	fn it_works() {
		use config_derive::Get;

		#[derive(Debug, Get)]
		struct Test {
			a: i32,
			b: i32,
		}

		let test = Test { a: 1, b: 2 };

		println!("{:?}", test.get("c"));
	}
}
