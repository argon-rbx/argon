use actix_web::{get, Responder, HttpResponse};

#[get("/")]
async fn main() -> impl Responder {
	HttpResponse::Ok().body("Home Page")
}
