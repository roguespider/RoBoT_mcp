// src/tools/mod.rs
// MCP tools for Zed Editor integration

#![allow(dead_code)]

use std::sync::{Arc, Mutex};
use serde::{Deserialize, Serialize};

use crate::bridge::mcp::McpContext;

/// Standard output type for MCP tool executions
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct ToolOutput {
    /// Whether the tool execution was successful
    pub success: bool,
    /// The result data from the tool
    pub data: serde_json::Value,
    /// Optional error message if execution failed
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

impl ToolOutput {
    /// Create a successful tool output
    pub fn success(data: serde_json::Value) -> Self {
        Self {
            success: true,
            data,
            error: None,
        }
    }

    /// Create a failed tool output
    pub fn error<E: std::fmt::Display>(msg: E) -> Self {
        Self {
            success: false,
            data: serde_json::Value::Null,
            error: Some(msg.to_string()),
        }
    }

    /// Create a successful output from a value that can be converted to JSON
    pub fn from_value<T: Serialize>(value: T) -> Result<Self, serde_json::Error> {
        Ok(Self::success(serde_json::to_value(value)?))
    }
}

pub mod memory;
pub mod experience;
pub mod reflection;
pub mod search;
pub mod ingestor;
pub mod agent;
pub mod hypothesis;
pub mod knowledge;
pub mod planner;

/// Global tool registry (lazily initialized, using Mutex since only written once at startup)
static TOOL_REGISTRY: std::sync::OnceLock<Arc<Mutex<ToolRegistry>>> = std::sync::OnceLock::new();

/// Tool registry for MCP tools
pub struct ToolRegistry {
    pub tools: Vec<crate::bridge::mcp::McpTool>,
}

impl ToolRegistry {
    pub fn new() -> Self {
        Self { tools: Vec::new() }
    }
}

/// Register all MCP tools with the given context
pub fn register_tools(context: &Arc<McpContext>) {
    let _ = context; // suppress unused warning
    let registry = TOOL_REGISTRY.get_or_init(|| Arc::new(Mutex::new(ToolRegistry::new())));
    
    // Register memory tools
    let tools = memory::definitions::all();
    tracing::info!("Registered {} memory tools", tools.len());
    
    // Register experience tools
    let tools = experience::definitions::all();
    tracing::info!("Registered {} experience tools", tools.len());
    
    // Register reflection tools
    let tools = reflection::definitions::all();
    tracing::info!("Registered {} reflection tools", tools.len());
    
    // Register search tools
    let tools = search::definitions::all();
    tracing::info!("Registered {} search tools", tools.len());
    
    // Register ingestor tools
    let tools = ingestor::definitions::all();
    tracing::info!("Registered {} ingestor tools", tools.len());
    
    // Register agent tools
    let tools = agent::definitions::all();
    tracing::info!("Registered {} agent tools", tools.len());
    
    // Register hypothesis tools
    let tools = hypothesis::definitions::all();
    tracing::info!("Registered {} hypothesis tools", tools.len());
    
    // Register knowledge tools
    let tools = knowledge::definitions::all();
    tracing::info!("Registered {} knowledge tools", tools.len());
    
    // Register planner tools
    let tools = planner::definitions::all();
    tracing::info!("Registered {} planner tools", tools.len());
    
    // Collect all tools
    let all_tools = memory::definitions::all()
        .into_iter()
        .chain(experience::definitions::all())
        .chain(reflection::definitions::all())
        .chain(search::definitions::all())
        .chain(ingestor::definitions::all())
        .chain(agent::definitions::all())
        .chain(hypothesis::definitions::all())
        .chain(knowledge::definitions::all())
        .chain(planner::definitions::all())
        .collect();
    
    // Update registry using mutex lock
    let mut reg = registry.lock().unwrap();
    reg.tools = all_tools;
    tracing::info!("Total MCP tools registered: {}", reg.tools.len());
}

/// Get all registered tools (sync version for use outside async context)
#[allow(dead_code)]
pub fn get_tools() -> Vec<crate::bridge::mcp::McpTool> {
    TOOL_REGISTRY
        .get()
        .map(|r| r.lock().unwrap().tools.clone())
        .unwrap_or_default()
}

/// Get all registered tools (async version for use inside async context)
#[allow(dead_code)]
pub async fn get_tools_async() -> Vec<crate::bridge::mcp::McpTool> {
    // Use blocking lock inside async context (safe since it's only read)
    let registry = TOOL_REGISTRY.get().expect("Tool registry should be initialized by register_tools()");
    let tools = registry.lock().unwrap().tools.clone();
    tools
}

