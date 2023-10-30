use actix_web::{get, Responder, HttpResponse};

#[get("/sync")]
async fn main() -> impl Responder {
	HttpResponse::Ok().body("test")
}
