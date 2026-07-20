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
#[allow(dead_code)]
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
    
    /// Server info
    pub server_info: McpServerInfo,
    
    /// Server capabilities
    pub capabilities: McpCapabilities,
}

impl McpContext {
    pub fn new(
        database: Arc<SqliteDatabase>,
        bus: Arc<ExperienceBus>,
        coordinator: Arc<ExperienceCoordinator>,
        reflection: Arc<ReflectionEngine>,
        evolution: Arc<EvolutionEngine>,
        scheduler: Arc<Scheduler>,
        metrics: Arc<MetricsCollector>,
    ) -> Self {
        Self {
            database,
            bus,
            coordinator,
            reflection,
            evolution,
            scheduler,
            metrics,
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
