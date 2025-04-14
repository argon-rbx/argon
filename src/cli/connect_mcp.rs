use anyhow::{bail, Result};
use clap::Parser;
use eventsource_client as sse;
use eventsource_client::Client;
use futures::StreamExt;
use log::{debug, error, info, warn};
use serde_json::Value;
use tokio::runtime::Runtime;

/// Connect to an MCP SSE endpoint (used internally by Cursor)
#[derive(Parser)]
pub struct ConnectMcp {
	/// The URL of the SSE endpoint
	#[arg()]
	url: String,

	/// Whether to run in client mode
	#[arg(long)]
	client: bool,
}

impl ConnectMcp {
	pub fn main(self) -> Result<()> {
		let rt = Runtime::new()?;
		rt.block_on(async { self.run_connection().await })
	}

	async fn run_connection(&self) -> Result<()> {
		info!("Connecting to MCP SSE stream at: {}", self.url);

		if self.client {
			info!("Running in client mode");
		} else {
			warn!("Server mode not supported for SSE transport, running in client mode");
		}

		// Build the client with retry
		let client = sse::ClientBuilder::for_url(&self.url)?
			.header("Accept", "text/event-stream")?
			.build();

		// Subscribe to the SSE stream
		let mut stream = client.stream();

		// Send initial subscribe message to register with the server
		self.send_subscribe_request(&self.url).await?;

		info!("Successfully connected to MCP server");

		// Process stream events
		let mut connection_attempts = 0;
		let max_connection_attempts = 10;

		loop {
			match stream.next().await {
				Some(Ok(sse_item)) => {
					// Reset connection attempts on successful message
					connection_attempts = 0;
					self.handle_sse_item(sse_item).await;
				}
				Some(Err(err)) => {
					error!("SSE stream error: {}", err);
					connection_attempts += 1;

					if connection_attempts >= max_connection_attempts {
						error!("Maximum connection attempts reached. Giving up.");
						bail!("Failed to maintain connection to MCP server: {}", err);
					}

					let delay = 2u64.pow(connection_attempts.min(6)) * 1000;
					warn!("Reconnecting in {} ms...", delay);
					tokio::time::sleep(tokio::time::Duration::from_millis(delay)).await;
				}
				None => {
					info!("SSE stream closed by server.");
					break;
				}
			}
		}

		Ok(())
	}

	async fn send_subscribe_request(&self, url: &str) -> Result<()> {
		// Log the subscription attempt
		debug!("Attempting to subscribe to SSE stream at: {}", url);

		// In a future implementation we could use reqwest to send an actual POST request
		// to notify the server about our subscription if the protocol requires it
		Ok(())
	}

	async fn handle_sse_item(&self, item: sse::SSE) {
		match item {
			sse::SSE::Event(event) => {
				// Process different event types according to MCP protocol
				match event.event_type.as_str() {
					"ping" => {
						debug!("Received MCP ping, sending pong");
						self.send_pong().await;
					}
					"message" => {
						// Parse the message as JSON
						if let Ok(json) = serde_json::from_str::<Value>(&event.data) {
							self.handle_jsonrpc_message(json).await;
						} else {
							debug!("Received non-JSON message: {}", event.data);
						}
					}
					_ => {
						debug!(
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

	async fn send_pong(&self) {
		// For now, we just log that we would send a pong response
		debug!("Would send pong response if required by the protocol");
	}

	async fn handle_jsonrpc_message(&self, json: Value) {
		// Check if it's a JSON-RPC message
		if let Some(jsonrpc) = json.get("jsonrpc") {
			if jsonrpc.as_str() == Some("2.0") {
				// Handle different types of JSON-RPC messages
				if let Some(method) = json.get("method").and_then(|m| m.as_str()) {
					// This is a request or notification
					self.handle_jsonrpc_request(method, json.clone()).await;
				} else if json.get("result").is_some() || json.get("error").is_some() {
					// This is a response
					self.handle_jsonrpc_response(json).await;
				}
			} else {
				debug!("Non 2.0 JSON-RPC message: {:?}", json);
			}
		} else {
			// Process as a regular JSON message for backward compatibility
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
				debug!("Received JSON message without type: {:?}", json);
			}
		}
	}

	async fn handle_jsonrpc_request(&self, method: &str, json: Value) {
		info!("Received JSON-RPC request for method: {}", method);

		let id = json.get("id").map(|id| id.to_string().trim_matches('"').to_string());

		// Process different method types
		match method {
			"callTool" => {
				// Tool call handling
				if let Some(params) = json.get("params") {
					if let Some(tool_name) = params.get("name").and_then(|n| n.as_str()) {
						info!("Received tool call for: {}", tool_name);

						// Log the parameters
						if let Some(args) = params.get("arguments") {
							debug!("Tool arguments: {:?}", args);
						}

						// For now, we just log the call. In a future implementation,
						// we could forward this to the appropriate tool handler
					}
				}
			}
			"initialize" => {
				info!("Received initialize request");
				if let Some(id) = id {
					debug!("Sending initialize response for id: {}", id);
					// In a future implementation, we would send a proper initialize response
				}
			}
			"$/cancelRequest" => {
				if let Some(params) = json.get("params") {
					if let Some(cancel_id) = params.get("id") {
						info!("Request cancelled: {}", cancel_id);
					}
				}
			}
			_ => {
				debug!("Unhandled JSON-RPC method: {}", method);
			}
		}
	}

	async fn handle_jsonrpc_response(&self, json: Value) {
		if let Some(id) = json.get("id") {
			if let Some(result) = json.get("result") {
				info!("Received successful response for request {}: {:?}", id, result);
			} else if let Some(error) = json.get("error") {
				error!("Received error response for request {}: {:?}", id, error);
			}
		} else {
			debug!("Received response without ID: {:?}", json);
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use serde_json::json;

	#[test]
	fn test_parse_jsonrpc_request() {
		let message = json!({
			"jsonrpc": "2.0",
			"method": "callTool",
			"params": {
				"name": "testTool",
				"arguments": {
					"arg1": "value1"
				}
			},
			"id": 1
		});

		// Ensure this doesn't panic
		let connect_mcp = ConnectMcp {
			url: "https://example.com".to_string(),
			client: true,
		};

		// We can't easily test the async function directly in a sync test,
		// but we can verify the handler doesn't panic when handling basic patterns
		let rt = Runtime::new().unwrap();
		rt.block_on(async {
			connect_mcp.handle_jsonrpc_message(message).await;
		});

		// Test response handling
		let response = json!({
			"jsonrpc": "2.0",
			"result": {
				"success": true
			},
			"id": 1
		});

		rt.block_on(async {
			connect_mcp.handle_jsonrpc_message(response).await;
		});
	}
}
