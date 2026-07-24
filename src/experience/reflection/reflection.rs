// robot/src/experience/reflection/reflection.rs
#![allow(dead_code)]

// Proposed structure
//    ├── metadata
//    ├── summary
//    ├── lessons
//    ├── evidence
//    ├── confidence
//    ├── relationships
//    ├── status
//    └── lifecycle methods

use chrono::Utc;
use serde::{Deserialize, Serialize};

use super::{ReflectionConfidence, ReflectionMetadata, ReflectionStatus, ReflectionType};

/// Type alias for experience ID
pub type ExperienceId = String;

/// Type alias for evidence ID
pub type EvidenceId = String;

/// Type alias for insight ID
pub type InsightId = String;

/// ============================================================================
/// Reflection
/// ============================================================================
///
/// A reflection represents knowledge extracted after examining one or more
/// experiences.
///
/// Reflections do NOT directly modify behavior.
///
/// Instead they provide:
///
/// • lessons learned
/// • detected patterns
/// • possible improvements
/// • supporting evidence
///
/// Other systems (Hypothesis, Evolution, Planning) decide how to use them.
/// ============================================================================
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Reflection {
    /// Unique identifier.
    pub id: String,

    /// Shared metadata.
    pub metadata: ReflectionMetadata,

    /// Reflection category.
    pub reflection_type: ReflectionType,

    /// Lifecycle state.
    pub status: ReflectionStatus,

    /// Human-readable title.
    pub title: String,

    /// Brief summary.
    pub summary: String,

    /// Full reasoning.
    pub description: String,

    /// Experiences examined.
    pub experience_ids: Vec<ExperienceId>,

    /// Related reflections.
    pub related_reflections: Vec<String>,

    /// Confidence measurements.
    pub confidence: ReflectionConfidence,

    /// Tags.
    pub tags: Vec<String>,
}

impl Reflection {
    /// Create a new reflection.
    pub fn new(
        id: impl Into<String>,
        reflection_type: ReflectionType,
        title: impl Into<String>,
    ) -> Self {
        let now = Utc::now();

        Self {
            id: id.into(),

            metadata: ReflectionMetadata {
                created_at: now,
                updated_at: now,
                source: "reflection_engine".to_string(),
                version: 1,
            },

            reflection_type,

            status: ReflectionStatus::Draft,

            title: title.into(),

            summary: String::new(),

            description: String::new(),

            experience_ids: Vec::new(),

            related_reflections: Vec::new(),

            confidence: ReflectionConfidence {
                score: 0.0,
                supporting_experiences: 0,
                contradictory_experiences: 0,
            },

            tags: Vec::new(),
        }
    }

    /// Attach an experience.
    pub fn add_experience(&mut self, experience_id: ExperienceId) {
        self.experience_ids.push(experience_id);
    }

    /// Mark as validated.
    pub fn validate(&mut self) {
        self.status = ReflectionStatus::Validated;
        self.metadata.updated_at = Utc::now();
    }

    /// Archive reflection.
    pub fn archive(&mut self) {
        self.status = ReflectionStatus::Archived;
        self.metadata.updated_at = Utc::now();
    }

    /// Update confidence.
    pub fn set_confidence(&mut self, score: f32) {
        self.confidence.score = score.clamp(0.0, 1.0);
        self.metadata.updated_at = Utc::now();
    }

    /// Has enough evidence to be useful?
    pub fn is_actionable(&self) -> bool {
        self.confidence.score >= 0.70
    }

    /// Number of experiences involved.
    pub fn experience_count(&self) -> usize {
        self.experience_ids.len()
    }
}

/// ============================================================================
/// Lesson
/// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Lesson {
    pub title: String,
    pub description: String,
    pub confidence: f32,
}

/// ============================================================================
/// Reflection Insight
/// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReflectionInsight {
    pub statement: String,
    pub confidence: f32,
    pub importance: f32,
}

/// ============================================================================
/// Evidence
/// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReflectionEvidence {
    pub experience_id: ExperienceId,
    pub description: String,
    pub weight: f32,
}
