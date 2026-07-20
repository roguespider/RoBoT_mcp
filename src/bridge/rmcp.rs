// src/bridge/rmcp.rs
// RMCP (Rust MCP) server implementation using the rmcp crate

use std::sync::Arc;

use anyhow::Result;
use rmcp::{
    serve_server,
    ServerHandler,
};

use super::mcp::McpContext;

/// RMCP server wrapper for MCP bridge
pub struct RmcpServer {
    context: Arc<McpContext>,
}

impl RmcpServer {
    /// Get the shared context
    pub fn context(&self) -> Arc<McpContext> {
        Arc::clone(&self.context)
    }
}

/// Create a new RMCP server with stdio transport
pub async fn run_stdio_server(
    name: &str,
    version: &str,
    context: Arc<McpContext>,
) -> Result<()> {
    tracing::info!("Starting RMCP server '{}' v{} with stdio transport", name, version);
    
    let handler = McpServerHandler {
        context,
        name: name.to_string(),
        version: version.to_string(),
    };
    
    let (stdin, stdout) = (tokio::io::stdin(), tokio::io::stdout());
    
    serve_server(handler, (stdin, stdout)).await?;
    
    Ok(())
}

/// MCP Server handler using the rmcp derive macros
#[derive(Clone)]
struct McpServerHandler {
    context: Arc<McpContext>,
    name: String,
    version: String,
}

impl McpServerHandler {
    fn new(context: Arc<McpContext>, name: String, version: String) -> Self {
        Self { context, name, version }
    }
}

impl ServerHandler for McpServerHandler {}
