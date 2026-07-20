// robot/src/experience/hypothesis/support/simulation.rs

//! ============================================================================
//! HYPOTHESIS SIMULATION
//! ============================================================================
//!
//! What-if reasoning system for exploring possible outcomes from hypotheses.
//!
//! This module simulates the implications of trusting or acting on hypotheses.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::experience::hypothesis::core::{HypothesisId, Hypothesis, HypothesisStatus};

/// ============================================================================
/// HYPOTHESIS SIMULATOR
/// ============================================================================

/// Simulator for exploring hypothesis implications
#[derive(Debug, Clone, Default)]
pub struct HypothesisSimulator {
    /// Simulation parameters
    params: SimulationParams,
}

impl HypothesisSimulator {
    /// Create a new simulator with default settings
    pub fn new() -> Self {
        Self {
            params: SimulationParams::default(),
        }
    }

    /// Create a simulator with custom parameters
    pub fn with_params(params: SimulationParams) -> Self {
        Self { params }
    }

    /// Simulate the outcome of trusting a hypothesis
    pub fn simulate(&self, hypothesis: &Hypothesis) -> SimulationResult {
        self.simulate_trust(hypothesis)
    }

    /// Simulate trusting a hypothesis
    fn simulate_trust(&self, hypothesis: &Hypothesis) -> SimulationResult {
        let mut outcomes = Vec::new();
        
        // Calculate expected outcomes based on confidence
        let success_probability = hypothesis.confidence.value;
        let failure_probability = 1.0 - success_probability;
        
        // Add primary outcome
        outcomes.push(Outcome {
            outcome_type: OutcomeType::Success,
            probability: success_probability,
            impact: self.params.success_impact,
            description: format!("Hypothesis '{}' is correct", hypothesis.title),
        });
        
        // Add failure outcome
        outcomes.push(Outcome {
            outcome_type: OutcomeType::Failure,
            probability: failure_probability,
            impact: self.params.failure_impact,
            description: format!("Hypothesis '{}' is incorrect", hypothesis.title),
        });
        
        // Calculate expected value
        let expected_value = (success_probability * self.params.success_impact)
            + (failure_probability * self.params.failure_impact);
        
        // Adjust confidence based on evidence count
        let evidence_bonus = (hypothesis.evaluations as f32 * 0.01).min(0.2);
        let adjusted_confidence = (hypothesis.confidence.value + evidence_bonus).min(1.0);
        
        SimulationResult {
            hypothesis_id: hypothesis.id.clone(),
            confidence: hypothesis.confidence.value,
            adjusted_confidence,
            outcomes,
            expected_value,
            risk_level: self.calculate_risk(success_probability),
            recommendations: self.generate_recommendations(hypothesis, success_probability),
            notes: format!(
                "Simulated {} outcomes with {:.2} expected value",
                outcomes.len(),
                expected_value
            ),
        }
    }

    /// Calculate risk level based on probability
    fn calculate_risk(&self, probability: f32) -> RiskLevel {
        if probability >= 0.9 {
            RiskLevel::VeryLow
        } else if probability >= 0.7 {
            RiskLevel::Low
        } else if probability >= 0.5 {
            RiskLevel::Medium
        } else if probability >= 0.3 {
            RiskLevel::High
        } else {
            RiskLevel::VeryHigh
        }
    }

    /// Generate recommendations based on simulation
    fn generate_recommendations(&self, hypothesis: &Hypothesis, probability: f32) -> Vec<String> {
        let mut recommendations = Vec::new();
        
        if probability >= 0.8 {
            recommendations.push("High confidence - safe to act on this hypothesis".to_string());
        } else if probability >= 0.6 {
            recommendations.push("Moderate confidence - consider validating further".to_string());
        } else {
            recommendations.push("Low confidence - gather more evidence before acting".to_string());
        }
        
        if hypothesis.evaluations < 3 {
            recommendations.push("Limited evidence - recommend additional testing".to_string());
        }
        
        if hypothesis.confirmations > hypothesis.contradictions {
            recommendations.push("Positive evidence trend - supports hypothesis".to_string());
        } else if hypothesis.contradictions > hypothesis.confirmations {
            recommendations.push("Negative evidence trend - reconsider hypothesis".to_string());
        }
        
        recommendations
    }

    /// Simulate multiple hypotheses
    pub fn simulate_batch(&self, hypotheses: &[Hypothesis]) -> Vec<SimulationResult> {
        hypotheses
            .iter()
            .map(|h| self.simulate(h))
            .collect()
    }

    /// Find the safest hypothesis to act on
    pub fn find_safest(&self, hypotheses: &[Hypothesis]) -> Option<&Hypothesis> {
        hypotheses
            .iter()
            .filter(|h| h.status == HypothesisStatus::Supported)
            .max_by(|a, b| {
                a.confidence.value
                    .partial_cmp(&b.confidence.value)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
    }

    /// Compare multiple hypotheses
    pub fn compare(&self, hypotheses: &[&Hypothesis]) -> Vec<SimulationResult> {
        hypotheses.iter().map(|h| self.simulate(h)).collect()
    }
}

/// ============================================================================
/// SIMULATION PARAMETERS
/// ============================================================================

#[derive(Debug, Clone)]
pub struct SimulationParams {
    /// Impact multiplier for success (positive value)
    pub success_impact: f32,
    
    /// Impact multiplier for failure (typically negative)
    pub failure_impact: f32,
    
    /// Number of simulation iterations
    pub iterations: u32,
}

impl Default for SimulationParams {
    fn default() -> Self {
        Self {
            success_impact: 1.0,
            failure_impact: -0.5,
            iterations: 100,
        }
    }
}

impl SimulationParams {
    pub fn conservative() -> Self {
        Self {
            success_impact: 0.8,
            failure_impact: -0.8,
            iterations: 100,
        }
    }
    
    pub fn aggressive() -> Self {
        Self {
            success_impact: 1.2,
            failure_impact: -0.3,
            iterations: 100,
        }
    }
}

/// ============================================================================
/// SIMULATION RESULT
/// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulationResult {
    pub hypothesis_id: HypothesisId,
    pub confidence: f32,
    pub adjusted_confidence: f32,
    pub outcomes: Vec<Outcome>,
    pub expected_value: f32,
    pub risk_level: RiskLevel,
    pub recommendations: Vec<String>,
    pub notes: String,
}

impl SimulationResult {
    /// Check if simulation suggests acting on hypothesis
    pub fn should_act(&self) -> bool {
        self.confidence >= 0.7 && self.expected_value > 0.0
    }
    
    /// Get the best outcome
    pub fn best_outcome(&self) -> Option<&Outcome> {
        self.outcomes
            .iter()
            .filter(|o| o.probability > 0.0)
            .max_by(|a, b| a.impact.partial_cmp(&b.impact).unwrap_or(std::cmp::Ordering::Equal))
    }
}

/// ============================================================================
/// OUTCOME
/// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Outcome {
    pub outcome_type: OutcomeType,
    pub probability: f32,
    pub impact: f32,
    pub description: String,
}

impl Outcome {
    /// Calculate expected value of this outcome
    pub fn expected_value(&self) -> f32 {
        self.probability * self.impact
    }
}

/// Outcome types
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum OutcomeType {
    Success,
    Failure,
    PartialSuccess,
    Unknown,
}

/// ============================================================================
/// RISK LEVEL
/// ============================================================================

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum RiskLevel {
    VeryLow,
    Low,
    Medium,
    High,
    VeryHigh,
}

impl std::fmt::Display for RiskLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RiskLevel::VeryLow => write!(f, "Very Low"),
            RiskLevel::Low => write!(f, "Low"),
            RiskLevel::Medium => write!(f, "Medium"),
            RiskLevel::High => write!(f, "High"),
            RiskLevel::VeryHigh => write!(f, "Very High"),
        }
    }
}

