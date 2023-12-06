use actix_web::{post, HttpResponse, Responder};
use log::trace;
use std::process;

async fn stop() {
	trace!("Argon stopped!");
	process::exit(1);
}

#[post("stop")]
async fn main() -> impl Responder {
	tokio::spawn(stop());
	HttpResponse::Ok().finish()
}
