use actix_web::{web, App, HttpServer, Responder};
use std::io::Result;

use crate::core::Core;

mod home;
mod stop;
mod sync_server;

async fn default_redirect() -> impl Responder {
	web::Redirect::to("/")
}

pub struct Server {
	core: Core,
}

impl Server {
	pub fn new(core: Core) -> Self {
		Self { core }
	}

	#[actix_web::main]
	pub async fn start(&self) -> Result<()> {
		let host = self.core.host();
		let port = self.core.port();

		self.core.start();

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
}
