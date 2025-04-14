use anyhow::Result;
use clap::Parser;
use eventsource_client as sse;
use eventsource_client::Client;
use log::{error, info};
use tokio::runtime::Runtime;

/// Connect to an MCP SSE endpoint (used internally by Cursor)
#[derive(Parser)]
pub struct ConnectMcp {
	/// The URL of the SSE endpoint
	#[arg()]
	url: String,
}

impl ConnectMcp {
	pub fn main(self) -> Result<()> {
		let rt = Runtime::new()?;
		rt.block_on(async { self.run_connection().await })
	}

	async fn run_connection(&self) -> Result<()> {
		info!("Connecting to MCP SSE stream at: {}", self.url);

		let client = sse::ClientBuilder::for_url(&self.url)?
			.header("Accept", "text/event-stream")?
			.build();

		use futures::StreamExt;
		let mut stream = client.stream();

		loop {
			match stream.next().await {
				Some(Ok(sse_item)) => {
					self.handle_sse_item(sse_item).await;
				}
				Some(Err(err)) => {
					error!("SSE stream error: {}", err);
					tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
				}
				None => {
					info!("SSE stream closed by server.");
					break;
				}
			}
		}

		Ok(())
	}

	async fn handle_sse_item(&self, item: sse::SSE) {
		match item {
			sse::SSE::Event(event) => {
				info!(
					"Received MCP message: type='{}', data='{}'",
					event.event_type, event.data
				);
				// TODO: Add logic to process the message data (event.data) based on its type
			}
			sse::SSE::Comment(comment) => {
				log::debug!("Received SSE comment: {}", comment);
			}
		}
	}
}
