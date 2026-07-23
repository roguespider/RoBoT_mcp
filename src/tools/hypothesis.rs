// src/tools/hypothesis.rs
// Hypothesis Engine: Observation -> Hypothesis -> Test -> Evidence -> Knowledge

use std::sync::Arc;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::database::sqlite::SqliteDatabase;
use crate::tools::ToolOutput;

use crate::database::models::{
    Evidence, Hypothesis, HypothesisStatus, Knowledge, Observation,
};

// ============================================================================
// TOOL INPUT/OUTPUT TYPES
// ============================================================================

/// Record an observation
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct RecordObservationInput {
    pub content: String,
    pub context: String,
    pub observation_type: String, // success, failure, pattern, anomaly
}

/// Create a hypothesis
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct CreateHypothesisInput {
    pub statement: String,
    pub domain: String,
    pub source_observations: Vec<String>,
}

/// Add evidence to a hypothesis
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct AddEvidenceInput {
    pub hypothesis_id: String,
    pub content: String,
    pub evidence_type: String, // success, failure, correlation, anomaly
    pub direction: String,     // support, contradict
    pub strength: f32,
}

/// Get hypothesis details
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct GetHypothesisInput {
    pub hypothesis_id: String,
}

/// List hypotheses with optional filter
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct ListHypothesesInput {
    pub domain: Option<String>,
    pub status: Option<String>,
    pub limit: Option<usize>,
}

/// List observations
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct ListObservationsInput {
    pub observation_type: Option<String>,
    pub limit: Option<usize>,
}

/// Get learned knowledge
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct GetKnowledgeInput {
    pub domain: Option<String>,
    pub limit: Option<usize>,
}

/// Evaluate and update hypothesis status
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct EvaluateHypothesisInput {
    pub hypothesis_id: String,
}

/// Convert supported hypothesis to knowledge
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct ExtractKnowledgeInput {
    pub hypothesis_id: String,
    pub knowledge_content: String,
}

// ============================================================================
// TOOL DEFINITIONS
// ============================================================================

pub mod definitions {
    pub const RECORD_OBSERVATION: &str = "record_observation";
    pub const CREATE_HYPOTHESIS: &str = "create_hypothesis";
    pub const ADD_EVIDENCE: &str = "add_evidence";
    pub const GET_HYPOTHESIS: &str = "get_hypothesis";
    pub const LIST_HYPOTHESES: &str = "list_hypotheses";
    pub const LIST_OBSERVATIONS: &str = "list_observations";
    pub const EVALUATE_HYPOTHESIS: &str = "evaluate_hypothesis";
    pub const GET_KNOWLEDGE: &str = "get_knowledge";
    pub const EXTRACT_KNOWLEDGE: &str = "extract_knowledge";

    pub fn all() -> Vec<crate::bridge::mcp::McpTool> {
        vec![
            crate::bridge::mcp::McpTool {
                name: RECORD_OBSERVATION.to_string(),
                description: "Record an observation. Observations are the starting point for learning - record successes, failures, patterns, or anomalies.".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "content": {
                            "type": "string",
                            "description": "What was observed"
                        },
                        "context": {
                            "type": "string",
                            "description": "Context or circumstances of the observation"
                        },
                        "observation_type": {
                            "type": "string",
                            "description": "Type: success, failure, pattern, anomaly"
                        }
                    },
                    "required": ["content", "observation_type"]
                }),
            },
            crate::bridge::mcp::McpTool {
                name: CREATE_HYPOTHESIS.to_string(),
                description: "Create a testable hypothesis from observations. A hypothesis is a statement that can be tested with evidence.".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "statement": {
                            "type": "string",
                            "description": "The hypothesis statement (e.g., 'Using X approach improves Y outcome')"
                        },
                        "domain": {
                            "type": "string",
                            "description": "Domain/category (e.g., workflow, tool, pattern)"
                        },
                        "source_observations": {
                            "type": "array",
                            "items": {"type": "string"},
                            "description": "IDs of observations that led to this hypothesis"
                        }
                    },
                    "required": ["statement", "domain"]
                }),
            },
            crate::bridge::mcp::McpTool {
                name: ADD_EVIDENCE.to_string(),
                description: "Add evidence to a hypothesis. Evidence can support or contradict the hypothesis.".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "hypothesis_id": {
                            "type": "string",
                            "description": "ID of the hypothesis"
                        },
                        "content": {
                            "type": "string",
                            "description": "Description of the evidence"
                        },
                        "evidence_type": {
                            "type": "string",
                            "description": "Type: success, failure, correlation, anomaly"
                        },
                        "direction": {
                            "type": "string",
                            "description": "support or contradict"
                        },
                        "strength": {
                            "type": "number",
                            "description": "Strength of evidence 0.0-1.0"
                        }
                    },
                    "required": ["hypothesis_id", "content", "direction"]
                }),
            },
            crate::bridge::mcp::McpTool {
                name: GET_HYPOTHESIS.to_string(),
                description: "Get details of a specific hypothesis including all its evidence.".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "hypothesis_id": {
                            "type": "string",
                            "description": "ID of the hypothesis"
                        }
                    },
                    "required": ["hypothesis_id"]
                }),
            },
            crate::bridge::mcp::McpTool {
                name: LIST_HYPOTHESES.to_string(),
                description: "List all hypotheses with optional filters.".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "domain": {
                            "type": "string",
                            "description": "Filter by domain"
                        },
                        "status": {
                            "type": "string",
                            "description": "Filter by status: testing, supported, refuted, inconclusive, superseded"
                        },
                        "limit": {
                            "type": "integer",
                            "description": "Max results (default: 10)"
                        }
                    }
                }),
            },
            crate::bridge::mcp::McpTool {
                name: LIST_OBSERVATIONS.to_string(),
                description: "List recorded observations.".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "observation_type": {
                            "type": "string",
                            "description": "Filter by type: success, failure, pattern, anomaly"
                        },
                        "limit": {
                            "type": "integer",
                            "description": "Max results (default: 10)"
                        }
                    }
                }),
            },
            crate::bridge::mcp::McpTool {
                name: EVALUATE_HYPOTHESIS.to_string(),
                description: "Evaluate a hypothesis based on its evidence and update its status. Returns the new status and confidence.".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "hypothesis_id": {
                            "type": "string",
                            "description": "ID of the hypothesis to evaluate"
                        }
                    },
                    "required": ["hypothesis_id"]
                }),
            },
            crate::bridge::mcp::McpTool {
                name: GET_KNOWLEDGE.to_string(),
                description: "Get learned knowledge extracted from validated hypotheses.".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "domain": {
                            "type": "string",
                            "description": "Filter by domain"
                        },
                        "limit": {
                            "type": "integer",
                            "description": "Max results (default: 10)"
                        }
                    }
                }),
            },
            crate::bridge::mcp::McpTool {
                name: EXTRACT_KNOWLEDGE.to_string(),
                description: "Extract knowledge from a validated hypothesis. Converts a supported hypothesis into reusable knowledge.".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "hypothesis_id": {
                            "type": "string",
                            "description": "ID of the supported hypothesis"
                        },
                        "knowledge_content": {
                            "type": "string",
                            "description": "The knowledge to extract (typically derived from hypothesis statement)"
                        }
                    },
                    "required": ["hypothesis_id", "knowledge_content"]
                }),
            },
        ]
    }
}

// ============================================================================
// DATABASE OPERATIONS
// ============================================================================

/// Record an observation in the database
pub async fn record_observation(db: &Arc<SqliteDatabase>, obs: &Observation) -> Result<()> {
    let conn = db.connection()?;
    conn.execute(
        "INSERT INTO observations (id, content, context, observation_type, related_experiences, triggered_hypothesis, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        (
            obs.id.to_string(),
            &obs.content,
            &obs.context,
            &obs.observation_type,
            serde_json::to_string(&obs.related_experiences)?,
            obs.triggered_hypothesis.map(|u| u.to_string()),
            obs.created_at.to_rfc3339(),
        ),
    )?;
    Ok(())
}

/// Get observation by ID
#[allow(dead_code)]
pub async fn get_observation_by_id(db: &Arc<SqliteDatabase>, id: &Uuid) -> Result<Option<Observation>> {
    let conn = db.connection()?;
    let mut stmt = conn.prepare(
        "SELECT id, content, context, observation_type, related_experiences, triggered_hypothesis, created_at
         FROM observations WHERE id = ?1"
    )?;
    
    let result = stmt.query_row([id.to_string()], |row| {
        let id_str: String = row.get(0)?;
        let content: String = row.get(1)?;
        let context: String = row.get(2)?;
        let observation_type: String = row.get(3)?;
        let related_experiences_str: String = row.get(4)?;
        let triggered_hypothesis: Option<String> = row.get(5)?;
        let created_at_str: String = row.get(6)?;
        
        Ok(Observation {
            id: Uuid::parse_str(&id_str).unwrap_or_default(),
            content,
            context,
            observation_type,
            related_experiences: serde_json::from_str(&related_experiences_str).unwrap_or_default(),
            triggered_hypothesis: triggered_hypothesis.and_then(|s| Uuid::parse_str(&s).ok()),
            created_at: chrono::DateTime::parse_from_rfc3339(&created_at_str)
                .map(|dt| dt.with_timezone(&chrono::Utc))
                .unwrap_or_else(|_| chrono::Utc::now()),
        })
    });
    
    match result {
        Ok(obs) => Ok(Some(obs)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(e.into()),
    }
}

/// Create a hypothesis in the database
pub async fn create_hypothesis(db: &Arc<SqliteDatabase>, hyp: &Hypothesis) -> Result<()> {
    let conn = db.connection()?;
    conn.execute(
        "INSERT INTO hypotheses (id, statement, domain, status, confidence, supporting_count, contradicting_count, source_observations, related_memories, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
        (
            hyp.id.to_string(),
            &hyp.statement,
            &hyp.domain,
            hyp.status.to_string(),
            hyp.confidence,
            hyp.supporting_count,
            hyp.contradicting_count,
            serde_json::to_string(&hyp.source_observations)?,
            serde_json::to_string(&hyp.related_memories)?,
            hyp.created_at.to_rfc3339(),
            hyp.updated_at.to_rfc3339(),
        ),
    )?;
    Ok(())
}

/// Get hypothesis by ID
pub async fn get_hypothesis_by_id(db: &Arc<SqliteDatabase>, id: &Uuid) -> Result<Option<Hypothesis>> {
    let conn = db.connection()?;
    let mut stmt = conn.prepare(
        "SELECT id, statement, domain, status, confidence, supporting_count, contradicting_count, source_observations, related_memories, created_at, updated_at
         FROM hypotheses WHERE id = ?1"
    )?;
    
    let result = stmt.query_row([id.to_string()], |row| {
        let id_str: String = row.get(0)?;
        let statement: String = row.get(1)?;
        let domain: String = row.get(2)?;
        let status_str: String = row.get(3)?;
        let confidence: f32 = row.get(4)?;
        let supporting_count: u32 = row.get(5)?;
        let contradicting_count: u32 = row.get(6)?;
        let source_observations_str: String = row.get(7)?;
        let related_memories_str: String = row.get(8)?;
        let created_at_str: String = row.get(9)?;
        let updated_at_str: String = row.get(10)?;
        
        let status = match status_str.as_str() {
            "supported" => HypothesisStatus::Supported,
            "refuted" => HypothesisStatus::Refuted,
            "inconclusive" => HypothesisStatus::Inconclusive,
            "superseded" => HypothesisStatus::Superseded,
            _ => HypothesisStatus::Testing,
        };
        
        Ok(Hypothesis {
            id: Uuid::parse_str(&id_str).unwrap_or_default(),
            statement,
            domain,
            status,
            confidence,
            supporting_count,
            contradicting_count,
            source_observations: serde_json::from_str(&source_observations_str).unwrap_or_default(),
            related_memories: serde_json::from_str(&related_memories_str).unwrap_or_default(),
            created_at: chrono::DateTime::parse_from_rfc3339(&created_at_str)
                .map(|dt| dt.with_timezone(&chrono::Utc))
                .unwrap_or_else(|_| chrono::Utc::now()),
            updated_at: chrono::DateTime::parse_from_rfc3339(&updated_at_str)
                .map(|dt| dt.with_timezone(&chrono::Utc))
                .unwrap_or_else(|_| chrono::Utc::now()),
        })
    });
    
    match result {
        Ok(hyp) => Ok(Some(hyp)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(e.into()),
    }
}

/// Update hypothesis
pub async fn update_hypothesis(db: &Arc<SqliteDatabase>, hyp: &Hypothesis) -> Result<()> {
    let conn = db.connection()?;
    conn.execute(
        "UPDATE hypotheses SET statement = ?2, domain = ?3, status = ?4, confidence = ?5,
         supporting_count = ?6, contradicting_count = ?7, source_observations = ?8,
         related_memories = ?9, updated_at = ?10 WHERE id = ?1",
        (
            hyp.id.to_string(),
            &hyp.statement,
            &hyp.domain,
            hyp.status.to_string(),
            hyp.confidence,
            hyp.supporting_count,
            hyp.contradicting_count,
            serde_json::to_string(&hyp.source_observations)?,
            serde_json::to_string(&hyp.related_memories)?,
            hyp.updated_at.to_rfc3339(),
        ),
    )?;
    Ok(())
}

/// Add evidence to a hypothesis
pub async fn add_evidence(db: &Arc<SqliteDatabase>, evidence: &Evidence) -> Result<()> {
    let conn = db.connection()?;
    conn.execute(
        "INSERT INTO evidence (id, hypothesis_id, content, evidence_type, direction, strength, experience_id, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        (
            evidence.id.to_string(),
            evidence.hypothesis_id.to_string(),
            &evidence.content,
            &evidence.evidence_type,
            &evidence.direction,
            evidence.strength,
            evidence.experience_id.map(|u| u.to_string()),
            evidence.created_at.to_rfc3339(),
        ),
    )?;
    Ok(())
}

/// Get evidence for a hypothesis
pub async fn get_evidence_for_hypothesis(db: &Arc<SqliteDatabase>, hypothesis_id: &Uuid) -> Result<Vec<Evidence>> {
    let conn = db.connection()?;
    let mut stmt = conn.prepare(
        "SELECT id, hypothesis_id, content, evidence_type, direction, strength, experience_id, created_at
         FROM evidence WHERE hypothesis_id = ?1 ORDER BY created_at DESC"
    )?;
    
    let evidence_iter = stmt.query_map([hypothesis_id.to_string()], |row| {
        let id_str: String = row.get(0)?;
        let hypothesis_id_str: String = row.get(1)?;
        let content: String = row.get(2)?;
        let evidence_type: String = row.get(3)?;
        let direction: String = row.get(4)?;
        let strength: f32 = row.get(5)?;
        let experience_id: Option<String> = row.get(6)?;
        let created_at_str: String = row.get(7)?;
        
        Ok(Evidence {
            id: Uuid::parse_str(&id_str).unwrap_or_default(),
            hypothesis_id: Uuid::parse_str(&hypothesis_id_str).unwrap_or_default(),
            content,
            evidence_type,
            direction,
            strength,
            experience_id: experience_id.and_then(|s| Uuid::parse_str(&s).ok()),
            created_at: chrono::DateTime::parse_from_rfc3339(&created_at_str)
                .map(|dt| dt.with_timezone(&chrono::Utc))
                .unwrap_or_else(|_| chrono::Utc::now()),
        })
    })?;
    
    let mut results = Vec::new();
    for evidence in evidence_iter {
        results.push(evidence?);
    }
    Ok(results)
}

/// Create learned knowledge
pub async fn create_knowledge(db: &Arc<SqliteDatabase>, knowledge: &Knowledge) -> Result<()> {
    let conn = db.connection()?;
    conn.execute(
        "INSERT INTO learned_knowledge (id, content, source_hypothesis, confidence, domain, derivation, active, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        (
            knowledge.id.to_string(),
            &knowledge.content,
            knowledge.source_hypothesis.map(|u| u.to_string()),
            knowledge.confidence,
            &knowledge.domain,
            &knowledge.derivation,
            knowledge.active as i32,
            knowledge.created_at.to_rfc3339(),
        ),
    )?;
    Ok(())
}

/// Get knowledge
pub async fn get_knowledge(db: &Arc<SqliteDatabase>, domain: Option<&str>, limit: usize) -> Result<Vec<Knowledge>> {
    let conn = db.connection()?;
    let query = if domain.is_some() {
        "SELECT id, content, source_hypothesis, confidence, domain, derivation, active, created_at
         FROM learned_knowledge WHERE active = 1 AND domain = ?1 ORDER BY confidence DESC LIMIT ?2"
    } else {
        "SELECT id, content, source_hypothesis, confidence, domain, derivation, active, created_at
         FROM learned_knowledge WHERE active = 1 ORDER BY confidence DESC LIMIT ?1"
    };
    
    let mut results = Vec::new();
    
    if let Some(d) = domain {
        let mut stmt = conn.prepare(query)?;
        let iter = stmt.query_map((d, limit as i64), |row| {
            let id_str: String = row.get(0)?;
            let content: String = row.get(1)?;
            let source_hypothesis: Option<String> = row.get(2)?;
            let confidence: f32 = row.get(3)?;
            let domain: String = row.get(4)?;
            let derivation: String = row.get(5)?;
            let active: i32 = row.get(6)?;
            let created_at_str: String = row.get(7)?;
            
            Ok(Knowledge {
                id: Uuid::parse_str(&id_str).unwrap_or_default(),
                content,
                source_hypothesis: source_hypothesis.and_then(|s| Uuid::parse_str(&s).ok()),
                confidence,
                domain,
                derivation,
                active: active != 0,
                created_at: chrono::DateTime::parse_from_rfc3339(&created_at_str)
                    .map(|dt| dt.with_timezone(&chrono::Utc))
                    .unwrap_or_else(|_| chrono::Utc::now()),
            })
        })?;
        for k in iter {
            results.push(k?);
        }
    } else {
        let mut stmt = conn.prepare(query)?;
        let iter = stmt.query_map([limit as i64], |row| {
            let id_str: String = row.get(0)?;
            let content: String = row.get(1)?;
            let source_hypothesis: Option<String> = row.get(2)?;
            let confidence: f32 = row.get(3)?;
            let domain: String = row.get(4)?;
            let derivation: String = row.get(5)?;
            let active: i32 = row.get(6)?;
            let created_at_str: String = row.get(7)?;
            
            Ok(Knowledge {
                id: Uuid::parse_str(&id_str).unwrap_or_default(),
                content,
                source_hypothesis: source_hypothesis.and_then(|s| Uuid::parse_str(&s).ok()),
                confidence,
                domain,
                derivation,
                active: active != 0,
                created_at: chrono::DateTime::parse_from_rfc3339(&created_at_str)
                    .map(|dt| dt.with_timezone(&chrono::Utc))
                    .unwrap_or_else(|_| chrono::Utc::now()),
            })
        })?;
        for k in iter {
            results.push(k?);
        }
    }
    
    Ok(results)
}

// ============================================================================
// TOOL EXECUTIONS
// ============================================================================

pub async fn execute_record_observation(
    input: RecordObservationInput,
    db: &Arc<SqliteDatabase>,
) -> Result<ToolOutput> {
    let observation = Observation::new(
        input.content,
        input.context,
        input.observation_type,
    );
    
    record_observation(db, &observation).await?;
    
    Ok(ToolOutput::success(serde_json::json!({
        "status": "observation_recorded",
        "observation": {
            "id": observation.id.to_string(),
            "content": observation.content,
            "context": observation.context,
            "observation_type": observation.observation_type,
            "created_at": observation.created_at.to_rfc3339()
        },
        "learning_workflow": "Observation recorded. Use create_hypothesis to form a testable hypothesis from this observation."
    })))
}

pub async fn execute_create_hypothesis(
    input: CreateHypothesisInput,
    db: &Arc<SqliteDatabase>,
) -> Result<ToolOutput> {
    let mut hypothesis = Hypothesis::new(input.statement, input.domain);
    hypothesis.source_observations = input.source_observations;
    
    create_hypothesis(db, &hypothesis).await?;
    
    Ok(ToolOutput::success(serde_json::json!({
        "status": "hypothesis_created",
        "hypothesis": {
            "id": hypothesis.id.to_string(),
            "statement": hypothesis.statement,
            "domain": hypothesis.domain,
            "status": hypothesis.status.to_string(),
            "confidence": hypothesis.confidence,
            "created_at": hypothesis.created_at.to_rfc3339()
        },
        "learning_workflow": "Hypothesis created. Use add_evidence to test this hypothesis with supporting or contradicting evidence."
    })))
}

pub async fn execute_add_evidence(
    input: AddEvidenceInput,
    db: &Arc<SqliteDatabase>,
) -> Result<ToolOutput> {
    let hypothesis_id = Uuid::parse_str(&input.hypothesis_id)
        .map_err(|e| anyhow::anyhow!("Invalid hypothesis ID: {}", e))?;
    
    // Get hypothesis to update counts
    let mut hypothesis = get_hypothesis_by_id(db, &hypothesis_id)
        .await?
        .ok_or_else(|| anyhow::anyhow!("Hypothesis not found"))?;
    
    // Create evidence
    let evidence = Evidence::new(
        hypothesis_id,
        input.content,
        input.evidence_type,
        input.direction.clone(),
        input.strength,
    );
    
    add_evidence(db, &evidence).await?;
    
    // Update hypothesis counts
    if input.direction == "support" {
        hypothesis.supporting_count += 1;
    } else if input.direction == "contradict" {
        hypothesis.contradicting_count += 1;
    }
    hypothesis.updated_at = chrono::Utc::now();
    
    // Recalculate confidence
    let total = hypothesis.supporting_count + hypothesis.contradicting_count;
    if total > 0 {
        hypothesis.confidence = hypothesis.supporting_count as f32 / total as f32;
    }
    
    update_hypothesis(db, &hypothesis).await?;
    
    Ok(ToolOutput::success(serde_json::json!({
        "status": "evidence_added",
        "evidence": {
            "id": evidence.id.to_string(),
            "content": evidence.content,
            "direction": evidence.direction,
            "strength": evidence.strength,
            "created_at": evidence.created_at.to_rfc3339()
        },
        "hypothesis_updated": {
            "supporting_count": hypothesis.supporting_count,
            "contradicting_count": hypothesis.contradicting_count,
            "confidence": hypothesis.confidence
        },
        "suggestion": "Use evaluate_hypothesis to determine if there's enough evidence to conclude."
    })))
}

pub async fn execute_get_hypothesis(
    input: GetHypothesisInput,
    db: &Arc<SqliteDatabase>,
) -> Result<ToolOutput> {
    let hypothesis_id = Uuid::parse_str(&input.hypothesis_id)
        .map_err(|e| anyhow::anyhow!("Invalid hypothesis ID: {}", e))?;
    
    let hypothesis = get_hypothesis_by_id(db, &hypothesis_id)
        .await?
        .ok_or_else(|| anyhow::anyhow!("Hypothesis not found"))?;
    
    let evidence = get_evidence_for_hypothesis(db, &hypothesis_id).await?;
    
    Ok(ToolOutput::success(serde_json::json!({
        "hypothesis": {
            "id": hypothesis.id.to_string(),
            "statement": hypothesis.statement,
            "domain": hypothesis.domain,
            "status": hypothesis.status.to_string(),
            "confidence": hypothesis.confidence,
            "supporting_count": hypothesis.supporting_count,
            "contradicting_count": hypothesis.contradicting_count,
            "source_observations": hypothesis.source_observations,
            "created_at": hypothesis.created_at.to_rfc3339(),
            "updated_at": hypothesis.updated_at.to_rfc3339()
        },
        "evidence": evidence.into_iter().map(|e| serde_json::json!({
            "id": e.id.to_string(),
            "content": e.content,
            "evidence_type": e.evidence_type,
            "direction": e.direction,
            "strength": e.strength,
            "created_at": e.created_at.to_rfc3339()
        })).collect::<Vec<_>>()
    })))
}

pub async fn execute_list_hypotheses(
    input: ListHypothesesInput,
    db: &Arc<SqliteDatabase>,
) -> Result<ToolOutput> {
    let conn = db.connection()?;
    let limit = input.limit.unwrap_or(10) as i64;
    
    let mut results = Vec::new();
    
    // Build query based on filters
    match (&input.domain, &input.status) {
        (Some(domain), Some(status)) => {
            let mut stmt = conn.prepare(
                "SELECT id, statement, domain, status, confidence, supporting_count, contradicting_count, created_at, updated_at 
                 FROM hypotheses WHERE domain = ?1 AND status = ?2 ORDER BY updated_at DESC LIMIT ?3"
            )?;
            let iter = stmt.query_map((domain, status, limit), |row| {
                Ok(serde_json::json!({
                    "id": row.get::<_, String>(0)?,
                    "statement": row.get::<_, String>(1)?,
                    "domain": row.get::<_, String>(2)?,
                    "status": row.get::<_, String>(3)?,
                    "confidence": row.get::<_, f32>(4)?,
                    "supporting_count": row.get::<_, u32>(5)?,
                    "contradicting_count": row.get::<_, u32>(6)?,
                    "created_at": row.get::<_, String>(7)?,
                    "updated_at": row.get::<_, String>(8)?
                }))
            })?;
            for h in iter {
                results.push(h?);
            }
        }
        (Some(domain), None) => {
            let mut stmt = conn.prepare(
                "SELECT id, statement, domain, status, confidence, supporting_count, contradicting_count, created_at, updated_at 
                 FROM hypotheses WHERE domain = ?1 ORDER BY updated_at DESC LIMIT ?2"
            )?;
            let iter = stmt.query_map((domain, limit), |row| {
                Ok(serde_json::json!({
                    "id": row.get::<_, String>(0)?,
                    "statement": row.get::<_, String>(1)?,
                    "domain": row.get::<_, String>(2)?,
                    "status": row.get::<_, String>(3)?,
                    "confidence": row.get::<_, f32>(4)?,
                    "supporting_count": row.get::<_, u32>(5)?,
                    "contradicting_count": row.get::<_, u32>(6)?,
                    "created_at": row.get::<_, String>(7)?,
                    "updated_at": row.get::<_, String>(8)?
                }))
            })?;
            for h in iter {
                results.push(h?);
            }
        }
        (None, Some(status)) => {
            let mut stmt = conn.prepare(
                "SELECT id, statement, domain, status, confidence, supporting_count, contradicting_count, created_at, updated_at 
                 FROM hypotheses WHERE status = ?1 ORDER BY updated_at DESC LIMIT ?2"
            )?;
            let iter = stmt.query_map((status, limit), |row| {
                Ok(serde_json::json!({
                    "id": row.get::<_, String>(0)?,
                    "statement": row.get::<_, String>(1)?,
                    "domain": row.get::<_, String>(2)?,
                    "status": row.get::<_, String>(3)?,
                    "confidence": row.get::<_, f32>(4)?,
                    "supporting_count": row.get::<_, u32>(5)?,
                    "contradicting_count": row.get::<_, u32>(6)?,
                    "created_at": row.get::<_, String>(7)?,
                    "updated_at": row.get::<_, String>(8)?
                }))
            })?;
            for h in iter {
                results.push(h?);
            }
        }
        (None, None) => {
            let mut stmt = conn.prepare(
                "SELECT id, statement, domain, status, confidence, supporting_count, contradicting_count, created_at, updated_at 
                 FROM hypotheses ORDER BY updated_at DESC LIMIT ?1"
            )?;
            let iter = stmt.query_map([limit], |row| {
                Ok(serde_json::json!({
                    "id": row.get::<_, String>(0)?,
                    "statement": row.get::<_, String>(1)?,
                    "domain": row.get::<_, String>(2)?,
                    "status": row.get::<_, String>(3)?,
                    "confidence": row.get::<_, f32>(4)?,
                    "supporting_count": row.get::<_, u32>(5)?,
                    "contradicting_count": row.get::<_, u32>(6)?,
                    "created_at": row.get::<_, String>(7)?,
                    "updated_at": row.get::<_, String>(8)?
                }))
            })?;
            for h in iter {
                results.push(h?);
            }
        }
    }
    
    Ok(ToolOutput::success(serde_json::json!({
        "hypotheses": results,
        "count": results.len()
    })))
}

pub async fn execute_list_observations(
    input: ListObservationsInput,
    db: &Arc<SqliteDatabase>,
) -> Result<ToolOutput> {
    let conn = db.connection()?;
    let limit = input.limit.unwrap_or(10) as i64;
    
    let query = if input.observation_type.is_some() {
        "SELECT id, content, context, observation_type, created_at FROM observations WHERE observation_type = ?1 ORDER BY created_at DESC LIMIT ?2"
    } else {
        "SELECT id, content, context, observation_type, created_at FROM observations ORDER BY created_at DESC LIMIT ?1"
    };
    
    let mut results = Vec::new();
    
    if let Some(obs_type) = input.observation_type {
        let mut stmt = conn.prepare(query)?;
        let iter = stmt.query_map((obs_type.as_str(), limit), |row| {
            let id_str: String = row.get(0)?;
            let content: String = row.get(1)?;
            let context: String = row.get(2)?;
            let observation_type: String = row.get(3)?;
            let created_at_str: String = row.get(4)?;
            
            Ok(serde_json::json!({
                "id": id_str,
                "content": content,
                "context": context,
                "observation_type": observation_type,
                "created_at": created_at_str
            }))
        })?;
        for o in iter {
            results.push(o?);
        }
    } else {
        let mut stmt = conn.prepare(query)?;
        let iter = stmt.query_map([limit], |row| {
            let id_str: String = row.get(0)?;
            let content: String = row.get(1)?;
            let context: String = row.get(2)?;
            let observation_type: String = row.get(3)?;
            let created_at_str: String = row.get(4)?;
            
            Ok(serde_json::json!({
                "id": id_str,
                "content": content,
                "context": context,
                "observation_type": observation_type,
                "created_at": created_at_str
            }))
        })?;
        for o in iter {
            results.push(o?);
        }
    }
    
    Ok(ToolOutput::success(serde_json::json!({
        "observations": results,
        "count": results.len()
    })))
}

pub async fn execute_evaluate_hypothesis(
    input: EvaluateHypothesisInput,
    db: &Arc<SqliteDatabase>,
) -> Result<ToolOutput> {
    let hypothesis_id = Uuid::parse_str(&input.hypothesis_id)
        .map_err(|e| anyhow::anyhow!("Invalid hypothesis ID: {}", e))?;
    
    let mut hypothesis = get_hypothesis_by_id(db, &hypothesis_id)
        .await?
        .ok_or_else(|| anyhow::anyhow!("Hypothesis not found"))?;
    
    let evidence = get_evidence_for_hypothesis(db, &hypothesis_id).await?;
    
    // Calculate new status based on evidence
    let supporting_count = evidence.iter().filter(|e| e.direction == "support").count() as u32;
    let contradicting_count = evidence.iter().filter(|e| e.direction == "contradict").count() as u32;
    let total = supporting_count + contradicting_count;
    
    // Update counts
    hypothesis.supporting_count = supporting_count;
    hypothesis.contradicting_count = contradicting_count;
    hypothesis.updated_at = chrono::Utc::now();
    
    // Determine status
    if total >= 3 {
        // Enough evidence to evaluate
        if supporting_count > contradicting_count * 2 {
            hypothesis.status = HypothesisStatus::Supported;
            hypothesis.confidence = supporting_count as f32 / total as f32;
        } else if contradicting_count > supporting_count * 2 {
            hypothesis.status = HypothesisStatus::Refuted;
            hypothesis.confidence = contradicting_count as f32 / total as f32;
        } else {
            hypothesis.status = HypothesisStatus::Inconclusive;
            hypothesis.confidence = 0.5;
        }
    } else {
        hypothesis.status = HypothesisStatus::Testing;
        hypothesis.confidence = if total > 0 {
            supporting_count as f32 / total as f32
        } else {
            0.5
        };
    }
    
    update_hypothesis(db, &hypothesis).await?;
    
    Ok(ToolOutput::success(serde_json::json!({
        "hypothesis_id": hypothesis_id.to_string(),
        "evaluation": {
            "total_evidence": total,
            "supporting_count": supporting_count,
            "contradicting_count": contradicting_count
        },
        "result": {
            "status": hypothesis.status.to_string(),
            "confidence": hypothesis.confidence,
            "updated_at": hypothesis.updated_at.to_rfc3339()
        },
        "workflow": if hypothesis.status == HypothesisStatus::Supported {
            "Hypothesis is supported! Use extract_knowledge to convert this into reusable knowledge."
        } else if hypothesis.status == HypothesisStatus::Refuted {
            "Hypothesis is refuted. This learning is still valuable - it prevents future mistakes."
        } else {
            "Not enough evidence yet. Continue gathering evidence with add_evidence."
        }
    })))
}

pub async fn execute_get_knowledge(
    input: GetKnowledgeInput,
    db: &Arc<SqliteDatabase>,
) -> Result<ToolOutput> {
    let limit = input.limit.unwrap_or(10);
    let knowledge = get_knowledge(db, input.domain.as_deref(), limit).await?;
    
    let count = knowledge.len();
    let knowledge_json: Vec<_> = knowledge.into_iter().map(|k| serde_json::json!({
        "id": k.id.to_string(),
        "content": k.content,
        "domain": k.domain,
        "confidence": k.confidence,
        "derivation": k.derivation,
        "created_at": k.created_at.to_rfc3339()
    })).collect();
    
    Ok(ToolOutput::success(serde_json::json!({
        "knowledge": knowledge_json,
        "count": count
    })))
}

pub async fn execute_extract_knowledge(
    input: ExtractKnowledgeInput,
    db: &Arc<SqliteDatabase>,
) -> Result<ToolOutput> {
    let hypothesis_id = Uuid::parse_str(&input.hypothesis_id)
        .map_err(|e| anyhow::anyhow!("Invalid hypothesis ID: {}", e))?;
    
    let hypothesis = get_hypothesis_by_id(db, &hypothesis_id)
        .await?
        .ok_or_else(|| anyhow::anyhow!("Hypothesis not found"))?;
    
    // Only allow extracting from supported hypotheses
    if hypothesis.status != HypothesisStatus::Supported {
        return Ok(ToolOutput::error(format!(
            "Can only extract knowledge from supported hypotheses. Current status: {}",
            hypothesis.status.to_string()
        )));
    }
    
    // Create knowledge
    let mut knowledge = Knowledge::new(
        input.knowledge_content,
        hypothesis.domain.clone(),
        format!("Extracted from hypothesis: {}", hypothesis.statement),
    );
    knowledge.source_hypothesis = Some(hypothesis_id);
    knowledge.confidence = hypothesis.confidence;
    
    create_knowledge(db, &knowledge).await?;
    
    // Mark hypothesis as superseded (knowledge extracted)
    let mut updated_hypothesis = hypothesis;
    updated_hypothesis.status = HypothesisStatus::Superseded;
    updated_hypothesis.updated_at = chrono::Utc::now();
    update_hypothesis(db, &updated_hypothesis).await?;
    
    Ok(ToolOutput::success(serde_json::json!({
        "status": "knowledge_extracted",
        "knowledge": {
            "id": knowledge.id.to_string(),
            "content": knowledge.content,
            "domain": knowledge.domain,
            "confidence": knowledge.confidence,
            "derivation": knowledge.derivation,
            "created_at": knowledge.created_at.to_rfc3339()
        },
        "hypothesis_status": "superseded",
        "learning_complete": "This knowledge is now available for future decisions. The hypothesis has been superseded by the extracted knowledge."
    })))
}
