#[cfg(test)]
mod tests {
	use std::{thread::sleep, time::Duration};

	#[test]
	fn it_works() {
		let _server = puffin_http::Server::new("localhost:8888").unwrap();
		puffin::set_scopes_on(true);

		#[profiling::function]
		fn test() {
			let mut vec = vec![];

			for i in 0..100 {
				profiling::scope!("scope1");
				vec.push(i);

				for i in 0..200 {
					profiling::scope!("scope2", "with data");
					vec.push(i);
				}
			}
		}

		#[profiling::function("with data")]
		fn test_data() {
			let mut vec = vec![];

			for i in 0..100 {
				profiling::scope!("scope3", i.to_string());
				vec.push(i);
			}
		}

		loop {
			profiling::start_frame!();

			test();
			test_data();

			sleep(Duration::from_millis(100));
		}
	}
}
