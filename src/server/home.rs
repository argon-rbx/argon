use actix_web::{get, HttpResponse, Responder};
use log::trace;

#[get("/")]
async fn main() -> impl Responder {
	trace!("Received request: home");
	HttpResponse::Ok().body("Home Page")
}
