// src/tools/agent.rs
// Agent-related MCP tools

use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::bridge::mcp_client::McpClient;
use crate::tools::get_tools;

/// Global MCP client instance
static MCP_CLIENT: std::sync::OnceLock<Arc<McpClient>> = std::sync::OnceLock::new();

/// Initialize the global MCP client
pub fn init_mcp_client(client: Arc<McpClient>) {
    let _ = MCP_CLIENT.set(client);
}

/// Get the global MCP client
fn get_mcp_client() -> Option<Arc<McpClient>> {
    MCP_CLIENT.get().cloned()
}

/// Tool input for ping (no parameters needed)
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct PingInput {}

/// Tool input for listing available tools
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct ListToolsInput {
    pub filter: Option<String>,
}

/// Tool input for getting tool details
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct GetToolInput {
    pub name: String,
}

/// Tool input for connecting to an MCP server
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct ConnectMcpServerInput {
    pub name: String,
    pub command: String,
    pub args: Option<Vec<String>>,
}

/// Tool input for calling an MCP tool
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct CallMcpToolInput {
    pub tool_name: String,
    pub arguments: Option<serde_json::Value>,
}

/// Agent tool definitions
pub mod definitions {
    pub const PING: &str = "ping";
    pub const LIST_TOOLS: &str = "list_tools";
    pub const GET_TOOL: &str = "get_tool";
    pub const CONNECT_MCP_SERVER: &str = "connect_mcp_server";
    pub const CALL_MCP_TOOL: &str = "call_mcp_tool";

    pub fn all() -> Vec<crate::bridge::mcp::McpTool> {
        vec![
            crate::bridge::mcp::McpTool {
                name: PING.to_string(),
                description: "Ping the MCP server to verify connection".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {}
                }),
            },
            crate::bridge::mcp::McpTool {
                name: LIST_TOOLS.to_string(),
                description: "List all available MCP tools with optional filter".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "filter": {
                            "type": "string",
                            "description": "Optional filter to match tool names or descriptions"
                        }
                    }
                }),
            },
            crate::bridge::mcp::McpTool {
                name: GET_TOOL.to_string(),
                description: "Get detailed information about a specific tool".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "name": {
                            "type": "string",
                            "description": "The name of the tool to get details for"
                        }
                    },
                    "required": ["name"]
                }),
            },
            crate::bridge::mcp::McpTool {
                name: CONNECT_MCP_SERVER.to_string(),
                description: "Connect to an external MCP server via child process".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "name": {
                            "type": "string",
                            "description": "Name to identify this server"
                        },
                        "command": {
                            "type": "string",
                            "description": "Path to the MCP server executable"
                        },
                        "args": {
                            "type": "array",
                            "items": { "type": "string" },
                            "description": "Command line arguments for the server"
                        }
                    },
                    "required": ["name", "command"]
                }),
            },
            crate::bridge::mcp::McpTool {
                name: CALL_MCP_TOOL.to_string(),
                description: "Call a tool on a connected MCP server".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "tool_name": {
                            "type": "string",
                            "description": "Name of the tool to call"
                        },
                        "arguments": {
                            "type": "object",
                            "description": "Arguments to pass to the tool (optional)"
                        }
                    },
                    "required": ["tool_name"]
                }),
            },
        ]
    }
}

/// Execute list_tools tool
pub async fn execute_list_tools(input: ListToolsInput) -> Result<serde_json::Value, anyhow::Error> {
    let all_tools = get_tools();
    let total_count = all_tools.len();
    
    let filtered_tools: Vec<serde_json::Value> = all_tools
        .into_iter()
        .filter(|tool| {
            if let Some(ref filter) = input.filter {
                let filter_lower = filter.to_lowercase();
                tool.name.to_lowercase().contains(&filter_lower)
                    || tool.description.to_lowercase().contains(&filter_lower)
            } else {
                true
            }
        })
        .map(|tool| {
            serde_json::json!({
                "name": tool.name,
                "description": tool.description,
                "input_schema": tool.input_schema
            })
        })
        .collect();

    Ok(serde_json::json!({
        "tools": filtered_tools,
        "count": filtered_tools.len(),
        "total_available": total_count
    }))
}

/// Execute get_tool tool
pub async fn execute_get_tool(input: GetToolInput) -> Result<serde_json::Value, anyhow::Error> {
    let all_tools = get_tools();
    
    let tool = all_tools
        .into_iter()
        .find(|t| t.name == input.name);

    match tool {
        Some(t) => Ok(serde_json::json!({
            "found": true,
            "tool": {
                "name": t.name,
                "description": t.description,
                "input_schema": t.input_schema
            }
        })),
        None => Ok(serde_json::json!({
            "found": false,
            "tool": null,
            "error": format!("Tool '{}' not found", input.name)
        })),
    }
}

/// Execute ping tool - verifies connection to the MCP server
pub async fn execute_ping(_input: PingInput) -> Result<serde_json::Value, anyhow::Error> {
    Ok(serde_json::json!({
        "status": "ok",
        "message": "MCP server is running",
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "available_tools": get_tools().len()
    }))
}

/// Execute connect_mcp_server tool - connect to an external MCP server
pub async fn execute_connect_mcp_server(input: ConnectMcpServerInput) -> Result<serde_json::Value, anyhow::Error> {
    let client = match get_mcp_client() {
        Some(c) => c,
        None => return Ok(serde_json::json!({
            "success": false,
            "error": "MCP client not initialized"
        })),
    };

    let args_vec = input.args.unwrap_or_default();
    let args: Vec<&str> = args_vec.iter().map(|s| s.as_str()).collect();

    match client.connect(&input.name, &input.command, &args).await {
        Ok(()) => Ok(serde_json::json!({
            "success": true,
            "server": input.name,
            "tools": client.list_all_tools().await.len()
        })),
        Err(e) => Ok(serde_json::json!({
            "success": false,
            "error": e.to_string()
        })),
    }
}

/// Execute call_mcp_tool tool - call a tool on a connected MCP server
pub async fn execute_call_mcp_tool(input: CallMcpToolInput) -> Result<serde_json::Value, anyhow::Error> {
    let client = match get_mcp_client() {
        Some(c) => c,
        None => return Ok(serde_json::json!({
            "success": false,
            "error": "MCP client not initialized"
        })),
    };

    match client.call_tool(&input.tool_name, input.arguments).await {
        Ok(result) => Ok(serde_json::json!({
            "success": true,
            "result": result
        })),
        Err(e) => Ok(serde_json::json!({
            "success": false,
            "error": e.to_string()
        })),
    }
}
