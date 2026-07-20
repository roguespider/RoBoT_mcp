// src/bridge/tools/experience.rs
// Experience-related MCP tools

use std::sync::Arc;

use anyhow::Result;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::database::queries;
use crate::database::sqlite::SqliteDatabase;
use crate::experience::coordinator::ExperienceCoordinator;
use crate::experience::types::{Experience, ExperienceContext, ExperienceOutcome, ExperienceType};

/// Tool: Record an experience
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct RecordExperienceInput {
    pub title: String,
    pub description: String,
    pub experience_type: String,
    pub outcome: String,
    pub context: Option<serde_json::Value>,
}

/// Tool: Get experience statistics
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct GetExperienceStatsInput {
    pub period: Option<String>,
}

/// Tool: List recent experiences
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct ListExperiencesInput {
    pub experience_type: Option<String>,
    pub limit: Option<usize>,
}

/// Tool: Get an experience by ID
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct GetExperienceInput {
    pub id: String,
}

/// Experience tool definitions
pub mod definitions {
    pub const RECORD_EXPERIENCE: &str = "record_experience";
    pub const GET_EXPERIENCE_STATS: &str = "get_experience_stats";
    pub const LIST_EXPERIENCES: &str = "list_experiences";
    pub const GET_EXPERIENCE: &str = "get_experience";
    
    pub fn all() -> Vec<crate::bridge::mcp::McpTool> {
        vec![
            crate::bridge::mcp::McpTool {
                name: RECORD_EXPERIENCE.to_string(),
                description: "Record a new experience from an action or observation".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "title": {
                            "type": "string",
                            "description": "Brief title for the experience"
                        },
                        "description": {
                            "type": "string",
                            "description": "Detailed description of what happened"
                        },
                        "experience_type": {
                            "type": "string",
                            "description": "Type of experience",
                            "enum": ["tool_execution", "memory_lookup", "memory_store", "workflow", "planning", "exploration", "hypothesis", "reflection", "learning", "conversation", "user_feedback", "error", "system"]
                        },
                        "outcome": {
                            "type": "string",
                            "description": "Outcome of the experience",
                            "enum": ["success", "failure", "partial", "timeout", "interrupted"]
                        },
                        "context": {
                            "type": "object",
                            "description": "Optional context information"
                        }
                    },
                    "required": ["title", "description", "experience_type", "outcome"]
                }),
            },
            crate::bridge::mcp::McpTool {
                name: GET_EXPERIENCE_STATS.to_string(),
                description: "Get statistics about recorded experiences".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "period": {
                            "type": "string",
                            "description": "Time period for stats: day, week, month, all",
                            "enum": ["day", "week", "month", "all"]
                        }
                    }
                }),
            },
            crate::bridge::mcp::McpTool {
                name: LIST_EXPERIENCES.to_string(),
                description: "List recent experiences".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "experience_type": {
                            "type": "string",
                            "description": "Filter by experience type"
                        },
                        "limit": {
                            "type": "number",
                            "description": "Maximum number of results",
                            "default": 20
                        }
                    }
                }),
            },
            crate::bridge::mcp::McpTool {
                name: GET_EXPERIENCE.to_string(),
                description: "Get a specific experience by ID".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "id": {
                            "type": "string",
                            "description": "Experience UUID"
                        }
                    },
                    "required": ["id"]
                }),
            },
        ]
    }
}

fn parse_experience_type(s: &str) -> ExperienceType {
    match s.to_lowercase().as_str() {
        "tool_execution" | "tool" => ExperienceType::ToolExecution,
        "memory_lookup" | "lookup" => ExperienceType::MemoryLookup,
        "memory_store" | "store" => ExperienceType::MemoryStore,
        "workflow" => ExperienceType::Workflow,
        "planning" => ExperienceType::Planning,
        "exploration" => ExperienceType::Exploration,
        "hypothesis" => ExperienceType::Hypothesis,
        "reflection" => ExperienceType::Reflection,
        "learning" => ExperienceType::Learning,
        "conversation" => ExperienceType::Conversation,
        "user_feedback" | "feedback" => ExperienceType::UserFeedback,
        "error" => ExperienceType::Error,
        "system" => ExperienceType::System,
        _ => ExperienceType::Custom(s.to_string()),
    }
}

fn parse_outcome(s: &str) -> ExperienceOutcome {
    match s.to_lowercase().as_str() {
        "success" => ExperienceOutcome::success(),
        "failure" => ExperienceOutcome::failure("Recorded via MCP tool"),
        "partial" => ExperienceOutcome::partial("Partial success"),
        "timeout" => ExperienceOutcome::timeout(),
        "interrupted" => ExperienceOutcome::interrupted(),
        _ => ExperienceOutcome::success(),
    }
}

/// Execute record experience tool
pub async fn execute_record_experience(
    input: RecordExperienceInput,
    coordinator: &Arc<ExperienceCoordinator>,
    database: &Arc<SqliteDatabase>,
) -> Result<serde_json::Value> {
    let experience = Experience {
        id: Uuid::new_v4(),
        timestamp: Utc::now(),
        experience_type: parse_experience_type(&input.experience_type),
        title: input.title.clone(),
        description: input.description.clone(),
        context: ExperienceContext::default(),
        outcome: parse_outcome(&input.outcome),
        score: None,
        encounter_ids: vec![],
        maturity: crate::experience::types::KnowledgeMaturity::Emerging,
        confidence: 0.5,
        lessons: vec![],
        evidence_count: 0,
        tags: vec![],
        metadata: Default::default(),
    };

    // Process through coordinator for scoring
    let processed = coordinator.process(experience.clone());

    // Store in database
    let conn = database.connection()?;
    let memory = crate::database::models::MemoryCard::from_experience(&processed);
    queries::insert_memory(&conn, &memory)?;

    Ok(serde_json::json!({
        "success": true,
        "message": "Experience recorded successfully",
        "id": processed.id.to_string(),
        "title": processed.title
    }))
}

/// Execute get experience stats tool
pub async fn execute_get_experience_stats(
    _input: GetExperienceStatsInput,
    database: &Arc<SqliteDatabase>,
) -> Result<serde_json::Value> {
    let conn = database.connection()?;
    let memories = queries::search_memory(&conn, "Experience:", 1000)?;
    
    let total = memories.len();
    
    // Count by type (simplified - counts all experiences)
    let by_type = serde_json::json!({
        "total": total
    });
    
    // Count by outcome
    let mut success = 0;
    let mut failure = 0;
    for m in &memories {
        if m.content.contains("Success") || m.content.contains("success") {
            success += 1;
        } else {
            failure += 1;
        }
    }
    
    let by_outcome = serde_json::json!({
        "success": success,
        "failure": failure
    });

    Ok(serde_json::json!({
        "total": total,
        "by_type": by_type,
        "by_outcome": by_outcome
    }))
}

/// Execute list experiences tool
pub async fn execute_list_experiences(
    input: ListExperiencesInput,
    database: &Arc<SqliteDatabase>,
) -> Result<serde_json::Value> {
    let limit = input.limit.unwrap_or(20);
    let conn = database.connection()?;
    let memories = queries::search_memory(&conn, "Experience:", limit)?;
    
    let experiences: Vec<serde_json::Value> = memories
        .into_iter()
        .map(|m| {
            serde_json::json!({
                "id": m.id.to_string(),
                "content": m.content,
                "confidence": m.confidence,
                "importance": m.importance,
                "created_at": m.created_at.to_rfc3339()
            })
        })
        .collect();

    Ok(serde_json::json!({
        "experiences": experiences,
        "count": experiences.len()
    }))
}

/// Execute get experience tool
pub async fn execute_get_experience(
    input: GetExperienceInput,
    database: &Arc<SqliteDatabase>,
) -> Result<serde_json::Value> {
    let uuid = Uuid::parse_str(&input.id)
        .map_err(|e| anyhow::anyhow!("Invalid UUID: {}", e))?;
    
    let conn = database.connection()?;
    let memory = queries::get_memory(&conn, uuid)?;

    match memory {
        Some(m) => Ok(serde_json::json!({
            "found": true,
            "experience": {
                "id": m.id.to_string(),
                "content": m.content,
                "confidence": m.confidence,
                "importance": m.importance,
                "created_at": m.created_at.to_rfc3339(),
                "updated_at": m.updated_at.to_rfc3339()
            }
        })),
        None => Ok(serde_json::json!({
            "found": false,
            "experience": null
        })),
    }
}
