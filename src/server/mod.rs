use actix_web::{web, App, HttpServer, Responder};
use std::io::Result;

mod home;
mod stop;
mod sync_server;

async fn default_redirect() -> impl Responder {
	web::Redirect::to("/")
}

#[tokio::main]
pub async fn start(host: String, port: u16) -> Result<()> {
	HttpServer::new(|| {
		App::new()
			.service(home::main)
			.service(stop::main)
			.service(sync_server::main)
			.default_service(web::to(default_redirect))
	})
	.bind((host, port))?
	.run()
	.await
}
