// src/bridge/mcp.rs
// MCP (Model Context Protocol) core types and traits

use std::sync::Arc;

use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::database::sqlite::SqliteDatabase;
use crate::experience::bus::ExperienceBus;
use crate::experience::coordinator::ExperienceCoordinator;
use crate::experience::evolution::EvolutionEngine;
use crate::experience::metrics::MetricsCollector;
use crate::experience::reflection::ReflectionEngine;
use crate::experience::scheduler::Scheduler;

/// MCP protocol version
pub const MCP_VERSION: &str = "2024-11-05";

/// MCP message types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum McpMessage {
    Request(McpRequest),
    Response(McpResponse),
    Notification(McpNotification),
}

/// MCP request message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpRequest {
    pub method: String,
    pub params: Option<serde_json::Value>,
    pub id: String,
}

/// MCP response message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpResponse {
    pub result: Option<serde_json::Value>,
    pub error: Option<McpError>,
    pub id: String,
}

/// MCP error
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpError {
    pub code: i32,
    pub message: String,
    pub data: Option<serde_json::Value>,
}

/// MCP notification message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpNotification {
    pub method: String,
    pub params: Option<serde_json::Value>,
}

/// Tool definition for MCP
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpTool {
    pub name: String,
    pub description: String,
    pub input_schema: serde_json::Value,
}

/// Resource definition for MCP
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpResource {
    pub uri: String,
    pub name: String,
    pub description: Option<String>,
    pub mime_type: Option<String>,
}

/// Prompt definition for MCP
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpPrompt {
    pub name: String,
    pub description: Option<String>,
    pub arguments: Vec<McpPromptArgument>,
}

/// Argument for a prompt
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpPromptArgument {
    pub name: String,
    pub description: Option<String>,
    pub required: bool,
}

/// Initialize request parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InitializeParams {
    pub protocol_version: String,
    pub capabilities: McpCapabilities,
    pub client_info: McpClientInfo,
}

/// Server capabilities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpCapabilities {
    #[serde(default)]
    pub tools: Option<McpEmpty>,
    #[serde(default)]
    pub resources: Option<McpResourcesCapability>,
    #[serde(default)]
    pub prompts: Option<McpEmpty>,
    #[serde(default)]
    pub logging: Option<McpEmpty>,
}

/// Empty capability marker
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct McpEmpty;

/// Resources capability
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpResourcesCapability {
    pub subscribe: Option<bool>,
    pub list_changed: Option<bool>,
}

/// Client information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpClientInfo {
    pub name: String,
    pub version: String,
}

/// Server information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpServerInfo {
    pub name: String,
    pub version: String,
}

/// Trait for MCP protocol handlers
pub trait McpHandler: Send + Sync {
    /// Handle an MCP request
    fn handle_request(&self, request: McpRequest) -> Result<McpResponse>;
    
    /// Handle an MCP notification
    fn handle_notification(&self, notification: McpNotification) -> Result<()>;
    
    /// Get server capabilities
    fn get_capabilities(&self) -> McpCapabilities;
    
    /// Get server info
    fn get_server_info(&self) -> McpServerInfo;
}

/// McpBridge context shared across handlers
pub struct McpContext {
    /// Database layer
    pub database: Arc<SqliteDatabase>,
    
    /// Event bus
    pub bus: Arc<ExperienceBus>,
    
    /// Experience coordinator
    pub coordinator: Arc<ExperienceCoordinator>,
    
    /// Reflection engine
    pub reflection: Arc<ReflectionEngine>,
    
    /// Evolution engine
    pub evolution: Arc<EvolutionEngine>,
    
    /// Background scheduler
    pub scheduler: Arc<Scheduler>,
    
    /// Metrics collector
    pub metrics: Arc<MetricsCollector>,
    
    /// Knowledge system - manages validated knowledge
    pub knowledge: Arc<crate::knowledge::KnowledgeStore>,
    
    /// Planner - task decomposition and execution
    pub planner: Arc<crate::planner::Planner>,
    
    /// Policy engine - decision-making rules
    pub policy: Arc<crate::planner::PolicyEngine>,
    
    /// Working memory - short-term memory layer
    pub working_memory: Arc<crate::memory::WorkingMemory>,
    
    /// Permanent memory - long-term memory layer  
    pub permanent_memory: Arc<crate::memory::PermanentMemory>,
    
    /// Memory retrieval - unified retrieval across layers
    pub memory_retrieval: Arc<crate::memory::MemoryRetrieval>,
    
    /// Server info
    pub server_info: McpServerInfo,
    
    /// Server capabilities
    pub capabilities: McpCapabilities,
}

impl McpContext {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        database: Arc<SqliteDatabase>,
        bus: Arc<ExperienceBus>,
        coordinator: Arc<ExperienceCoordinator>,
        reflection: Arc<ReflectionEngine>,
        evolution: Arc<EvolutionEngine>,
        scheduler: Arc<Scheduler>,
        metrics: Arc<MetricsCollector>,
        knowledge: Arc<crate::knowledge::KnowledgeStore>,
        planner: Arc<crate::planner::Planner>,
        policy: Arc<crate::planner::PolicyEngine>,
        working_memory: Arc<crate::memory::WorkingMemory>,
        permanent_memory: Arc<crate::memory::PermanentMemory>,
        memory_retrieval: Arc<crate::memory::MemoryRetrieval>,
    ) -> Self {
        Self {
            database,
            bus,
            coordinator,
            reflection,
            evolution,
            scheduler,
            metrics,
            knowledge,
            planner,
            policy,
            working_memory,
            permanent_memory,
            memory_retrieval,
            server_info: McpServerInfo {
                name: env!("CARGO_PKG_NAME").to_string(),
                version: env!("CARGO_PKG_VERSION").to_string(),
            },
            capabilities: McpCapabilities {
                tools: Some(McpEmpty),
                resources: Some(McpResourcesCapability {
                    subscribe: Some(true),
                    list_changed: Some(true),
                }),
                prompts: Some(McpEmpty),
                logging: Some(McpEmpty),
            },
        }
    }
}

// ============================================================================
// MCP Client - For connecting to external MCP servers
// ============================================================================

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
    running: RunningService<RoleClient, SimpleClientHandler>,
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
            running,
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

        // Get a reference to the running service (server_name already found above)
        let peer = {
            let servers = self.servers.read().await;
            let server = servers.iter().find(|s| s.name == server_name)
                .expect("Server should exist - name was validated above");
            server.running.peer().clone()
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
