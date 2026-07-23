// src/tools/knowledge.rs
//! Knowledge system MCP tools

use std::sync::Arc;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::knowledge::{
    KnowledgeItem, KnowledgeQuery, KnowledgeResult, KnowledgeStore, 
    apply_query, rank_items, 
    types::{KnowledgeConfidence, KnowledgeSource, KnowledgeStatus, KnowledgeType},
};
use crate::tools::ToolOutput;

/// Tool: Add new knowledge
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct AddKnowledgeInput {
    /// The knowledge statement
    pub statement: String,
    /// Type of knowledge
    pub knowledge_type: Option<String>,
    /// Initial confidence (0.0 - 1.0)
    pub confidence: Option<f32>,
    /// Tags for categorization
    pub tags: Option<Vec<String>>,
    /// Source of this knowledge
    pub source: Option<String>,
}

/// Tool: Query knowledge
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct QueryKnowledgeInput {
    /// Text search
    pub query: String,
    /// Filter by type
    pub knowledge_type: Option<String>,
    /// Minimum confidence
    pub min_confidence: Option<f32>,
    /// Only mature knowledge
    pub mature_only: Option<bool>,
    /// Maximum results
    pub limit: Option<usize>,
}

/// Tool: Record knowledge application
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct RecordKnowledgeApplicationInput {
    /// Knowledge ID that was applied
    pub knowledge_id: String,
    /// Whether application was successful
    pub success: bool,
}

/// Tool: Get knowledge statistics
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct GetKnowledgeStatsInput {
    // No parameters needed
}

/// Tool: Get mature knowledge
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct GetMatureKnowledgeInput {
    pub limit: Option<usize>,
}

/// Knowledge tool definitions
pub mod definitions {
    pub const ADD_KNOWLEDGE: &str = "add_knowledge";
    pub const QUERY_KNOWLEDGE: &str = "query_knowledge";
    pub const RECORD_KNOWLEDGE_APPLICATION: &str = "record_knowledge_application";
    pub const GET_KNOWLEDGE_STATS: &str = "get_knowledge_stats";
    pub const GET_MATURE_KNOWLEDGE: &str = "get_mature_knowledge";
    
    pub fn all() -> Vec<crate::bridge::mcp::McpTool> {
        vec![
            crate::bridge::mcp::McpTool {
                name: ADD_KNOWLEDGE.to_string(),
                description: "Add new validated knowledge to the knowledge base".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "statement": {
                            "type": "string",
                            "description": "The knowledge statement to add"
                        },
                        "knowledge_type": {
                            "type": "string",
                            "description": "Type: fact, procedure, causality, pattern, insight, rule, concept",
                            "enum": ["fact", "procedure", "causality", "pattern", "insight", "rule", "concept"]
                        },
                        "confidence": {
                            "type": "number",
                            "description": "Initial confidence (0.0 - 1.0)"
                        },
                        "tags": {
                            "type": "array",
                            "items": { "type": "string" },
                            "description": "Tags for categorization"
                        },
                        "source": {
                            "type": "string",
                            "description": "Source: user, tool, planner, reflection, hypothesis, exploration, external"
                        }
                    },
                    "required": ["statement"]
                }),
            },
            crate::bridge::mcp::McpTool {
                name: QUERY_KNOWLEDGE.to_string(),
                description: "Query the knowledge base for relevant knowledge".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "query": {
                            "type": "string",
                            "description": "Search query"
                        },
                        "knowledge_type": {
                            "type": "string",
                            "description": "Filter by type"
                        },
                        "min_confidence": {
                            "type": "number",
                            "description": "Minimum confidence threshold"
                        },
                        "mature_only": {
                            "type": "boolean",
                            "description": "Only return mature (high confidence) knowledge"
                        },
                        "limit": {
                            "type": "number",
                            "description": "Maximum results to return"
                        }
                    },
                    "required": ["query"]
                }),
            },
            crate::bridge::mcp::McpTool {
                name: RECORD_KNOWLEDGE_APPLICATION.to_string(),
                description: "Record the result of applying knowledge".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "knowledge_id": {
                            "type": "string",
                            "description": "ID of the knowledge that was applied"
                        },
                        "success": {
                            "type": "boolean",
                            "description": "Whether the application was successful"
                        }
                    },
                    "required": ["knowledge_id", "success"]
                }),
            },
            crate::bridge::mcp::McpTool {
                name: GET_KNOWLEDGE_STATS.to_string(),
                description: "Get statistics about the knowledge base".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {}
                }),
            },
            crate::bridge::mcp::McpTool {
                name: GET_MATURE_KNOWLEDGE.to_string(),
                description: "Get all mature (high-confidence) knowledge".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "limit": {
                            "type": "number",
                            "description": "Maximum results to return"
                        }
                    }
                }),
            },
        ]
    }
}

/// Execute add knowledge tool
pub async fn execute_add_knowledge(
    input: AddKnowledgeInput,
    knowledge: &Arc<KnowledgeStore>,
) -> ToolOutput {
    let knowledge_type = match input.knowledge_type.as_deref() {
        Some("fact") => KnowledgeType::Fact,
        Some("procedure") => KnowledgeType::Procedure,
        Some("causality") => KnowledgeType::Causality,
        Some("pattern") => KnowledgeType::Pattern,
        Some("insight") => KnowledgeType::Insight,
        Some("rule") => KnowledgeType::Rule,
        Some("concept") => KnowledgeType::Concept,
        Some(t) => KnowledgeType::Custom(t.to_string()),
        None => KnowledgeType::Insight,
    };
    
    let source = match input.source.as_deref() {
        Some("user") => KnowledgeSource::User,
        Some("tool") => KnowledgeSource::Tool,
        Some("planner") => KnowledgeSource::Planner,
        Some("reflection") => KnowledgeSource::Reflection(Uuid::new_v4()),
        Some("hypothesis") => KnowledgeSource::Hypothesis(Uuid::new_v4()),
        Some("exploration") => KnowledgeSource::Exploration(Uuid::new_v4()),
        Some(s) => KnowledgeSource::External(s.to_string()),
        None => KnowledgeSource::User,
    };
    
    let confidence = input.confidence.unwrap_or(0.5);
    
    let item = KnowledgeItem {
        id: Uuid::new_v4(),
        statement: input.statement,
        knowledge_type,
        confidence: KnowledgeConfidence::new(confidence),
        status: KnowledgeStatus::Active,
        source,
        supporting_evidence: Vec::new(),
        contradicting_evidence: Vec::new(),
        relations: Vec::new(),
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
        success_count: 0,
        failure_count: 0,
        tags: input.tags.unwrap_or_default(),
        metadata: std::collections::HashMap::new(),
    };
    
    let id = knowledge.add(item).await;
    
    ToolOutput::success(serde_json::json!({
        "status": "added",
        "knowledge_id": id.to_string(),
        "message": "Knowledge added successfully"
    }))
}

/// Execute query knowledge tool
pub async fn execute_query_knowledge(
    input: QueryKnowledgeInput,
    knowledge: &Arc<KnowledgeStore>,
) -> ToolOutput {
    let query_text = input.query.clone();
    
    let ktype = input.knowledge_type.as_ref().map(|t| match t.as_str() {
        "fact" => KnowledgeType::Fact,
        "procedure" => KnowledgeType::Procedure,
        "causality" => KnowledgeType::Causality,
        "pattern" => KnowledgeType::Pattern,
        "insight" => KnowledgeType::Insight,
        "rule" => KnowledgeType::Rule,
        "concept" => KnowledgeType::Concept,
        t => KnowledgeType::Custom(t.to_string()),
    });
    
    let query = KnowledgeQuery {
        text: Some(query_text.clone()),
        knowledge_type: ktype,
        status: None,
        min_confidence: input.min_confidence,
        tags: None,
        mature_only: input.mature_only.unwrap_or(false),
        include_related: false,
        limit: input.limit,
    };
    
    let all_items = knowledge.get_all().await;
    let filtered = apply_query(&all_items, &query);
    let ranked = rank_items(filtered, &query);
    
    let result = KnowledgeResult::new(ranked, query.clone());
    
    let items_json: Vec<serde_json::Value> = result.items.iter().map(|item| {
        serde_json::json!({
            "id": item.id.to_string(),
            "statement": item.statement,
            "type": format!("{:?}", item.knowledge_type),
            "confidence": item.overall_confidence(),
            "status": format!("{:?}", item.status),
            "tags": item.tags,
            "success_count": item.success_count,
            "failure_count": item.failure_count,
        })
    }).collect();
    
    ToolOutput::success(serde_json::json!({
        "items": items_json,
        "total_matches": result.total_matches,
        "returned": result.items.len(),
        "query": query_text,
    }))
}

/// Execute record knowledge application tool
pub async fn execute_record_knowledge_application(
    input: RecordKnowledgeApplicationInput,
    knowledge: &Arc<KnowledgeStore>,
) -> ToolOutput {
    let id = match Uuid::parse_str(&input.knowledge_id) {
        Ok(id) => id,
        Err(_) => return ToolOutput::error("Invalid knowledge ID format"),
    };
    
    let success = if input.success {
        knowledge.record_success(id).await
    } else {
        knowledge.record_failure(id).await
    };
    
    if success {
        ToolOutput::success(serde_json::json!({
            "status": "recorded",
            "knowledge_id": input.knowledge_id,
            "result": if input.success { "success_recorded" } else { "failure_recorded" },
        }))
    } else {
        ToolOutput::error(format!("Knowledge item {} not found", input.knowledge_id))
    }
}

/// Execute get knowledge stats tool
pub async fn execute_get_knowledge_stats(
    _input: GetKnowledgeStatsInput,
    knowledge: &Arc<KnowledgeStore>,
) -> ToolOutput {
    let stats = knowledge.stats().await;
    
    ToolOutput::success(serde_json::json!({
        "total": stats.total,
        "active": stats.active,
        "mature": stats.mature,
        "needs_review": stats.needs_review,
        "average_confidence": stats.average_confidence,
    }))
}

/// Execute get mature knowledge tool
pub async fn execute_get_mature_knowledge(
    input: GetMatureKnowledgeInput,
    knowledge: &Arc<KnowledgeStore>,
) -> ToolOutput {
    let mut mature = knowledge.get_mature().await;
    
    if let Some(l) = input.limit {
        mature.truncate(l);
    }
    
    let items_json: Vec<serde_json::Value> = mature.iter().map(|item| {
        serde_json::json!({
            "id": item.id.to_string(),
            "statement": item.statement,
            "type": format!("{:?}", item.knowledge_type),
            "confidence": item.overall_confidence(),
            "tags": item.tags,
            "success_count": item.success_count,
        })
    }).collect();
    
    ToolOutput::success(serde_json::json!({
        "items": items_json,
        "count": items_json.len(),
    }))
}
