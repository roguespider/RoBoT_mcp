// src/workflows/engine.rs
//! Workflow execution engine

use std::sync::Arc;
use std::collections::HashMap;
use std::collections::HashSet;

use anyhow::Result;
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::experience::metrics::MetricsCollector;
use crate::tools::{self, ToolOutput};

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
    pub fn with_database(metrics: Arc<MetricsCollector>, database: Arc<crate::database::sqlite::SqliteDatabase>) -> Self {
        Self {
            metrics,
            workflows: Arc::new(RwLock::new(HashMap::new())),
            executing: Arc::new(RwLock::new(HashSet::new())),
            database: Some(database),
        }
    }

    /// Create a new workflow definition
    pub async fn create_workflow(&self, name: impl Into<String>, description: impl Into<String>) -> Workflow {
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
    pub async fn set_variable(&self, workflow_id: &str, key: impl Into<String>, value: impl Into<String>) -> Result<()> {
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
        workflows.values()
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
                        anyhow::bail!("Step {} references non-existent success target: {}", step.id, on_success);
                    }
                }
                if let Some(ref on_failure) = step.on_failure {
                    if !workflow.steps.iter().any(|s| &s.id == on_failure) {
                        anyhow::bail!("Step {} references non-existent failure target: {}", step.id, on_failure);
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
            let vars = workflows.get(workflow_id).map(|w| w.variables.clone()).unwrap_or_default();
            (steps, vars)
        };

        let Some(steps) = steps else {
            return Ok(());
        };

        let mut step_results: HashMap<String, ToolOutput> = HashMap::new();

        for step in &steps {
            tracing::info!("Executing workflow {} step: {} (action: {})", workflow_id, step.name, step.action);

            // Replace variables in parameters
            let params = Self::replace_variables(&step.parameters, &variables, &step_results);

            // Execute the step action
            let result = self.execute_step_action(&step.action, &params).await;
            
            match result {
                Ok(output) => {
                    tracing::info!("Step {} completed successfully", step.name);
                    step_results.insert(step.id.clone(), output.clone());
                    self.metrics.increment("workflows.steps.executed").await;
                    
                    // Handle on_success: store result in variable if specified
                    if let Some(var_name) = &step.on_success {
                        variables.insert(var_name.clone(), serde_json::to_string(&output.data).unwrap_or_default());
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
    async fn execute_step_action(&self, action: &str, params: &HashMap<String, String>) -> Result<ToolOutput> {
        // Helper to get param as string
        let get_param = |key: &str| params.get(key).cloned().unwrap_or_default();
        
        match action {
            // Memory actions
            "store_memory" => {
                let input = tools::memory::StoreMemoryInput {
                    content: get_param("content"),
                    memory_type: params.get("memory_type").cloned().unwrap_or_else(|| "note".to_string()),
                    confidence: params.get("confidence").and_then(|s| s.parse().ok()),
                    importance: params.get("importance").and_then(|s| s.parse().ok()),
                    tags: params.get("tags").map(|s| s.split(',').map(String::from).collect()),
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
                let context_value = params.get("context")
                    .and_then(|s| serde_json::from_str::<serde_json::Value>(s).ok())
                    .map(|v| serde_json::to_string(&v).unwrap_or_default());

                let input = tools::experience::RecordExperienceInput {
                    title: get_param("title"),
                    description: get_param("description"),
                    experience_type: params.get("experience_type").cloned().unwrap_or_else(|| "general".to_string()),
                    outcome: params.get("outcome").cloned().unwrap_or_else(|| "success".to_string()),
                    context: context_value,
                };
                
                if let Some(db) = &self.database {
                    let scorer = crate::experience::scorer::ExperienceScorer::new();
                    let coordinator = Arc::new(crate::experience::coordinator::ExperienceCoordinator::new(scorer));
                    let result = tools::experience::execute_record_experience(
                        input, 
                        &coordinator,
                        db
                    ).await?;
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
                    reflection_type: params.get("reflection_type").cloned().unwrap_or_else(|| "general".to_string()),
                    experience_ids: params.get("experience_ids").map(|s| s.split(',').map(String::from).collect()).unwrap_or_default(),
                };
                
                // Need reflection engine - create one if available
                let reflection = Arc::new(crate::experience::reflection::ReflectionEngine::new());
                let result = tools::reflection::execute_create_reflection(input, &reflection).await?;
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
            _ => {
                Ok(ToolOutput::success(serde_json::json!({
                    "status": "executed",
                    "action": action,
                    "parameters": params
                })))
            }
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
