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
}

impl WorkflowEngine {
    /// Create a new workflow engine
    pub fn new(metrics: Arc<MetricsCollector>) -> Self {
        Self {
            metrics,
            workflows: Arc::new(RwLock::new(HashMap::new())),
            executing: Arc::new(RwLock::new(HashSet::new())),
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
        let steps = {
            let workflows = self.workflows.read().await;
            workflows.get(workflow_id).map(|w| w.steps.clone())
        };

        let Some(steps) = steps else {
            return Ok(());
        };

        for step in steps {
            // Execute step (placeholder - actual implementation would call tools)
            tracing::info!("Executing workflow {} step: {}", workflow_id, step.name);

            // Simulate step execution
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

            self.metrics.increment("workflows.steps.executed").await;
        }

        // Mark workflow as completed
        {
            let mut workflows = self.workflows.write().await;
            if let Some(workflow) = workflows.get_mut(workflow_id) {
                workflow.status = WorkflowStatus::Completed;
                workflow.completed_at = Some(chrono::Utc::now());
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
        }
    }
}

impl Default for WorkflowEngine {
    fn default() -> Self {
        Self {
            metrics: Arc::new(MetricsCollector::new()),
            workflows: Arc::new(RwLock::new(HashMap::new())),
            executing: Arc::new(RwLock::new(HashSet::new())),
        }
    }
}
