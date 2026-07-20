//! ============================================================================
//! Reflection System
//! ============================================================================
//!
//! Reflection is responsible for examining past experiences and extracting
//! reusable knowledge from them.
//!
//! Reflection **does not change behavior directly.**
//!
//! Instead, it:
//! - Identifies lessons learned
//! - Detects recurring patterns
//! - Measures confidence in conclusions
//! - Produces insights for the Hypothesis system
//! - Supports long-term learning and evolution
//!
//! The overall learning pipeline is:
//!
//! Experience
//!     ↓
//! Reflection
//!     ↓
//! Insights / Patterns
//!     ↓
//! Hypothesis
//!     ↓
//! Exploration
//!     ↓
//! Evolution
//!
//! The reflection system acts as the robot's "thinking after doing."

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

pub mod engine;
pub mod insight;
pub mod pattern;
pub mod reflection;
pub mod review;
pub mod services;

// Re-export from submodules
pub use reflection::Reflection;
pub use engine::ReflectionEngine;

/// ============================================================================
/// Reflection Types
/// ============================================================================

/// High-level category describing the purpose of a reflection.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum ReflectionType {
    /// Something worked particularly well.
    Success,

    /// Something failed or performed poorly.
    Failure,

    /// Opportunity for improvement.
    Improvement,

    /// A recurring pattern has been observed.
    Pattern,

    /// Unexpected or unusual behavior.
    Anomaly,

    /// Review of an overall strategy or workflow.
    Strategy,

    /// General learning not fitting another category.
    General,
}

/// Current lifecycle state.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum ReflectionStatus {
    Draft,
    Active,
    Validated,
    Archived,
}

/// Overall confidence assigned to a reflection.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReflectionConfidence {
    /// Overall confidence (0.0 - 1.0)
    pub score: f32,

    /// Number of supporting experiences.
    pub supporting_experiences: usize,

    /// Number of contradictory experiences.
    pub contradictory_experiences: usize,
}

/// ============================================================================
/// Traits
/// ============================================================================

/// Produces reflections from one or more experiences.
pub trait Reflector {
    type Input;
    type Output;

    fn reflect(&self, input: Self::Input) -> anyhow::Result<Self::Output>;
}

/// Something that can be validated over time.
pub trait ValidatableReflection {
    fn confidence(&self) -> f32;

    fn validate(&mut self);

    fn invalidate(&mut self);
}

/// Anything capable of producing insights.
pub trait InsightProducer {
    fn generate_insights(&self) -> Vec<String>;
}

/// ============================================================================
/// Metadata
/// ============================================================================

/// Common metadata shared by reflection records.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReflectionMetadata {
    /// Creation timestamp.
    pub created_at: DateTime<Utc>,

    /// Last update timestamp.
    pub updated_at: DateTime<Utc>,

    /// Source of the reflection.
    pub source: String,

    /// Version number.
    pub version: u32,
}
