use actix_msgpack::MsgPackConfig;
use actix_web::{
	web::{self, Data},
	App, HttpServer, Responder,
};
use derive_from_one::FromOne;
use serde::{Deserialize, Serialize};
use std::{io::Result, net::TcpListener, sync::Arc};

use crate::{
	constants::MAX_PAYLOAD_SIZE,
	core::{changes::Changes, Core},
	project::ProjectDetails,
};

mod details;
mod exec;
mod home;
mod open;
mod read;
mod snapshot;
mod stop;
mod subscribe;
mod unsubscribe;
mod write;

#[derive(Debug, Clone, Serialize, FromOne)]
pub enum Message {
	SyncChanges(SyncChanges),
	SyncbackChanges(SyncbackChanges),
	SyncDetails(SyncDetails),
	ExecuteCode(ExecuteCode),
	Disconnect(Disconnect),
}

impl Message {
	pub fn is_change(&self) -> bool {
		matches!(self, Message::SyncChanges(_) | Message::SyncbackChanges(_))
	}
}

#[derive(Debug, Clone, Serialize)]
pub struct SyncChanges(pub Changes);

#[derive(Debug, Clone, Serialize)]
pub struct SyncbackChanges();

#[derive(Debug, Clone, Serialize)]
pub struct SyncDetails(pub ProjectDetails);

#[derive(Debug, Clone, Serialize)]
pub struct ExecuteCode {
	pub code: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct Disconnect {
	pub message: String,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct AuthRequest {
	client_id: u32,
}

pub struct Server {
	core: Arc<Core>,
	host: String,
	port: u16,
}

impl Server {
	pub fn new(core: Arc<Core>, host: &str, port: u16) -> Self {
		Self {
			core,
			host: host.to_owned(),
			port,
		}
	}

	#[actix_web::main]
	pub async fn start(&self) -> Result<()> {
		let core = self.core.clone();

		HttpServer::new(move || {
			let mut msgpack_config = MsgPackConfig::default();
			msgpack_config.limit(MAX_PAYLOAD_SIZE);

			App::new()
				.app_data(Data::new(core.clone()))
				.app_data(msgpack_config)
				.service(details::main)
				.service(subscribe::main)
				.service(unsubscribe::main)
				.service(snapshot::main)
				.service(read::main)
				.service(write::main)
				.service(exec::main)
				.service(open::main)
				.service(stop::main)
				.service(home::main)
				.default_service(web::to(Self::default_redirect))
		})
		.backlog(0)
		.disable_signals()
		.bind((self.host.clone(), self.port))?
		.run()
		.await
	}

	async fn default_redirect() -> impl Responder {
		web::Redirect::to("/")
	}
}

pub fn is_port_free(host: &str, port: u16) -> bool {
	TcpListener::bind((host, port)).is_ok()
}

pub fn get_free_port(host: &str, port: u16) -> u16 {
	let mut port = port;

	while !is_port_free(host, port) {
		port += 1;

		// This should never happen, but just in case
		if port == 65535 {
			break;
		}
	}

	port
}

pub fn format_address(host: &str, port: u16) -> String {
	format!("http://{}:{}", host, port)
}
