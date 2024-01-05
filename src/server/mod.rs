use actix_web::{
	web::{self, Data},
	App, HttpServer, Responder,
};
use std::{io::Result, sync::Arc};

use crate::core::Core;

mod exec;
mod home;
mod read;
mod read_all;
mod stop;
mod subscribe;
mod unsubscribe;

async fn default_redirect() -> impl Responder {
	web::Redirect::to("/")
}

pub struct Server {
	core: Arc<Core>,
	host: String,
	port: u16,
}

impl Server {
	pub fn new(core: Core, host: &String, port: &u16) -> Self {
		Self {
			core: Arc::new(core),
			host: host.to_owned(),
			port: port.to_owned(),
		}
	}

	#[actix_web::main]
	pub async fn start(&self) -> Result<()> {
		let core = self.core.clone();
		core.watch(None);

		HttpServer::new(move || {
			App::new()
				.app_data(Data::new(core.clone()))
				.service(subscribe::main)
				.service(unsubscribe::main)
				.service(home::main)
				.service(stop::main)
				.service(read::main)
				.service(read_all::main)
				.service(exec::main)
				.default_service(web::to(default_redirect))
		})
		.bind((self.host.clone(), self.port))?
		.run()
		.await
	}
}
