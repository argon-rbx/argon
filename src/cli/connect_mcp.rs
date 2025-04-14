use anyhow::Result;
use clap::Parser;
use eventsource_client as sse;
use eventsource_client::Client;
use log::{debug, error, info};
use serde_json::Value;
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
				// Process different event types
				match event.event_type.as_str() {
					"ping" => {
						debug!("Received MCP ping");
					}
					"message" => {
						// Try to parse the message as JSON
						if let Ok(json) = serde_json::from_str::<Value>(&event.data) {
							if let Some(message_type) = json.get("type").and_then(|t| t.as_str()) {
								match message_type {
									"completion" => {
										if let Some(completion) = json.get("completion").and_then(|c| c.as_str()) {
											info!("Received completion: {}", completion);
										}
									}
									"error" => {
										if let Some(error_msg) = json.get("error").and_then(|e| e.as_str()) {
											error!("Received MCP error: {}", error_msg);
										}
									}
									"status" => {
										if let Some(status) = json.get("status").and_then(|s| s.as_str()) {
											info!("MCP status change: {}", status);
										}
									}
									_ => {
										debug!("Unhandled message type: {}", message_type);
									}
								}
							} else {
								debug!("Received JSON message without type: {}", event.data);
							}
						} else {
							debug!("Received non-JSON message: {}", event.data);
						}
					}
					_ => {
						info!(
							"Received unhandled event type: {} with data: {}",
							event.event_type, event.data
						);
					}
				}
			}
			sse::SSE::Comment(comment) => {
				debug!("Received SSE comment: {}", comment);
			}
		}
	}
}
