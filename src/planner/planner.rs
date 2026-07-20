// src/planner/planner.rs
//! Core planning engine for task decomposition and execution

use std::sync::Arc;
use std::collections::HashMap;

use anyhow::Result;
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::experience::metrics::MetricsCollector;

/// A planned task with decomposition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Plan {
    pub id: String,
    pub goal: String,
    pub steps: Vec<PlanStep>,
    pub status: PlanStatus,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub completed_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanStep {
    pub id: String,
    pub description: String,
    pub action: String,
    pub dependencies: Vec<String>,
    pub status: StepStatus,
    pub result: Option<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum PlanStatus {
    Pending,
    InProgress,
    Completed,
    Failed,
    Cancelled,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum StepStatus {
    Pending,
    Blocked,
    Ready,
    InProgress,
    Completed,
    Failed,
    Skipped,
}

/// Core planning engine
pub struct Planner {
    metrics: Arc<MetricsCollector>,
    active_plans: Arc<RwLock<HashMap<String, Plan>>>,
}

impl Planner {
    /// Create a new planner
    pub fn new(metrics: Arc<MetricsCollector>) -> Self {
        Self {
            metrics,
            active_plans: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Create a new plan from a goal
    pub async fn create_plan(&self, goal: impl Into<String>) -> Result<Plan> {
        let plan = Plan {
            id: Uuid::new_v4().to_string(),
            goal: goal.into(),
            steps: Vec::new(),
            status: PlanStatus::Pending,
            created_at: chrono::Utc::now(),
            completed_at: None,
        };

        let mut plans = self.active_plans.write().await;
        plans.insert(plan.id.clone(), plan.clone());

        self.metrics.increment("planner.plans.created").await;

        Ok(plan)
    }

    /// Add a step to a plan
    pub async fn add_step(&self, plan_id: &str, description: impl Into<String>, action: impl Into<String>) -> Result<PlanStep> {
        let step = PlanStep {
            id: Uuid::new_v4().to_string(),
            description: description.into(),
            action: action.into(),
            dependencies: Vec::new(),
            status: StepStatus::Pending,
            result: None,
        };

        let mut plans = self.active_plans.write().await;
        if let Some(plan) = plans.get_mut(plan_id) {
            plan.steps.push(step.clone());
        }

        self.metrics.increment("planner.steps.added").await;

        Ok(step)
    }

    /// Add dependency to a step
    pub async fn add_dependency(&self, plan_id: &str, step_id: &str, depends_on: &str) -> Result<()> {
        let mut plans = self.active_plans.write().await;
        if let Some(plan) = plans.get_mut(plan_id) {
            if let Some(step) = plan.steps.iter_mut().find(|s| s.id == step_id) {
                if !step.dependencies.contains(&depends_on.to_string()) {
                    step.dependencies.push(depends_on.to_string());
                }
            }
        }
        Ok(())
    }

    /// Start executing a plan
    pub async fn start_plan(&self, plan_id: &str) -> Result<()> {
        let mut plans = self.active_plans.write().await;
        if let Some(plan) = plans.get_mut(plan_id) {
            plan.status = PlanStatus::InProgress;
            self.metrics.increment("planner.plans.started").await;
        }
        Ok(())
    }

    /// Complete a step
    pub async fn complete_step(&self, plan_id: &str, step_id: &str, result: Option<String>) -> Result<()> {
        let mut plans = self.active_plans.write().await;
        if let Some(plan) = plans.get_mut(plan_id) {
            if let Some(step) = plan.steps.iter_mut().find(|s| s.id == step_id) {
                step.status = StepStatus::Completed;
                step.result = result;
            }

            // Check if all steps are complete
            let all_complete = plan.steps.iter().all(|s| 
                s.status == StepStatus::Completed || s.status == StepStatus::Skipped
            );
            if all_complete && !plan.steps.is_empty() {
                plan.status = PlanStatus::Completed;
                plan.completed_at = Some(chrono::Utc::now());
                self.metrics.increment("planner.plans.completed").await;
            }
        }
        Ok(())
    }

    /// Fail a step
    pub async fn fail_step(&self, plan_id: &str, step_id: &str, error: String) -> Result<()> {
        let mut plans = self.active_plans.write().await;
        if let Some(plan) = plans.get_mut(plan_id) {
            if let Some(step) = plan.steps.iter_mut().find(|s| s.id == step_id) {
                step.status = StepStatus::Failed;
                step.result = Some(format!("Failed: {}", error));
            }
            plan.status = PlanStatus::Failed;
            self.metrics.increment("planner.plans.failed").await;
        }
        Ok(())
    }

    /// Get a plan by ID
    pub async fn get_plan(&self, plan_id: &str) -> Option<Plan> {
        let plans = self.active_plans.read().await;
        plans.get(plan_id).cloned()
    }

    /// List all active plans
    pub async fn list_plans(&self) -> Vec<Plan> {
        let plans = self.active_plans.read().await;
        plans.values().cloned().collect()
    }

    /// Cancel a plan
    pub async fn cancel_plan(&self, plan_id: &str) -> Result<()> {
        let mut plans = self.active_plans.write().await;
        if let Some(plan) = plans.get_mut(plan_id) {
            plan.status = PlanStatus::Cancelled;
        }
        Ok(())
    }

    /// Clean up completed/failed plans older than a duration
    pub async fn cleanup_old_plans(&self, max_age: chrono::Duration) -> Result<usize> {
        let cutoff = chrono::Utc::now() - max_age;
        let mut plans = self.active_plans.write().await;
        let initial_count = plans.len();

        plans.retain(|_, plan| {
            if let Some(completed) = plan.completed_at {
                completed > cutoff
            } else {
                plan.created_at > cutoff || plan.status == PlanStatus::InProgress
            }
        });

        Ok(initial_count - plans.len())
    }
}

impl Default for Planner {
    fn default() -> Self {
        Self {
            metrics: Arc::new(MetricsCollector::new()),
            active_plans: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}
