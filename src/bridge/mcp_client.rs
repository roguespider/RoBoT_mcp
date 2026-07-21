// src/bridge/mcp_client.rs
// MCP Client implementation for connecting to external MCP servers

use std::sync::Arc;

use anyhow::Result;
use rmcp::{
    ClientHandler,
    model::{CallToolRequestParams, ClientInfo, Tool},
    service::{RoleClient, RunningService},
};
use tokio::sync::RwLock;
use tokio::process::Command;

/// A connected MCP server
struct ConnectedServer {
    name: String,
    /// The running service - kept alive to maintain the connection
    _running: RunningService<RoleClient, SimpleClientHandler>,
    /// Cached tools from this server
    tools: Vec<Tool>,
}

/// Tool invocation error
#[derive(Debug)]
pub struct ToolError {
    pub message: String,
    pub server: String,
    pub tool: String,
}

impl std::fmt::Display for ToolError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}] {}: {}", self.server, self.tool, self.message)
    }
}

impl std::error::Error for ToolError {}

/// MCP Client for connecting to external MCP servers
pub struct McpClient {
    /// Connected servers and their tools
    servers: Arc<RwLock<Vec<ConnectedServer>>>,
}

impl McpClient {
    /// Create a new MCP client
    pub fn new() -> Self {
        Self {
            servers: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Connect to an MCP server via child process transport
    pub async fn connect(&self, name: &str, command: &str, args: &[&str]) -> Result<()> {
        use rmcp::transport::child_process::TokioChildProcess;
        
        tracing::info!("Connecting to MCP server '{}': {} {:?}", name, command, args);

        // Create child process transport
        let mut cmd = Command::new(command);
        cmd.args(args);
        let transport = TokioChildProcess::new(cmd)?;
        
        // Create client handler
        let client = SimpleClientHandler {
            info: ClientInfo::default(),
        };

        // Start the client and get the running service
        let running = rmcp::serve_client(client, transport).await?;
        
        // Get the peer to list tools
        let peer = running.peer().clone();
        
        // List tools from the server
        let tools = match peer.list_all_tools().await {
            Ok(tools) => {
                tracing::info!("Server '{}' exposed {} tools", name, tools.len());
                tools
            }
            Err(e) => {
                tracing::warn!("Failed to list tools from '{}': {}", name, e);
                Vec::new()
            }
        };

        let tools_count = tools.len();

        // Store the server connection
        let server = ConnectedServer {
            name: name.to_string(),
            _running: running,
            tools,
        };

        self.servers.write().await.push(server);
        
        tracing::info!("MCP client connected to '{}' with {} tools", name, tools_count);
        Ok(())
    }

    /// List tools from all connected servers
    pub async fn list_all_tools(&self) -> Vec<Tool> {
        let servers = self.servers.read().await;
        let mut tools = Vec::new();
        for server in servers.iter() {
            tools.extend(server.tools.clone());
        }
        tools
    }

    /// Get a specific tool by name
    pub async fn get_tool(&self, name: &str) -> Option<Tool> {
        let servers = self.servers.read().await;
        for server in servers.iter() {
            if let Some(tool) = server.tools.iter().find(|t| t.name == name) {
                return Some(tool.clone());
            }
        }
        None
    }

    /// Call a tool on a connected server
    pub async fn call_tool(
        &self,
        tool_name: &str,
        arguments: Option<serde_json::Value>,
    ) -> Result<serde_json::Value, ToolError> {
        // Find the server that has this tool
        let server_name;
        {
            let servers = self.servers.read().await;
            let found = servers.iter().find(|s| s.tools.iter().any(|t| t.name == tool_name));
            match found {
                Some(s) => server_name = s.name.clone(),
                None => return Err(ToolError {
                    message: format!("Tool '{}' not found on any connected server", tool_name),
                    server: "unknown".to_string(),
                    tool: tool_name.to_string(),
                }),
            }
        }

        // Get a reference to the running service
        let peer = {
            let servers = self.servers.read().await;
            let server = servers.iter().find(|s| s.tools.iter().any(|t| t.name == tool_name)).unwrap();
            server._running.peer().clone()
        };

        // Call the tool via the server's peer
        let params = match arguments {
            Some(v) => CallToolRequestParams::new(tool_name.to_string())
                .with_arguments(v.as_object().cloned().unwrap_or_default()),
            None => CallToolRequestParams::new(tool_name.to_string()),
        };

        match peer.call_tool(params).await {
            Ok(result) => {
                // Extract content from the result
                if let Some(content) = result.content.first() {
                    if let Some(text) = content.as_text() {
                        match serde_json::from_str(&text.text) {
                            Ok(json) => Ok(json),
                            Err(_) => Ok(serde_json::json!(text.text)),
                        }
                    } else {
                        Ok(serde_json::json!(content))
                    }
                } else {
                    Ok(serde_json::json!({"result": "ok"}))
                }
            }
            Err(e) => Err(ToolError {
                message: format!("Tool call failed: {:?}", e),
                server: server_name,
                tool: tool_name.to_string(),
            }),
        }
    }
}

impl Default for McpClient {
    fn default() -> Self {
        Self::new()
    }
}

/// A simple MCP client handler that does nothing
struct SimpleClientHandler {
    info: ClientInfo,
}

impl ClientHandler for SimpleClientHandler {
    fn get_info(&self) -> ClientInfo {
        self.info.clone()
    }
}
