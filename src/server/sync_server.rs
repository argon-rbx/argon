use actix_web::{get, HttpResponse, Responder};

#[get("/sync")]
async fn main() -> impl Responder {
	HttpResponse::Ok().body("test")
}
