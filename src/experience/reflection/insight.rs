// robot/src/experience/reflection/insight.rs
#![allow(dead_code)]

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// ============================================================================
/// Insight
/// ============================================================================
///
/// An Insight represents reusable knowledge extracted from one or more
/// reflections.
///
/// Unlike a Reflection, which is tied to a specific review of experiences,
/// an Insight is intended to survive long-term and be referenced by other
/// learning systems.
///
/// Insights can later support:
///
/// • Hypotheses
/// • Planning
/// • Reputation
/// • Exploration
/// • Evolution
///
/// Think of an Insight as:
///
///     "This appears to be true."
///
/// ============================================================================
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Insight {
    /// Unique identifier.
    pub id: String,

    /// Human-readable title.
    pub title: String,

    /// Main learning statement.
    pub statement: String,

    /// Detailed explanation.
    pub explanation: String,

    /// Type of insight.
    pub insight_type: InsightType,

    /// Confidence (0.0 - 1.0).
    pub confidence: f32,

    /// Estimated usefulness.
    pub usefulness: f32,

    /// Number of confirmations.
    pub confirmations: u32,

    /// Number of contradictions.
    pub contradictions: u32,

    /// Source reflections.
    pub reflection_ids: Vec<String>,

    /// Supporting experiences.
    pub experience_ids: Vec<String>,

    /// Related hypotheses.
    pub hypothesis_ids: Vec<String>,

    /// Optional tags.
    pub tags: Vec<String>,

    /// Creation time.
    pub created_at: DateTime<Utc>,

    /// Last updated.
    pub updated_at: DateTime<Utc>,
}

impl Insight {
    /// Create a new insight.
    pub fn new(
        id: impl Into<String>,
        title: impl Into<String>,
        statement: impl Into<String>,
        insight_type: InsightType,
    ) -> Self {
        let now = Utc::now();

        Self {
            id: id.into(),
            title: title.into(),
            statement: statement.into(),
            explanation: String::new(),
            insight_type,

            confidence: 0.5,
            usefulness: 0.5,

            confirmations: 0,
            contradictions: 0,

            reflection_ids: Vec::new(),
            experience_ids: Vec::new(),
            hypothesis_ids: Vec::new(),

            tags: Vec::new(),

            created_at: now,
            updated_at: now,
        }
    }

    /// Increase confidence.
    pub fn confirm(&mut self) {
        self.confirmations += 1;
        self.updated_at = Utc::now();
    }

    /// Record contradictory evidence.
    pub fn contradict(&mut self) {
        self.contradictions += 1;
        self.updated_at = Utc::now();
    }

    /// Attach a reflection.
    pub fn add_reflection(&mut self, reflection_id: impl Into<String>) {
        self.reflection_ids.push(reflection_id.into());
    }

    /// Attach an experience.
    pub fn add_experience(&mut self, experience_id: impl Into<String>) {
        self.experience_ids.push(experience_id.into());
    }

    /// Attach a hypothesis.
    pub fn add_hypothesis(&mut self, hypothesis_id: impl Into<String>) {
        self.hypothesis_ids.push(hypothesis_id.into());
    }

    /// Has enough evidence to influence planning?
    pub fn is_trusted(&self) -> bool {
        self.confidence >= 0.80 && self.confirmations > self.contradictions
    }
}

/// ============================================================================
/// Insight Type
/// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum InsightType {
    Behavioral,
    Performance,
    Strategy,
    Optimization,
    Pattern,
    Failure,
    Success,
    Communication,
    Memory,
    Reasoning,
    General,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum KnowledgeMaturity {
    /// Newly discovered.
    Emerging,

    /// Early supporting evidence exists.
    Developing,

    /// Multiple independent confirmations.
    Established,

    /// Extensively validated over time.
    Trusted,

    /// Losing confidence but still retained.
    Questioned,

    /// Superseded by newer evidence.
    Deprecated,

    /// Proven false.
    Rejected,
}

pub struct MaturityHistory {
    pub timestamp: DateTime<Utc>,
    pub previous: KnowledgeMaturity,
    pub current: KnowledgeMaturity,
    pub reason: String,
}
