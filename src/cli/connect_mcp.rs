use anyhow::{anyhow, Context, Result};
use clap::Parser;
use log::{error, info, trace, warn};
use mcpr::{
	io::*, // Brings in JsonRpcTransport and related traits
	server::{RequestHandler, Server},
	spec::*, // Brings in message types like InitializeRequest, ToolCallRequest etc.
	transport::stdio::StdioTransport,
	McprError, // Added McprError import explicitly
};
use reqwest::Client as ReqwestClient; // Added reqwest
use serde::{Deserialize, Serialize}; // Added serde
use serde_json::json;
use std::{env, time::Duration}; // Added env and Duration
use tokio::runtime::Runtime;

// --- Pinecone Structs ---
#[derive(Serialize, Deserialize, Debug)]
struct PineconeSnippet {
	#[serde(default)]
	r#type: String, // Use raw identifier as type is a keyword
	#[serde(default)]
	content: String,
	#[serde(default)]
	score: f64,
	#[serde(default)]
	reference: serde_json::Value, // Keep as Value for flexibility
}

#[derive(Serialize, Deserialize, Debug)]
struct PineconeResponse {
	#[serde(default)]
	snippets: Vec<PineconeSnippet>,
}
// --- End Pinecone Structs ---

/// Connect to an MCP stdio server locally
#[derive(Parser)]
pub struct ConnectMcp {
	// Keep URL optional for now, might not be needed
	#[arg()]
	url: Option<String>,
}

// Server state can hold the reqwest client and API key
struct ArgonMcpState {
	http_client: ReqwestClient,
	pinecone_api_key: Option<String>,
}

impl ArgonMcpState {
	fn new() -> Self {
		// Attempt to read API key from environment variable
		let pinecone_api_key = env::var("PINECONE_API_KEY").ok();
		if pinecone_api_key.is_none() {
			warn!("PINECONE_API_KEY environment variable not set. Pinecone requests will fail.");
		}

		ArgonMcpState {
			http_client: ReqwestClient::builder()
				.timeout(Duration::from_secs(30)) // Add a timeout
				.build()
				.unwrap_or_else(|e| {
					error!("Failed to build Reqwest client: {}", e);
					// Return a default client even on error, requests will likely fail
					ReqwestClient::new()
				}),
			pinecone_api_key,
		}
	}
}

// Request handler holds the state
struct ArgonMcpHandler {
	state: ArgonMcpState,
}

// Helper function to call Pinecone
async fn get_pinecone_context(
	state: &ArgonMcpState,
	assistant_name: &str,
	query: &str,
	top_k: usize,
) -> Result<PineconeResponse, String> { // Return String error for simplicity
	let api_key = match &state.pinecone_api_key {
		Some(key) => key,
		None => return Err("Pinecone API key is not configured.".to_string()),
	};

	let endpoint = format!(
		"https://prod-1-data.ke.pinecone.io/assistant/chat/{}/context",
		assistant_name
	);
	let payload = json!({
		"query": query,
		"top_k": top_k
	});

	trace!("Querying Pinecone endpoint: {}", endpoint);
	trace!("Payload: {}", payload);

	match state
		.http_client
		.post(&endpoint)
		.header("Api-Key", api_key)
		.header("accept", "application/json")
		.header("Content-Type", "application/json")
		.header("X-Pinecone-API-Version", "2025-04") // Match TS example
		.json(&payload)
		.send()
		.await
	{
		Ok(response) => {
			let status = response.status();
			trace!("Pinecone response status: {}", status);
			if status.is_success() {
				match response.json::<PineconeResponse>().await {
					Ok(data) => Ok(data),
					Err(e) => Err(format!("Failed to parse Pinecone JSON response: {}", e)),
				}
			} else {
				let error_body = response.text().await.unwrap_or_else(|_| "Failed to read error body".to_string());
				Err(format!("Pinecone request failed with status {}: {}", status, error_body))
			}
		}
		Err(e) => Err(format!("Failed to send request to Pinecone: {}", e)),
	}
}

#[async_trait::async_trait]
impl RequestHandler for ArgonMcpHandler {
	async fn initialize(
		&mut self,
		_req: InitializeRequest,
	) -> Result<InitializeResult, McprError> {
		trace!("Received initialize request");
		// Define the tools our local server provides
		Ok(InitializeResult {
			protocol_version: MCP_VERSION.to_string(),
			server_info: ServerInfo {
				name: "argon-mcp-local".to_string(),
				version: env!("CARGO_PKG_VERSION").to_string(), // Use argon's version
			},
			tools: vec![ToolDefinition {
				name: "local_rag".to_string(),
				description: "Performs RAG locally using available context.".to_string(),
				input_schema: json!({ // Define expected input schema for the RAG tool
					"type": "object",
					"properties": {
						"query": { "type": "string", "description": "The user query for RAG." },
						// Add other parameters needed for RAG (e.g., file context?)
					},
					"required": ["query"]
				}),
			}],
		})
	}

	async fn tools_list(
		&mut self,
		_req: ToolsListRequest,
	) -> Result<ToolsListResult, McprError> {
		trace!("Received tools/list request");
		// Reuse the tools defined in initialize
		let init_result = self.initialize(InitializeRequest::default()).await?;
		Ok(ToolsListResult {
			tools: init_result.tools,
		})
	}

	async fn tools_call(
		&mut self,
		req: ToolCallRequest,
	) -> Result<ToolCallResult, McprError> {
		trace!("Received tools/call request: {:?}", req);

		match req.name.as_str() {
			"local_rag" => {
				// --- RAG Logic Placeholder ---
				// 1. Extract parameters from req.arguments
				let query = req
					.arguments
					.get("query")
					.and_then(|v| v.as_str())
					.ok_or_else(|| {
						McprError::invalid_params("Missing or invalid 'query' parameter")
					})?;

				info!("Performing local RAG for query: {}", query);

				// 2. Implement or call the local RAG logic here
				//    - Access vector stores?
				//    - Search files?
				//    - Call LLM APIs?
				//    - This might involve calling other functions within the argon crate
				//      or even spawning external processes if necessary.
				let rag_result = format!("Successfully performed RAG for query: '{}'. Result data goes here.", query);

				// 3. Format the result according to expected output schema
				Ok(ToolCallResult {
					content: json!({ // Define output schema
						"success": true,
						"result_text": rag_result,
						// Add other result fields
					}),
				})
				// --- End RAG Logic Placeholder ---
			}
			_ => {
				error!("Unknown tool called: {}", req.name);
				Err(McprError::method_not_found(&format!(
					"Tool '{}' not found",
					req.name
				)))
			}
		}
	}

	// Implement other handlers like shutdown if needed
	async fn shutdown(&mut self, _req: ShutdownRequest) -> Result<(), McprError> {
		trace!("Received shutdown request");
		// Perform any cleanup before exiting
		Ok(())
	}
}

impl ConnectMcp {
	pub fn main(self) -> Result<()> {
		let rt = Runtime::new().context("Failed to create Tokio runtime")?;
		rt.block_on(async { self.run_server().await })
	}

	async fn run_server(&self) -> Result<()> {
		info!("Starting Argon MCP server using mcpr (stdio)...");

		// Create the transport
		let transport = StdioTransport::new();

		// Create the handler
		let handler = ArgonMcpHandler {}; // Add state if needed

		// Create and run the server
		let server = Server::new(handler);

		// Use JsonRpcTransport for stdio
		let mut transport_handler = JsonRpcTransport::new(transport);

		// Run the server loop
		if let Err(e) = server.run(&mut transport_handler).await {
			error!("MCP server error: {}", e);
			return Err(anyhow!("MCP server failed: {}", e));
		}

		info!("Argon MCP server finished.");
		Ok(())
	}
}

// Remove the old stdio_reader_loop and SSE handling functions

// Keep tests if relevant, or update/remove them
#[cfg(test)]
mod tests {
	// Add tests for the new MCPR-based implementation if needed
}
