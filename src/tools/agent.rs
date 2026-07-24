// src/tools/agent.rs
// Agent-related MCP tools


use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::bridge::mcp::McpClient;
use crate::tools::{get_tools_async, ToolOutput};

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

/// Tool input for getting workflow rules (MUST be called first)
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct GetWorkflowInput {
    pub purpose: Option<String>,
}

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
    /// JSON-encoded arguments as a string (e.g., "{\"key\": \"value\"}")
    pub arguments: Option<String>,
}

/// Agent tool definitions
pub mod definitions {
    pub const GET_WORKFLOW: &str = "get_workflow";
    pub const LIST_TOOLS: &str = "list_tools";
    pub const GET_TOOL: &str = "get_tool";
    pub const CONNECT_MCP_SERVER: &str = "connect_mcp_server";
    #[allow(dead_code)]
    pub const CALL_TOOL: &str = "call_tool";
    pub const CALL_MCP_TOOL: &str = "call_tool"; // Alias

    pub fn all() -> Vec<crate::bridge::mcp::McpTool> {
        vec![
            crate::bridge::mcp::McpTool {
                name: GET_WORKFLOW.to_string(),
                description: "MANDATORY: Get workflow rules. MUST be called before any other tool. Returns the required workflow for this MCP server.".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "purpose": {
                            "type": "string",
                            "description": "Context for why you need the workflow (e.g., 'file_ingestion', 'memory_search', 'general')"
                        }
                    }
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
                            "type": "string",
                            "description": "JSON-encoded arguments to pass to the tool (e.g., '{\"key\": \"value\"}')"
                        }
                    },
                    "required": ["tool_name"]
                }),
            },
        ]
    }
}

/// Execute get_workflow tool - MUST be called before any other tool
pub async fn execute_get_workflow(input: GetWorkflowInput) -> Result<ToolOutput, anyhow::Error> {
    let purpose = input.purpose.unwrap_or_else(|| "general".to_string());
    
    let workflow = match purpose.to_lowercase().as_str() {
        "file_ingestion" | "ingest" | "import" => serde_json::json!({
            "workflow_name": "File Ingestion Workflow",
            "mandatory_steps": [
                {
                    "step": 1,
                    "tool": "get_workflow",
                    "action": "Review workflow rules",
                    "description": "You called get_workflow - good start!"
                },
                {
                    "step": 2,
                    "tool": "list_importable",
                    "action": "Check available files in files_to_import folder",
                    "parameters": {"recursive": true},
                    "description": "Lists files ready for ingestion. Use recursive=true to include subfolders. Files with skip_reason are NOT ingestible (size limits, embedding patterns)."
                },
                {
                    "step": 3,
                    "tool": "ingest_files",
                    "action": "Ingest ONE file at a time",
                    "parameters": {"limit": 1, "recursive": true},
                    "description": "Use limit=1 for single file. NEVER batch ingest without explicit instruction."
                },
                {
                    "step": 4,
                    "tool": "check response.NEXT_ACTION",
                    "action": "Follow the NEXT_ACTION in the response",
                    "description": "The ingest response includes a 'NEXT_ACTION' field telling you what to do next."
                },
                {
                    "step": 5,
                    "tool": "search_memory",
                    "action": "Verify ingestion in memory",
                    "description": "Search to confirm file was stored correctly."
                },
                {
                    "step": 6,
                    "tool": "ASK USER",
                    "action": "Follow the NEXT_ACTION guidance - typically ask for deletion permission",
                    "description": "NEVER delete without explicit user confirmation!"
                },
                {
                    "step": 7,
                    "tool": "delete_ingested_files",
                    "action": "Delete original file only after user confirmation",
                    "parameters": {"confirmation": "yes"},
                    "description": "Use files from 'files_ready_for_deletion' in the ingest response. confirmation MUST be 'yes' (exactly)."
                }
            ],
            "critical_rules": [
                "ALWAYS use limit=1 for single file ingestion",
                "NEVER batch ingest without explicit user instruction",
                "ALWAYS follow the NEXT_ACTION in ingest response",
                "ALWAYS ask user before calling delete_ingested_files",
                "confirmation parameter MUST be exactly 'yes'",
                "Folders are NOT deleted - only files",
                "files_to_import is relative to executable location"
            ],
            "files_that_are_skipped": [
                "JSON files >10MB (embedding/metadata files don't chunk well)",
                "Text files >50MB (size limit to prevent timeouts)",
                "Files matching patterns: embeddings, vectors, chroma, pinecone, faiss, metadata, etc.",
                "Files already ingested (tracked in memory, not offered again)"
            ],
            "common_mistakes_to_avoid": [
                "Calling ingest_files without limit=1 (causes batch ingest)",
                "Calling delete_ingested_files without asking user first",
                "Using confirmation values other than 'yes'",
                "Trying to delete folders instead of files",
                "Forgetting to verify ingestion with search_memory"
            ]
        }),
        
        "memory_search" | "search" | "memory" => serde_json::json!({
            "workflow_name": "Memory Search Workflow",
            "mandatory_steps": [
                {
                    "step": 1,
                    "tool": "get_workflow",
                    "action": "Review workflow rules",
                    "description": "You called get_workflow - good start!"
                },
                {
                    "step": 2,
                    "tool": "global_search",
                    "action": "Search across all memories",
                    "description": "Use query parameter to find relevant memories."
                },
                {
                    "step": 3,
                    "tool": "get_patterns",
                    "action": "Check for relevant patterns",
                    "description": "Patterns may contain learned knowledge."
                },
                {
                    "step": 4,
                    "tool": "get_insights",
                    "action": "Review actionable insights",
                    "description": "Insights with high confidence can guide decisions."
                }
            ],
            "critical_rules": [
                "Use global_search for comprehensive results",
                "Review patterns before making repetitive decisions",
                "Consider insight confidence levels when making decisions"
            ]
        }),
        
        _ => serde_json::json!({
            "workflow_name": "General MCP Workflow",
            "mandatory_steps": [
                {
                    "step": 1,
                    "tool": "get_workflow",
                    "action": "Review workflow rules",
                    "description": "You called get_workflow - good start! Always call this first."
                },
                {
                    "step": 2,
                    "tool": "list_tools",
                    "action": "See all available tools",
                    "description": "Get full list of MCP tools."
                },
                {
                    "step": 3,
                    "tool": "search_memory",
                    "action": "Check existing memory for relevant context",
                    "description": "Always check memory before taking action."
                },
                {
                    "step": 4,
                    "tool": "get_patterns",
                    "action": "Review learned patterns",
                    "description": "Patterns may inform your approach."
                },
                {
                    "step": 5,
                    "tool": "PROCEED",
                    "action": "Take action based on gathered context",
                    "description": "Now you have context - proceed with your task."
                }
            ],
            "critical_rules": [
                "MUST call get_workflow first before ANY other tool",
                "MUST check memory (search_memory) before taking action",
                "MUST review patterns (get_patterns) for repetitive decisions",
                "ALWAYS ask user before destructive operations (delete_ingested_files)"
            ],
            "destructive_operations": {
                "delete_ingested_files": {
                    "requires_confirmation": true,
                    "confirmation_value": "yes",
                    "warning": "This deletes files permanently!"
                }
            },
            "directory_structure": {
                "exe_location": "robot_brain.exe or robot_brain",
                "db_location": "robot_brain.db (in same directory as exe)",
                "import_folder": "files_to_import/ (in same directory as exe)",
                "note": "All paths are relative to executable location"
            },
            "quick_reference": {
                "list_importable": "Check files available for import",
                "ingest_files": "Ingest files (use limit=1 for single file)",
                "delete_ingested_files": "Delete files (MUST have confirmation='yes')",
                "search_memory": "Search stored memories",
                "global_search": "Search all data types",
                "analyze_patterns": "Detect patterns in experiences",
                "get_patterns": "Get stored patterns",
                "get_insights": "Get actionable insights"
            }
        })
    };

    Ok(ToolOutput::success(serde_json::json!({
        "status": "workflow_retrieved",
        "workflow": workflow,
        "reminder": "You MUST follow this workflow. Call get_workflow again anytime you need a reminder.",
        "version": "1.0"
    })))
}

/// Execute list_tools tool
pub async fn execute_list_tools(input: ListToolsInput) -> Result<ToolOutput, anyhow::Error> {
    let all_tools = get_tools_async().await;
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

    Ok(ToolOutput::success(serde_json::json!({
        "tools": filtered_tools,
        "count": filtered_tools.len(),
        "total_available": total_count
    })))
}

/// Execute get_tool tool
pub async fn execute_get_tool(input: GetToolInput) -> Result<ToolOutput, anyhow::Error> {
    let all_tools = get_tools_async().await;
    
    let tool = all_tools
        .into_iter()
        .find(|t| t.name == input.name);

    match tool {
        Some(t) => Ok(ToolOutput::success(serde_json::json!({
            "found": true,
            "tool": {
                "name": t.name,
                "description": t.description,
                "input_schema": t.input_schema
            }
        }))),
        None => Ok(ToolOutput::success(serde_json::json!({
            "found": false,
            "tool": serde_json::Value::Null,
            "error": format!("Tool '{}' not found", input.name)
        }))),
    }
}

/// Execute connect_mcp_server tool - connect to an external MCP server
pub async fn execute_connect_mcp_server(input: ConnectMcpServerInput) -> Result<ToolOutput, anyhow::Error> {
    let client = match get_mcp_client() {
        Some(c) => c,
        None => return Ok(ToolOutput::success(serde_json::json!({
            "success": false,
            "error": "MCP client not initialized"
        }))),
    };

    let args_vec = input.args.unwrap_or_default();
    let args: Vec<&str> = args_vec.iter().map(|s| s.as_str()).collect();

    match client.connect(&input.name, &input.command, &args).await {
        Ok(()) => Ok(ToolOutput::success(serde_json::json!({
            "success": true,
            "server": input.name,
            "tools": client.list_all_tools().await.len()
        }))),
        Err(e) => Ok(ToolOutput::success(serde_json::json!({
            "success": false,
            "error": e.to_string()
        }))),
    }
}

/// Execute call_mcp_tool tool - call a tool on a connected MCP server
pub async fn execute_call_mcp_tool(input: CallMcpToolInput) -> Result<ToolOutput, anyhow::Error> {
    let client = match get_mcp_client() {
        Some(c) => c,
        None => return Ok(ToolOutput::success(serde_json::json!({
            "success": false,
            "error": "MCP client not initialized"
        }))),
    };

    // Parse arguments string as JSON if provided
    let arguments = match input.arguments {
        Some(args_str) => {
            match serde_json::from_str(&args_str) {
                Ok(v) => Some(v),
                Err(e) => return Ok(ToolOutput::success(serde_json::json!({
                    "success": false,
                    "error": format!("Invalid JSON in arguments: {}", e)
                }))),
            }
        }
        None => None,
    };

    match client.call_tool(&input.tool_name, arguments).await {
        Ok(result) => Ok(ToolOutput::success(serde_json::json!({
            "success": true,
            "result": result
        }))),
        Err(e) => Ok(ToolOutput::success(serde_json::json!({
            "success": false,
            "error": e.to_string()
        }))),
    }
}
