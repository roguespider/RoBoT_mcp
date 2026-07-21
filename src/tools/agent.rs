// src/tools/agent.rs
// Agent-related MCP tools

use serde::{Deserialize, Serialize};

use crate::tools::get_tools;

/// Tool input for ping (no parameters needed)
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct PingInput;

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

/// Agent tool definitions
pub mod definitions {
    pub const PING: &str = "ping";
    pub const LIST_TOOLS: &str = "list_tools";
    pub const GET_TOOL: &str = "get_tool";

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
