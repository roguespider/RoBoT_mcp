// src/bridge/tools/memory.rs
// Memory-related MCP tools

use std::sync::Arc;

use anyhow::Result;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::database::models::{MemoryCard, MemoryType};
use crate::database::queries;
use crate::database::sqlite::SqliteDatabase;

/// Tool: Store a new memory
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct StoreMemoryInput {
    pub content: String,
    pub memory_type: String,
    pub confidence: Option<f32>,
    pub importance: Option<f32>,
    pub tags: Option<Vec<String>>,
}

/// Tool: Search memories
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct SearchMemoryInput {
    pub query: String,
    pub limit: Option<usize>,
}

/// Tool: Get a specific memory
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct GetMemoryInput {
    pub id: String,
}

/// Tool: List recent memories
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct ListMemoriesInput {
    pub memory_type: Option<String>,
    pub limit: Option<usize>,
}

/// Memory tool definitions
pub mod definitions {
    pub const STORE_MEMORY: &str = "store_memory";
    pub const SEARCH_MEMORY: &str = "search_memory";
    pub const GET_MEMORY: &str = "get_memory";
    pub const LIST_MEMORIES: &str = "list_memories";
    
    pub fn all() -> Vec<crate::bridge::mcp::McpTool> {
        vec![
            crate::bridge::mcp::McpTool {
                name: STORE_MEMORY.to_string(),
                description: "Store a new memory in the knowledge base".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "content": {
                            "type": "string",
                            "description": "The content to store"
                        },
                        "memory_type": {
                            "type": "string",
                            "description": "Type of memory: note, fact, task, file, conversation, code, decision, event",
                            "enum": ["note", "fact", "task", "file", "conversation", "code", "decision", "event"]
                        },
                        "confidence": {
                            "type": "number",
                            "description": "Confidence level (0.0 - 1.0)",
                            "minimum": 0.0,
                            "maximum": 1.0
                        },
                        "importance": {
                            "type": "number",
                            "description": "Importance level (0.0 - 1.0)",
                            "minimum": 0.0,
                            "maximum": 1.0
                        },
                        "tags": {
                            "type": "array",
                            "items": { "type": "string" },
                            "description": "Optional tags for categorization"
                        }
                    },
                    "required": ["content", "memory_type"]
                }),
            },
            crate::bridge::mcp::McpTool {
                name: SEARCH_MEMORY.to_string(),
                description: "Search memories by content".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "query": {
                            "type": "string",
                            "description": "Search query"
                        },
                        "limit": {
                            "type": "number",
                            "description": "Maximum number of results",
                            "default": 10
                        }
                    },
                    "required": ["query"]
                }),
            },
            crate::bridge::mcp::McpTool {
                name: GET_MEMORY.to_string(),
                description: "Get a specific memory by ID".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "id": {
                            "type": "string",
                            "description": "Memory UUID"
                        }
                    },
                    "required": ["id"]
                }),
            },
            crate::bridge::mcp::McpTool {
                name: LIST_MEMORIES.to_string(),
                description: "List recent memories".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "memory_type": {
                            "type": "string",
                            "description": "Filter by memory type"
                        },
                        "limit": {
                            "type": "number",
                            "description": "Maximum number of results",
                            "default": 20
                        }
                    }
                }),
            },
        ]
    }
}

fn parse_memory_type(s: &str) -> MemoryType {
    match s.to_lowercase().as_str() {
        "fact" => MemoryType::Fact,
        "task" => MemoryType::Task,
        "file" => MemoryType::File,
        "conversation" => MemoryType::Conversation,
        "code" => MemoryType::Code,
        "decision" => MemoryType::Decision,
        "event" => MemoryType::Event,
        "encounter" => MemoryType::Encounter,
        "experience" => MemoryType::Experience,
        _ => MemoryType::Note,
    }
}

/// Execute store memory tool
pub async fn execute_store_memory(
    input: StoreMemoryInput,
    database: &Arc<SqliteDatabase>,
) -> Result<serde_json::Value> {
    let memory = MemoryCard {
        id: Uuid::new_v4(),
        content: input.content,
        memory_type: parse_memory_type(&input.memory_type),
        confidence: input.confidence.unwrap_or(0.5),
        importance: input.importance.unwrap_or(0.5),
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };

    let conn = database.connection()?;
    queries::insert_memory(&conn, &memory)?;

    Ok(serde_json::json!({
        "success": true,
        "message": "Memory stored successfully",
        "id": memory.id.to_string()
    }))
}

/// Execute search memory tool
pub async fn execute_search_memory(
    input: SearchMemoryInput,
    database: &Arc<SqliteDatabase>,
) -> Result<serde_json::Value> {
    let limit = input.limit.unwrap_or(10);
    let conn = database.connection()?;
    let results = queries::search_memory(&conn, &input.query, limit)?;

    let memories: Vec<serde_json::Value> = results
        .into_iter()
        .map(|m| {
            serde_json::json!({
                "id": m.id.to_string(),
                "content": m.content,
                "memory_type": m.memory_type.to_string(),
                "confidence": m.confidence,
                "importance": m.importance,
                "created_at": m.created_at.to_rfc3339(),
                "updated_at": m.updated_at.to_rfc3339()
            })
        })
        .collect();

    Ok(serde_json::json!({
        "results": memories,
        "count": memories.len()
    }))
}

/// Execute get memory tool
pub async fn execute_get_memory(
    input: GetMemoryInput,
    database: &Arc<SqliteDatabase>,
) -> Result<serde_json::Value> {
    let uuid = Uuid::parse_str(&input.id)
        .map_err(|e| anyhow::anyhow!("Invalid UUID: {}", e))?;
    
    let conn = database.connection()?;
    let memory = queries::get_memory(&conn, uuid)?;

    match memory {
        Some(m) => Ok(serde_json::json!({
            "found": true,
            "memory": {
                "id": m.id.to_string(),
                "content": m.content,
                "memory_type": m.memory_type.to_string(),
                "confidence": m.confidence,
                "importance": m.importance,
                "created_at": m.created_at.to_rfc3339(),
                "updated_at": m.updated_at.to_rfc3339()
            }
        })),
        None => Ok(serde_json::json!({
            "found": false,
            "memory": null
        })),
    }
}

/// Execute list memories tool
pub async fn execute_list_memories(
    input: ListMemoriesInput,
    database: &Arc<SqliteDatabase>,
) -> Result<serde_json::Value> {
    let limit = input.limit.unwrap_or(20);
    let conn = database.connection()?;
    let memories = queries::list_memories(&conn, input.memory_type.as_deref(), limit)?;

    let result: Vec<serde_json::Value> = memories
        .into_iter()
        .map(|m| {
            serde_json::json!({
                "id": m.id.to_string(),
                "content": m.content,
                "memory_type": m.memory_type.to_string(),
                "confidence": m.confidence,
                "importance": m.importance,
                "created_at": m.created_at.to_rfc3339(),
                "updated_at": m.updated_at.to_rfc3339()
            })
        })
        .collect();

    Ok(serde_json::json!({
        "memories": result,
        "count": result.len()
    }))
}
