use actix_web::{get, web, Responder, Result};

#[derive(serde::Serialize)]
struct Test {
	name: String,
}

#[get("/sync")]
async fn main() -> Result<impl Responder> {
	let test = Test {
		name: String::from("test"),
	};

	Ok(web::Json(test))
}
