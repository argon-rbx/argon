use actix_web::{get, HttpResponse, Responder};

#[get("/")]
async fn main() -> impl Responder {
	HttpResponse::Ok().body("Home Page")
}
