// robot/src/experience/hypothesis/support/planner.rs

//! ============================================================================
//! HYPOTHESIS PLANNER
//! ============================================================================
//!
//! Decision-support layer for converting trusted hypotheses into actions.
//!
//! This module helps RoBoT plan actions based on high-confidence hypotheses.

use serde::{Deserialize, Serialize};

use crate::experience::hypothesis::core::{HypothesisId, Hypothesis, HypothesisConfidence};

/// ============================================================================
/// HYPOTHESIS PLANNER
/// ============================================================================

/// Planner that converts hypotheses into actionable plans
#[derive(Debug, Clone, Default)]
pub struct HypothesisPlanner {
    /// Minimum confidence threshold for considering a hypothesis
    min_confidence: f32,
}

impl HypothesisPlanner {
    /// Create a new planner with default settings
    pub fn new() -> Self {
        Self {
            min_confidence: 0.7,
        }
    }

    /// Create a planner with custom confidence threshold
    pub fn with_confidence_threshold(mut self, threshold: f32) -> Self {
        self.min_confidence = threshold.clamp(0.0, 1.0);
        self
    }

    /// Create a plan for a hypothesis
    pub fn create_plan(&self, hypothesis: &Hypothesis) -> PlanningResult {
        if hypothesis.confidence.value >= self.min_confidence {
            self.generate_actionable_plan(hypothesis)
        } else {
            PlanningResult {
                hypothesis_id: hypothesis.id.clone(),
                confidence: hypothesis.confidence.value,
                actions: Vec::new(),
                status: PlanningStatus::LowConfidence,
                notes: format!(
                    "Hypothesis confidence ({:.2}) below threshold ({:.2}). More evidence needed.",
                    hypothesis.confidence.value, self.min_confidence
                ),
            }
        }
    }

    /// Generate actionable plan for a high-confidence hypothesis
    fn generate_actionable_plan(&self, hypothesis: &Hypothesis) -> PlanningResult {
        let actions = self.derive_actions(hypothesis);
        
        PlanningResult {
            hypothesis_id: hypothesis.id.clone(),
            confidence: hypothesis.confidence.value,
            actions,
            status: PlanningStatus::Ready,
            notes: format!(
                "Created plan for '{}' with {} actionable steps",
                hypothesis.title, 
                if actions.is_empty() { "0" } else { "multiple" }
            ),
        }
    }

    /// Derive actionable steps from hypothesis
    fn derive_actions(&self, hypothesis: &Hypothesis) -> Vec<PlannedAction> {
        let mut actions = Vec::new();
        
        // Generate actions based on hypothesis category
        match hypothesis.category {
            crate::experience::hypothesis::core::HypothesisCategory::Behavioral => {
                actions.push(PlannedAction {
                    action_type: ActionType::AdjustBehavior,
                    description: format!("Implement behavior: {}", hypothesis.title),
                    priority: hypothesis.priority,
                    estimated_impact: hypothesis.confidence.value,
                });
            }
            crate::experience::hypothesis::core::HypothesisCategory::Preference => {
                actions.push(PlannedAction {
                    action_type: ActionType::UpdatePreference,
                    description: format!("Update preference based on: {}", hypothesis.title),
                    priority: hypothesis.priority,
                    estimated_impact: hypothesis.confidence.value,
                });
            }
            crate::experience::hypothesis::core::HypothesisCategory::Performance => {
                actions.push(PlannedAction {
                    action_type: ActionType::Optimize,
                    description: format!("Optimize performance: {}", hypothesis.title),
                    priority: hypothesis.priority,
                    estimated_impact: hypothesis.confidence.value,
                });
            }
            crate::experience::hypothesis::core::HypothesisCategory::Workflow => {
                actions.push(PlannedAction {
                    action_type: ActionType::ImproveWorkflow,
                    description: format!("Improve workflow: {}", hypothesis.title),
                    priority: hypothesis.priority,
                    estimated_impact: hypothesis.confidence.value,
                });
            }
            crate::experience::hypothesis::core::HypothesisCategory::Knowledge => {
                actions.push(PlannedAction {
                    action_type: ActionType::Learn,
                    description: format!("Learn from: {}", hypothesis.title),
                    priority: hypothesis.priority,
                    estimated_impact: hypothesis.confidence.value,
                });
            }
            _ => {
                actions.push(PlannedAction {
                    action_type: ActionType::General,
                    description: format!("Apply learning: {}", hypothesis.title),
                    priority: hypothesis.priority,
                    estimated_impact: hypothesis.confidence.value,
                });
            }
        }
        
        // Add validation action if evidence is weak
        if hypothesis.evaluations < 3 {
            actions.push(PlannedAction {
                action_type: ActionType::Validate,
                description: "Gather more evidence to validate this hypothesis".to_string(),
                priority: crate::experience::hypothesis::core::HypothesisPriority::Normal,
                estimated_impact: 0.5,
            });
        }
        
        actions
    }

    /// Create multiple plans for a list of hypotheses
    pub fn create_plans(&self, hypotheses: &[Hypothesis]) -> Vec<PlanningResult> {
        hypotheses
            .iter()
            .filter(|h| h.status == crate::experience::hypothesis::core::HypothesisStatus::Supported)
            .map(|h| self.create_plan(h))
            .collect()
    }

    /// Get prioritized list of actions across all plans
    pub fn get_prioritized_actions(&self, hypotheses: &[Hypothesis]) -> Vec<PlannedAction> {
        let plans = self.create_plans(hypotheses);
        let mut all_actions: Vec<(i32, f32, PlannedAction)> = plans
            .into_iter()
            .filter(|p| p.status == PlanningStatus::Ready)
            .flat_map(|p| {
                p.actions.into_iter().map(|a| {
                    let priority_value = match p.confidence >= 0.9 {
                        true => 100,
                        false if p.confidence >= 0.8 => 75,
                        false if p.confidence >= 0.7 => 50,
                        _ => 25,
                    };
                    (priority_value, a.estimated_impact, a)
                })
            })
            .collect();
        
        // Sort by priority (descending) then impact (descending)
        all_actions.sort_by(|a, b| {
            b.0.cmp(&a.0)
                .then_with(|| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal))
        });
        
        all_actions.into_iter().map(|(_, _, a)| a).collect()
    }
}

/// ============================================================================
/// PLANNING RESULT
/// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanningResult {
    pub hypothesis_id: HypothesisId,
    pub confidence: f32,
    pub actions: Vec<PlannedAction>,
    pub status: PlanningStatus,
    pub notes: String,
}

/// Planning status
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum PlanningStatus {
    /// Plan is ready to execute
    Ready,
    /// Low confidence, more evidence needed
    LowConfidence,
    /// Hypothesis was rejected
    Rejected,
    /// No actions possible
    NoActions,
}

impl Default for PlanningStatus {
    fn default() -> Self {
        Self::NoActions
    }
}

/// ============================================================================
/// PLANNED ACTION
/// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlannedAction {
    pub action_type: ActionType,
    pub description: String,
    pub priority: crate::experience::hypothesis::core::HypothesisPriority,
    pub estimated_impact: f32,
}

/// Types of actions that can be planned
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum ActionType {
    /// General action
    General,
    /// Adjust behavior based on hypothesis
    AdjustBehavior,
    /// Update user/system preference
    UpdatePreference,
    /// Optimize performance
    Optimize,
    /// Improve workflow
    ImproveWorkflow,
    /// Learn from hypothesis
    Learn,
    /// Validate hypothesis with more evidence
    Validate,
}
