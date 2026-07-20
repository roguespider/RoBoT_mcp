// src/tools/mod.rs
// MCP tools for Zed Editor integration

#![allow(dead_code)]

use std::sync::Arc;
use tokio::sync::RwLock;

use crate::bridge::mcp::McpContext;

pub mod memory;
pub mod experience;
pub mod reflection;
pub mod search;
pub mod ingestor;

/// Global tool registry (lazily initialized)
static TOOL_REGISTRY: std::sync::OnceLock<Arc<RwLock<ToolRegistry>>> = std::sync::OnceLock::new();

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
    let registry = TOOL_REGISTRY.get_or_init(|| Arc::new(RwLock::new(ToolRegistry::new())));
    
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
    
    // Collect all tools
    let all_tools = memory::definitions::all()
        .into_iter()
        .chain(experience::definitions::all())
        .chain(reflection::definitions::all())
        .chain(search::definitions::all())
        .chain(ingestor::definitions::all())
        .collect();
    
    // Update registry using blocking write (safe since we have the OnceLock guard)
    if let Ok(mut reg) = registry.try_write() {
        reg.tools = all_tools;
        tracing::info!("Total MCP tools registered: {}", reg.tools.len());
    } else {
        tracing::warn!("Could not acquire write lock on tool registry");
    }
}

/// Get all registered tools
#[allow(dead_code)]
pub fn get_tools() -> Vec<crate::bridge::mcp::McpTool> {
    TOOL_REGISTRY
        .get()
        .map(|r| r.blocking_read().tools.clone())
        .unwrap_or_default()
}

