// src/workflows/engine.rs
//! Workflow execution engine
#![allow(dead_code)]

use std::collections::HashMap;
use std::collections::HashSet;
use std::sync::Arc;

use anyhow::Result;
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::experience::metrics::MetricsCollector;
use crate::experience::types::OutcomeKind;
use crate::tools::{self, ToolOutput};

/// Parse a string to OutcomeKind enum
fn parse_outcome_kind(s: &str) -> OutcomeKind {
    match s.to_lowercase().as_str() {
        "success" => OutcomeKind::Success,
        "failure" => OutcomeKind::Failure,
        "partial" => OutcomeKind::Partial,
        "timeout" => OutcomeKind::Timeout,
        "interrupted" => OutcomeKind::Interrupted,
        _ => OutcomeKind::Success,
    }
}

/// Actions that should skip memory read (already do their own lookup)
const SKIP_MEMORY_READ: &[&str] = &[
    "search_memory",
    "list_memories",
    "get_memory",
    "get_experience",
    "list_experiences",
    "get_experience_stats",
];

/// A workflow definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Workflow {
    pub id: String,
    pub name: String,
    pub description: String,
    pub steps: Vec<WorkflowStep>,
    pub variables: HashMap<String, String>,
    pub status: WorkflowStatus,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub started_at: Option<chrono::DateTime<chrono::Utc>>,
    pub completed_at: Option<chrono::DateTime<chrono::Utc>>,
}

/// A single step in a workflow
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowStep {
    pub id: String,
    pub name: String,
    pub action: String,
    pub parameters: HashMap<String, String>,
    pub retry_count: u32,
    pub max_retries: u32,
    pub timeout_seconds: u64,
    pub on_success: Option<String>,
    pub on_failure: Option<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum WorkflowStatus {
    Draft,
    Ready,
    Running,
    Paused,
    Completed,
    Failed,
    Cancelled,
}

/// Workflow execution engine
pub struct WorkflowEngine {
    metrics: Arc<MetricsCollector>,
    workflows: Arc<RwLock<HashMap<String, Workflow>>>,
    executing: Arc<RwLock<HashSet<String>>>,
    database: Option<Arc<crate::database::sqlite::SqliteDatabase>>,
}

impl WorkflowEngine {
    /// Create a new workflow engine
    pub fn new(metrics: Arc<MetricsCollector>) -> Self {
        Self {
            metrics,
            workflows: Arc::new(RwLock::new(HashMap::new())),
            executing: Arc::new(RwLock::new(HashSet::new())),
            database: None,
        }
    }

    /// Create a new workflow engine with database access
    pub fn with_database(
        metrics: Arc<MetricsCollector>,
        database: Arc<crate::database::sqlite::SqliteDatabase>,
    ) -> Self {
        Self {
            metrics,
            workflows: Arc::new(RwLock::new(HashMap::new())),
            executing: Arc::new(RwLock::new(HashSet::new())),
            database: Some(database),
        }
    }

    /// Create a new workflow definition
    pub async fn create_workflow(
        &self,
        name: impl Into<String>,
        description: impl Into<String>,
    ) -> Workflow {
        let workflow = Workflow {
            id: Uuid::new_v4().to_string(),
            name: name.into(),
            description: description.into(),
            steps: Vec::new(),
            variables: HashMap::new(),
            status: WorkflowStatus::Draft,
            created_at: chrono::Utc::now(),
            started_at: None,
            completed_at: None,
        };

        let mut workflows = self.workflows.write().await;
        workflows.insert(workflow.id.clone(), workflow.clone());

        self.metrics.increment("workflows.created").await;

        workflow
    }

    /// Add a step to a workflow
    pub async fn add_step(
        &self,
        workflow_id: &str,
        name: impl Into<String>,
        action: impl Into<String>,
    ) -> Result<Option<WorkflowStep>> {
        let mut workflows = self.workflows.write().await;

        if let Some(workflow) = workflows.get_mut(workflow_id) {
            let step = WorkflowStep {
                id: Uuid::new_v4().to_string(),
                name: name.into(),
                action: action.into(),
                parameters: HashMap::new(),
                retry_count: 0,
                max_retries: 3,
                timeout_seconds: 300,
                on_success: None,
                on_failure: None,
            };

            workflow.steps.push(step.clone());
            self.metrics.increment("workflows.steps.added").await;

            return Ok(Some(step));
        }

        Ok(None)
    }

    /// Set workflow variable
    pub async fn set_variable(
        &self,
        workflow_id: &str,
        key: impl Into<String>,
        value: impl Into<String>,
    ) -> Result<()> {
        let mut workflows = self.workflows.write().await;

        if let Some(workflow) = workflows.get_mut(workflow_id) {
            workflow.variables.insert(key.into(), value.into());
        }

        Ok(())
    }

    /// Get a workflow by ID
    pub async fn get_workflow(&self, workflow_id: &str) -> Option<Workflow> {
        let workflows = self.workflows.read().await;
        workflows.get(workflow_id).cloned()
    }

    /// List all workflows
    pub async fn list_workflows(&self) -> Vec<Workflow> {
        let workflows = self.workflows.read().await;
        workflows.values().cloned().collect()
    }

    /// List workflows by status
    pub async fn list_by_status(&self, status: WorkflowStatus) -> Vec<Workflow> {
        let workflows = self.workflows.read().await;
        workflows
            .values()
            .filter(|w| w.status == status)
            .cloned()
            .collect()
    }

    /// Validate workflow readiness
    pub async fn validate_workflow(&self, workflow_id: &str) -> Result<bool> {
        let workflows = self.workflows.read().await;

        if let Some(workflow) = workflows.get(workflow_id) {
            if workflow.steps.is_empty() {
                return Ok(false);
            }

            // Check step references are valid
            for step in &workflow.steps {
                if let Some(ref on_success) = step.on_success {
                    if !workflow.steps.iter().any(|s| &s.id == on_success) {
                        anyhow::bail!(
                            "Step {} references non-existent success target: {}",
                            step.id,
                            on_success
                        );
                    }
                }
                if let Some(ref on_failure) = step.on_failure {
                    if !workflow.steps.iter().any(|s| &s.id == on_failure) {
                        anyhow::bail!(
                            "Step {} references non-existent failure target: {}",
                            step.id,
                            on_failure
                        );
                    }
                }
            }

            return Ok(true);
        }

        Ok(false)
    }

    /// Start workflow execution
    pub async fn start(&self, workflow_id: &str) -> Result<()> {
        // Check if already executing
        {
            let executing = self.executing.read().await;
            if executing.contains(workflow_id) {
                anyhow::bail!("Workflow {} is already executing", workflow_id);
            }
        }

        let is_valid = self.validate_workflow(workflow_id).await?;
        if !is_valid {
            anyhow::bail!("Workflow {} is not valid", workflow_id);
        }

        // Mark as executing
        {
            let mut executing = self.executing.write().await;
            executing.insert(workflow_id.to_string());
        }

        // Update workflow status
        {
            let mut workflows = self.workflows.write().await;
            if let Some(workflow) = workflows.get_mut(workflow_id) {
                workflow.status = WorkflowStatus::Running;
                workflow.started_at = Some(chrono::Utc::now());
            }
        }

        self.metrics.increment("workflows.started").await;

        // Execute workflow steps asynchronously
        let engine = self.clone();
        let workflow_id_owned = workflow_id.to_string();
        tokio::spawn(async move {
            if let Err(e) = engine.execute_workflow(&workflow_id_owned).await {
                tracing::error!("Workflow {} execution error: {}", workflow_id_owned, e);
            }
        });

        Ok(())
    }

    /// Execute workflow steps
    async fn execute_workflow(&self, workflow_id: &str) -> Result<()> {
        let (steps, mut variables) = {
            let workflows = self.workflows.read().await;
            let steps = workflows.get(workflow_id).map(|w| w.steps.clone());
            let vars = workflows
                .get(workflow_id)
                .map(|w| w.variables.clone())
                .unwrap_or_default();
            (steps, vars)
        };

        let Some(steps) = steps else {
            return Ok(());
        };

        let mut step_results: HashMap<String, ToolOutput> = HashMap::new();

        for step in &steps {
            tracing::info!(
                "Executing workflow {} step: {} (action: {})",
                workflow_id,
                step.name,
                step.action
            );

            // Replace variables in parameters
            let params = Self::replace_variables(&step.parameters, &variables, &step_results);

            // ============================================================
            // AUTOMATIC MEMORY READ BEFORE ACTION
            // ============================================================
            let memory_context = self.read_memory_before_action(&step.action, &params).await;

            // If we got relevant memories, include them in the context for the action
            // (The action implementation would need to check for context, but we log it)
            if let Some(ref ctx) = memory_context {
                if let Some(memories) = ctx.data.get("memories").and_then(|v| v.as_array()) {
                    if !memories.is_empty() {
                        tracing::info!(
                            "Found {} relevant memories before action '{}'",
                            memories.len(),
                            step.action
                        );
                    }
                }
            }

            // Execute the step action
            let result = self.execute_step_action(&step.action, &params).await;

            match result {
                Ok(output) => {
                    tracing::info!("Step {} completed successfully", step.name);
                    step_results.insert(step.id.clone(), output.clone());
                    self.metrics.increment("workflows.steps.executed").await;

                    // ============================================================
                    // RECORD EXPERIENCE FOR LATER REFLECTION/CURATION
                    // Following architecture #9: Not direct permanent storage
                    // Experiences are recorded for the reflection system to curate
                    // ============================================================
                    self.record_experience_after_action(&step.action, &params, &output)
                        .await;

                    // Handle on_success: store result in variable if specified
                    if let Some(var_name) = &step.on_success {
                        variables.insert(
                            var_name.clone(),
                            serde_json::to_string(&output.data).unwrap_or_default(),
                        );
                    }
                }
                Err(e) => {
                    tracing::error!("Step {} failed: {}", step.name, e);
                    self.metrics.increment("workflows.steps.failed").await;

                    // Update workflow status to failed
                    {
                        let mut workflows = self.workflows.write().await;
                        if let Some(workflow) = workflows.get_mut(workflow_id) {
                            workflow.status = WorkflowStatus::Failed;
                        }
                    }

                    // Remove from executing
                    {
                        let mut executing = self.executing.write().await;
                        executing.remove(workflow_id);
                    }
                    return Err(e);
                }
            }
        }

        // Mark workflow as completed
        {
            let mut workflows = self.workflows.write().await;
            if let Some(workflow) = workflows.get_mut(workflow_id) {
                workflow.status = WorkflowStatus::Completed;
                workflow.completed_at = Some(chrono::Utc::now());
                workflow.variables = variables;
            }
        }

        // Remove from executing
        {
            let mut executing = self.executing.write().await;
            executing.remove(workflow_id);
        }

        self.metrics.increment("workflows.completed").await;

        Ok(())
    }

    /// Replace variables in parameters with their values
    fn replace_variables(
        params: &HashMap<String, String>,
        workflow_vars: &HashMap<String, String>,
        step_results: &HashMap<String, ToolOutput>,
    ) -> HashMap<String, String> {
        let mut resolved = params.clone();
        for value in resolved.values_mut() {
            // Replace workflow variables ${var_name}
            for (var_name, var_value) in workflow_vars {
                let placeholder = format!("${{{}}}", var_name);
                *value = value.replace(&placeholder, var_value);
            }
            // Replace step result references ${step_id.output_field}
            for (step_id, result) in step_results {
                // Use the data field from ToolOutput
                if let Some(obj) = result.data.as_object() {
                    for (field, field_value) in obj {
                        let placeholder = format!("${{{}.{}}}", step_id, field);
                        *value = value.replace(&placeholder, &field_value.to_string());
                    }
                }
            }
        }
        resolved
    }

    /// Execute a step action by name with actual tool execution
    async fn execute_step_action(
        &self,
        action: &str,
        params: &HashMap<String, String>,
    ) -> Result<ToolOutput> {
        // Helper to get param as string
        let get_param = |key: &str| params.get(key).cloned().unwrap_or_default();

        match action {
            // Memory actions
            "store_memory" => {
                let input = tools::memory::StoreMemoryInput {
                    content: get_param("content"),
                    memory_type: params
                        .get("memory_type")
                        .cloned()
                        .unwrap_or_else(|| "note".to_string()),
                    confidence: params.get("confidence").and_then(|s| s.parse().ok()),
                    importance: params.get("importance").and_then(|s| s.parse().ok()),
                    tags: params
                        .get("tags")
                        .map(|s| s.split(',').map(String::from).collect()),
                };

                if let Some(db) = &self.database {
                    let result = tools::memory::execute_store_memory(input, db).await?;
                    Ok(result)
                } else {
                    Ok(ToolOutput::success(serde_json::json!({
                        "status": "no_database",
                        "message": "Workflow engine created without database access",
                        "action": action
                    })))
                }
            }
            "search_memory" => {
                let input = tools::memory::SearchMemoryInput {
                    query: get_param("query"),
                    limit: params.get("limit").and_then(|s| s.parse().ok()),
                };

                if let Some(db) = &self.database {
                    let result = tools::memory::execute_search_memory(input, db).await?;
                    Ok(result)
                } else {
                    Ok(ToolOutput::success(serde_json::json!({
                        "status": "no_database",
                        "message": "Workflow engine created without database access",
                        "action": action
                    })))
                }
            }
            "list_memories" => {
                let input = tools::memory::ListMemoriesInput {
                    memory_type: params.get("memory_type").cloned(),
                    limit: params.get("limit").and_then(|s| s.parse().ok()),
                };

                if let Some(db) = &self.database {
                    let result = tools::memory::execute_list_memories(input, db).await?;
                    Ok(result)
                } else {
                    Ok(ToolOutput::success(serde_json::json!({
                        "status": "no_database",
                        "message": "Workflow engine created without database access",
                        "action": action
                    })))
                }
            }

            // Experience actions
            "record_experience" => {
                let context_value = params
                    .get("context")
                    .and_then(|s| serde_json::from_str::<serde_json::Value>(s).ok())
                    .map(|v| serde_json::to_string(&v).unwrap_or_default());

                let input = tools::experience::RecordExperienceInput {
                    title: get_param("title"),
                    description: get_param("description"),
                    experience_type: params
                        .get("experience_type")
                        .cloned()
                        .unwrap_or_else(|| "general".to_string()),
                    outcome: parse_outcome_kind(
                        params
                            .get("outcome")
                            .map(|s| s.as_str())
                            .unwrap_or("success"),
                    ),
                    context: context_value,
                };

                if let Some(db) = &self.database {
                    let scorer = crate::experience::scorer::ExperienceScorer::new();
                    let bus = Arc::new(crate::experience::bus::ExperienceBus::new());
                    let coordinator = Arc::new(
                        crate::experience::coordinator::ExperienceCoordinator::new(scorer, bus),
                    );
                    let result =
                        tools::experience::execute_record_experience(input, &coordinator, db)
                            .await?;
                    Ok(result)
                } else {
                    Ok(ToolOutput::success(serde_json::json!({
                        "status": "no_database",
                        "message": "Workflow engine created without database access",
                        "action": action
                    })))
                }
            }

            // Reflection actions
            "create_reflection" => {
                let input = tools::reflection::CreateReflectionInput {
                    title: get_param("title"),
                    description: get_param("description"),
                    reflection_type: params
                        .get("reflection_type")
                        .cloned()
                        .unwrap_or_else(|| "general".to_string()),
                    experience_ids: params
                        .get("experience_ids")
                        .map(|s| s.split(',').map(String::from).collect())
                        .unwrap_or_default(),
                };

                // Need reflection engine - create one if available
                let reflection = Arc::new(crate::experience::reflection::ReflectionEngine::new());
                let result =
                    tools::reflection::execute_create_reflection(input, &reflection).await?;
                Ok(result)
            }

            // Ingestor actions
            "ingest_files" => {
                let input = tools::ingestor::IngestFilesInput {
                    folder: params.get("folder").cloned(),
                    file_path: params.get("file_path").cloned(),
                    limit: params.get("limit").and_then(|s| s.parse().ok()),
                    chunk_size: params.get("chunk_size").and_then(|s| s.parse().ok()),
                    memory_type: params.get("memory_type").cloned(),
                    timeout_seconds: params.get("timeout_seconds").and_then(|s| s.parse().ok()),
                };

                if let Some(db) = &self.database {
                    let result = tools::ingestor::ingest_file(input, Arc::clone(db)).await?;
                    Ok(result)
                } else {
                    Ok(ToolOutput::success(serde_json::json!({
                        "status": "no_database",
                        "message": "Workflow engine created without database access",
                        "action": action
                    })))
                }
            }

            // Generic tool call - returns params as result
            _ => Ok(ToolOutput::success(serde_json::json!({
                "status": "executed",
                "action": action,
                "parameters": params
            }))),
        }
    }

    /// Pause workflow execution
    pub async fn pause(&self, workflow_id: &str) -> Result<()> {
        let mut workflows = self.workflows.write().await;
        if let Some(workflow) = workflows.get_mut(workflow_id) {
            if workflow.status == WorkflowStatus::Running {
                workflow.status = WorkflowStatus::Paused;
                self.metrics.increment("workflows.paused").await;
            }
        }
        Ok(())
    }

    /// Resume paused workflow
    pub async fn resume(&self, workflow_id: &str) -> Result<()> {
        let mut workflows = self.workflows.write().await;
        if let Some(workflow) = workflows.get_mut(workflow_id) {
            if workflow.status == WorkflowStatus::Paused {
                workflow.status = WorkflowStatus::Running;
                self.metrics.increment("workflows.resumed").await;
            }
        }
        Ok(())
    }

    /// Cancel workflow execution
    pub async fn cancel(&self, workflow_id: &str) -> Result<()> {
        // Remove from executing
        {
            let mut executing = self.executing.write().await;
            executing.remove(workflow_id);
        }

        // Update status
        {
            let mut workflows = self.workflows.write().await;
            if let Some(workflow) = workflows.get_mut(workflow_id) {
                workflow.status = WorkflowStatus::Cancelled;
            }
        }

        self.metrics.increment("workflows.cancelled").await;

        Ok(())
    }

    /// Delete workflow
    pub async fn delete(&self, workflow_id: &str) -> Result<()> {
        let mut workflows = self.workflows.write().await;
        workflows.remove(workflow_id);
        Ok(())
    }

    // ============================================================
    // AUTOMATIC MEMORY MIDDLEWARE
    // ============================================================

    /// Check if action should skip memory read
    fn should_skip_memory_read(action: &str) -> bool {
        SKIP_MEMORY_READ.iter().any(|&s| s == action)
    }

    /// Automatically read relevant memories before executing an action
    /// Following architecture #8: Working memory is temporary context for active tasks
    async fn read_memory_before_action(
        &self,
        action: &str,
        params: &HashMap<String, String>,
    ) -> Option<ToolOutput> {
        // Skip for read-only actions
        if Self::should_skip_memory_read(action) {
            return None;
        }

        let db = self.database.as_ref()?;

        // Build a query from the action and parameters
        let query = build_search_query(action, params);

        tracing::info!(
            "[Working Memory] Searching for context before action '{}': '{}'",
            action,
            query
        );

        let input = tools::memory::SearchMemoryInput {
            query: query.clone(),
            limit: Some(5),
        };

        match tools::memory::execute_search_memory(input, db).await {
            Ok(result) => {
                if !result
                    .data
                    .get("memories")
                    .map(|v| v.as_array().map(|a| !a.is_empty()).unwrap_or(false))
                    .unwrap_or(false)
                {
                    tracing::debug!(
                        "[Working Memory] No relevant context found for action '{}'",
                        action
                    );
                } else {
                    let count = result
                        .data
                        .get("memories")
                        .and_then(|v| v.as_array())
                        .map(|a| a.len())
                        .unwrap_or(0);
                    tracing::info!(
                        "[Working Memory] Found {} relevant context items for action '{}'",
                        count,
                        action
                    );
                }
                Some(result)
            }
            Err(e) => {
                tracing::warn!(
                    "[Working Memory] Failed to search context before action '{}': {}",
                    action,
                    e
                );
                None
            }
        }
    }

    /// Record an experience after action completion
    /// Following architecture #9: Information moves to permanent ONLY after evaluation via reflection
    /// This records to working storage for later curation, NOT direct permanent storage
    async fn record_experience_after_action(
        &self,
        action: &str,
        params: &HashMap<String, String>,
        result: &ToolOutput,
    ) {
        // Skip for read-only actions
        if Self::should_skip_memory_read(action) {
            return;
        }

        let db = match &self.database {
            Some(db) => db,
            None => return,
        };

        // Determine outcome based on result
        let outcome_kind = if result.success {
            OutcomeKind::Success
        } else {
            OutcomeKind::Failure
        };

        // Build experience title and description
        let title = build_experience_title(action, params);
        let description = build_experience_description(action, params, result);

        tracing::info!(
            "[Experience] Recording: {} - Outcome: {:?}",
            title,
            outcome_kind
        );

        let input = tools::experience::RecordExperienceInput {
            title,
            description,
            experience_type: map_action_to_experience_type(action),
            outcome: outcome_kind,
            context: None,
        };

        let scorer = crate::experience::scorer::ExperienceScorer::new();
        let bus = Arc::new(crate::experience::bus::ExperienceBus::new());
        let coordinator = Arc::new(crate::experience::coordinator::ExperienceCoordinator::new(
            scorer, bus,
        ));

        match tools::experience::execute_record_experience(input, &coordinator, db).await {
            Ok(_) => {
                tracing::debug!(
                    "[Experience] Recorded for future reflection/curation: action='{}'",
                    action
                );
            }
            Err(e) => {
                tracing::warn!("[Experience] Failed to record: {}", e);
            }
        }
    }
}

/// Build search query from action and parameters
fn build_search_query(action: &str, params: &HashMap<String, String>) -> String {
    let mut parts = vec![action.replace('_', " ")];

    // Add relevant parameters to the query
    for (_key, value) in params.iter() {
        if !value.is_empty() && value.len() < 100 {
            let normalized_value = value.replace(['[', ']', '{', '}'], "").trim().to_string();
            if !normalized_value.is_empty() {
                parts.push(normalized_value);
            }
        }
    }

    // Limit query length
    let query = parts.join(" ");
    if query.len() > 200 {
        query[..200].to_string()
    } else {
        query
    }
}

/// Build experience title from action and parameters
fn build_experience_title(action: &str, params: &HashMap<String, String>) -> String {
    let subject = params
        .get("title")
        .or(params.get("name"))
        .or(params.get("path"))
        .or(params.get("file_path"))
        .or(params.get("command"))
        .cloned()
        .unwrap_or_else(|| action.replace('_', " "));

    format!("Workflow: {}", subject)
}

/// Build experience description following architecture #5, #10
/// Architecture #5: Separate Observation From Interpretation
/// Architecture #10: Reflection asks what happened, why, expected, what changes
#[derive(Debug, Clone)]
pub struct ExperienceRecord {
    /// Raw observation - what actually happened (fact, observable)
    pub observation: String,
    /// Raw result - the objective outcome
    pub outcome: String,
    /// Interpretation placeholder - what this might mean (hypothesis)
    pub interpretation: Option<String>,
    /// Reflection questions
    pub reflection_questions: Vec<String>,
}

impl ExperienceRecord {
    /// Create a new experience record with separated observation/interpretation
    pub fn new(action: &str, params: &HashMap<String, String>, result: &ToolOutput) -> Self {
        // OBSERVATION: Raw, observable facts (architecture #5)
        let observation = Self::build_observation(action, params);

        // OUTCOME: Objective result
        let outcome = Self::build_outcome(result);

        // REFLECTION QUESTIONS (architecture #10)
        let reflection_questions = vec![
            "Why did this happen?".to_string(),
            "Was the outcome expected?".to_string(),
            "What should change?".to_string(),
            "What should be attempted next?".to_string(),
        ];

        Self {
            observation,
            outcome,
            interpretation: None, // Interpretation is added by Reflection system, not here
            reflection_questions,
        }
    }

    /// Build raw observation - observable facts only
    fn build_observation(action: &str, _params: &HashMap<String, String>) -> String {
        match action {
            "create_file" | "write_file" => format!("File operation: create/write"),
            "edit_file" => format!("File operation: edit"),
            "delete_file" => format!("File operation: delete"),
            "run_command" | "execute_command" | "bash" => format!("Command executed"),
            "create_reflection" => format!("Reflection created"),
            "ingest_files" | "import_files" => format!("Files ingested"),
            "record_experience" => format!("Experience recorded"),
            "search_memory" | "get_memory" => format!("Memory accessed"),
            "add_knowledge" => format!("Knowledge added"),
            _ => format!("Action: {}", action),
        }
    }

    /// Build outcome description
    fn build_outcome(result: &ToolOutput) -> String {
        let status = if result.success { "success" } else { "failure" };
        let summary = extract_result_summary(result);
        format!("{}: {}", status, summary)
    }

    /// Convert to description string for storage
    /// Separates observation from interpretation for architecture #5
    pub fn to_description(&self) -> String {
        let mut parts = Vec::new();

        // OBSERVATION (what happened - fact)
        parts.push(format!("[OBSERVATION] {}", self.observation));

        // OUTCOME (what was result)
        parts.push(format!("[OUTCOME] {}", self.outcome));

        // INTERPRETATION (what it might mean - placeholder for Reflection)
        if let Some(ref interp) = self.interpretation {
            parts.push(format!("[INTERPRETATION] {}", interp));
        } else {
            parts.push("[INTERPRETATION] Pending reflection".to_string());
        }

        // REFLECTION QUESTIONS
        parts.push("[REFLECTION QUESTIONS]".to_string());
        for q in &self.reflection_questions {
            parts.push(format!("  - {}", q));
        }

        parts.join("\n")
    }
}

/// Build experience description (legacy compatibility)
/// Now properly separates observation from interpretation per architecture #5
fn build_experience_description(
    action: &str,
    params: &HashMap<String, String>,
    result: &ToolOutput,
) -> String {
    let record = ExperienceRecord::new(action, params, result);
    record.to_description()
}

/// Extract a brief summary from the result
fn extract_result_summary(result: &ToolOutput) -> String {
    result
        .data
        .get("message")
        .or_else(|| result.data.get("summary"))
        .or_else(|| result.data.get("content"))
        .and_then(|v| v.as_str())
        .map(|s| {
            if s.len() > 100 {
                format!("{}...", &s[..100])
            } else {
                s.to_string()
            }
        })
        .unwrap_or_else(|| "No details".to_string())
}

/// Map workflow action names to ExperienceType
fn map_action_to_experience_type(action: &str) -> String {
    match action {
        // File operations
        "create_file" | "write_file" | "edit_file" | "delete_file" => "tool_execution".to_string(),

        // Command execution
        "run_command" | "execute_command" | "bash" => "tool_execution".to_string(),

        // Memory operations
        "store_memory" | "write_memory" => "memory_store".to_string(),
        "search_memory" | "read_memory" | "get_memory" => "memory_lookup".to_string(),

        // Workflow operations
        "create_workflow" | "start_workflow" | "execute_workflow" => "workflow".to_string(),

        // Reflection
        "create_reflection" | "reflect" => "reflection".to_string(),

        // File ingestion
        "ingest_files" | "import_files" => "tool_execution".to_string(),

        // Experience recording
        "record_experience" => "learning".to_string(),

        // Generic fallback
        _ => "system".to_string(),
    }
}

impl Clone for WorkflowEngine {
    fn clone(&self) -> Self {
        Self {
            metrics: self.metrics.clone(),
            workflows: self.workflows.clone(),
            executing: self.executing.clone(),
            database: self.database.clone(),
        }
    }
}

impl Default for WorkflowEngine {
    fn default() -> Self {
        Self {
            metrics: Arc::new(MetricsCollector::new()),
            workflows: Arc::new(RwLock::new(HashMap::new())),
            executing: Arc::new(RwLock::new(HashSet::new())),
            database: None,
        }
    }
}
