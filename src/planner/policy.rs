// src/planner/policy.rs
//! Policy engine for decision-making rules

use std::sync::Arc;

use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;

/// A policy rule that can match contexts and provide guidance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyRule {
    pub id: String,
    pub name: String,
    pub description: String,
    pub priority: i32,
    pub condition: PolicyCondition,
    pub action: PolicyAction,
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PolicyCondition {
    Always,
    Never,
    IfConfidenceAbove(f32),
    IfConfidenceBelow(f32),
    IfReputationAbove { target: String, threshold: f32 },
    IfReputationBelow { target: String, threshold: f32 },
    IfExperienceCountAbove { experience_type: String, count: usize },
    IfTaskType(String),
    IfErrorCountAbove(u32),
    Custom(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PolicyAction {
    Allow,
    Deny,
    Prefer(String),
    Avoid(String),
    RequireConfidence(f32),
    ScaleReputation { delta: f32 },
    LogAndAllow,
    LogAndDeny,
    Defer,
}

/// Context for policy evaluation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyContext {
    pub task_type: Option<String>,
    pub task_description: Option<String>,
    pub confidence: f32,
    pub target: Option<String>,
    pub experience_count: usize,
    pub error_count: u32,
    pub is_exploration: bool,
}

/// Policy engine that evaluates rules
pub struct PolicyEngine {
    rules: Arc<RwLock<Vec<PolicyRule>>>,
}

impl PolicyEngine {
    /// Create a new policy engine
    pub fn new() -> Self {
        Self {
            rules: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Add a policy rule
    pub async fn add_rule(&self, rule: PolicyRule) {
        let mut rules = self.rules.write().await;
        rules.push(rule);
        rules.sort_by(|a, b| b.priority.cmp(&a.priority));
    }

    /// Remove a rule by ID
    pub async fn remove_rule(&self, rule_id: &str) {
        let mut rules = self.rules.write().await;
        rules.retain(|r| r.id != rule_id);
    }

    /// Enable a rule
    pub async fn enable_rule(&self, rule_id: &str) {
        let mut rules = self.rules.write().await;
        if let Some(rule) = rules.iter_mut().find(|r| r.id == rule_id) {
            rule.enabled = true;
        }
    }

    /// Disable a rule
    pub async fn disable_rule(&self, rule_id: &str) {
        let mut rules = self.rules.write().await;
        if let Some(rule) = rules.iter_mut().find(|r| r.id == rule_id) {
            rule.enabled = false;
        }
    }

    /// Evaluate context against all rules
    pub async fn evaluate(&self, context: &PolicyContext) -> PolicyResult {
        let rules = self.rules.read().await;

        for rule in rules.iter() {
            if !rule.enabled {
                continue;
            }

            if self.matches_condition(&rule.condition, context).await {
                return PolicyResult::Decision {
                    action: rule.action.clone(),
                    matched_rule: rule.id.clone(),
                    rule_name: rule.name.clone(),
                };
            }
        }

        PolicyResult::NoMatch
    }

    /// Check if a condition matches the context
    async fn matches_condition(&self, condition: &PolicyCondition, context: &PolicyContext) -> bool {
        match condition {
            PolicyCondition::Always => true,
            PolicyCondition::Never => false,
            PolicyCondition::IfConfidenceAbove(threshold) => context.confidence > *threshold,
            PolicyCondition::IfConfidenceBelow(threshold) => context.confidence < *threshold,
            PolicyCondition::IfReputationAbove { target, threshold } => {
                // For now, use context-based reputation check
                context.target.as_ref().map_or(false, |t| t == target) && context.confidence > *threshold
            }
            PolicyCondition::IfReputationBelow { target, threshold } => {
                // For now, use context-based reputation check
                context.target.as_ref().map_or(false, |t| t == target) && context.confidence < *threshold
            }
            PolicyCondition::IfExperienceCountAbove { experience_type: _, count } => {
                context.experience_count > *count
            }
            PolicyCondition::IfTaskType(task_type) => {
                context.task_type.as_ref().map_or(false, |t| t == task_type)
            }
            PolicyCondition::IfErrorCountAbove(threshold) => context.error_count > *threshold,
            PolicyCondition::Custom(_) => false,
        }
    }

    /// List all rules
    pub async fn list_rules(&self) -> Vec<PolicyRule> {
        let rules = self.rules.read().await;
        rules.clone()
    }

    /// Load default policy rules
    pub async fn load_defaults(&self) {
        let defaults = vec![
            PolicyRule {
                id: "high-confidence-allow".to_string(),
                name: "High Confidence Allow".to_string(),
                description: "Allow actions when confidence is high".to_string(),
                priority: 100,
                condition: PolicyCondition::IfConfidenceAbove(0.8),
                action: PolicyAction::Allow,
                enabled: true,
            },
            PolicyRule {
                id: "low-confidence-deny".to_string(),
                name: "Low Confidence Deny".to_string(),
                description: "Deny actions when confidence is too low".to_string(),
                priority: 90,
                condition: PolicyCondition::IfConfidenceBelow(0.3),
                action: PolicyAction::RequireConfidence(0.5),
                enabled: true,
            },
            PolicyRule {
                id: "exploration-log".to_string(),
                name: "Exploration Logging".to_string(),
                description: "Log exploratory actions".to_string(),
                priority: 50,
                condition: PolicyCondition::Always,
                action: PolicyAction::LogAndAllow,
                enabled: true,
            },
        ];

        let mut rules = self.rules.write().await;
        *rules = defaults;
    }
}

impl Default for PolicyEngine {
    fn default() -> Self {
        Self::new()
    }
}

/// Result of policy evaluation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PolicyResult {
    Decision {
        action: PolicyAction,
        matched_rule: String,
        rule_name: String,
    },
    NoMatch,
}

/// Policy container with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Policy {
    pub id: String,
    pub name: String,
    pub version: String,
    pub rules: Vec<PolicyRule>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

impl Policy {
    /// Create a new empty policy
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            name: name.into(),
            version: "1.0.0".to_string(),
            rules: Vec::new(),
            created_at: chrono::Utc::now(),
        }
    }

    /// Add a rule to the policy
    pub fn add_rule(&mut self, rule: PolicyRule) {
        self.rules.push(rule);
    }
}
