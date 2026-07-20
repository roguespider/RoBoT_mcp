// src/bridge/tools/search.rs
// Search-related MCP tools

use std::sync::Arc;

use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::database::queries;
use crate::database::sqlite::SqliteDatabase;

/// Tool: Full-text search across all data
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct GlobalSearchInput {
    pub query: String,
    pub types: Option<Vec<String>>,
    pub limit: Option<usize>,
}

/// Tool: Get recommendations
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct GetRecommendationsInput {
    pub context: Option<String>,
    pub limit: Option<usize>,
}

/// Tool: Get reputation for a target
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct GetReputationInput {
    pub target: String,
}

/// Search tool definitions
pub mod definitions {
    pub const GLOBAL_SEARCH: &str = "global_search";
    pub const GET_RECOMMENDATIONS: &str = "get_recommendations";
    pub const GET_REPUTATION: &str = "get_reputation";
    
    pub fn all() -> Vec<crate::bridge::mcp::McpTool> {
        vec![
            crate::bridge::mcp::McpTool {
                name: GLOBAL_SEARCH.to_string(),
                description: "Search across all memories, experiences, and reflections".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "query": {
                            "type": "string",
                            "description": "Search query"
                        },
                        "types": {
                            "type": "array",
                            "items": { "type": "string" },
                            "description": "Data types to search: memories, experiences, reflections"
                        },
                        "limit": {
                            "type": "number",
                            "description": "Maximum number of results",
                            "default": 20
                        }
                    },
                    "required": ["query"]
                }),
            },
            crate::bridge::mcp::McpTool {
                name: GET_RECOMMENDATIONS.to_string(),
                description: "Get recommendations based on learned patterns".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "context": {
                            "type": "string",
                            "description": "Optional context for recommendations"
                        },
                        "limit": {
                            "type": "number",
                            "description": "Maximum number of recommendations",
                            "default": 5
                        }
                    }
                }),
            },
            crate::bridge::mcp::McpTool {
                name: GET_REPUTATION.to_string(),
                description: "Get reputation score for a target (tool, file, workflow, etc.)".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "target": {
                            "type": "string",
                            "description": "Target identifier"
                        }
                    },
                    "required": ["target"]
                }),
            },
        ]
    }
}

/// Execute global search tool
pub async fn execute_global_search(
    input: GlobalSearchInput,
    database: &Arc<SqliteDatabase>,
) -> Result<serde_json::Value> {
    let limit = input.limit.unwrap_or(20);
    let conn = database.connection()?;
    
    // Search memories
    let memories = queries::search_memory(&conn, &input.query, limit)?;
    
    // Categorize results
    let mut memory_results = Vec::new();
    let mut experience_results = Vec::new();
    
    for m in memories {
        let item = serde_json::json!({
            "id": m.id.to_string(),
            "content": m.content,
            "type": m.memory_type.to_string(),
            "confidence": m.confidence,
            "created_at": m.created_at.to_rfc3339()
        });
        
        if m.memory_type.to_string() == "experience" {
            experience_results.push(item);
        } else {
            memory_results.push(item);
        }
    }
    
    let total = memory_results.len() + experience_results.len();

    Ok(serde_json::json!({
        "results": {
            "memories": memory_results,
            "experiences": experience_results,
            "reflections": []
        },
        "total": total,
        "query": input.query
    }))
}

/// Execute get recommendations tool
pub async fn execute_get_recommendations(
    input: GetRecommendationsInput,
    database: &Arc<SqliteDatabase>,
) -> Result<serde_json::Value> {
    let limit = input.limit.unwrap_or(5);
    let conn = database.connection()?;
    
    // Get recent experiences with high confidence
    let experiences = queries::search_memory(&conn, "Experience:", 100)?;
    
    // Filter high-confidence experiences for recommendations
    let recommendations: Vec<serde_json::Value> = experiences
        .into_iter()
        .filter(|e| e.confidence >= 0.7)
        .take(limit)
        .map(|e| {
            serde_json::json!({
                "type": "experience",
                "id": e.id.to_string(),
                "description": e.content,
                "confidence": e.confidence
            })
        })
        .collect();

    Ok(serde_json::json!({
        "recommendations": recommendations,
        "based_on": input.context.unwrap_or_else(|| "recent_high_confidence_experiences".to_string())
    }))
}

/// Execute get reputation tool
pub async fn execute_get_reputation(
    input: GetReputationInput,
    database: &Arc<SqliteDatabase>,
) -> Result<serde_json::Value> {
    let conn = database.connection()?;
    
    // Search for mentions of the target
    let results = queries::search_memory(&conn, &input.target, 100)?;
    
    // Calculate simple reputation based on mentions
    let total_uses = results.len();
    let high_confidence = results.iter().filter(|r| r.confidence >= 0.7).count();
    let score = if total_uses > 0 {
        high_confidence as f32 / total_uses as f32
    } else {
        0.5
    };

    Ok(serde_json::json!({
        "target": input.target,
        "score": score,
        "success_count": high_confidence,
        "failure_count": total_uses - high_confidence,
        "total_uses": total_uses
    }))
}
