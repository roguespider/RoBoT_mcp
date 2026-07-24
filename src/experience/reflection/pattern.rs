// /src/experience/reflection/pattern.rs
// Pattern detection and representation
#![allow(dead_code)]

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Represents a detected pattern from analyzing experiences
#[derive(Debug, Clone, Serialize, Deserialize, PartialOrd, PartialEq)]
pub struct Pattern {
    /// Unique identifier
    pub id: String,

    /// Human-readable description of the pattern
    pub description: String,

    /// Number of times this pattern was observed
    pub occurrences: u32,

    /// Confidence in the pattern (0.0 - 1.0)
    pub confidence: f32,

    /// IDs of experiences that support this pattern
    pub evidence: Vec<String>,

    /// When the pattern was first detected
    pub first_observed: DateTime<Utc>,

    /// When the pattern was last updated
    pub last_updated: DateTime<Utc>,

    /// Pattern type/category
    pub pattern_type: PatternType,

    /// Optional tags for categorization
    pub tags: Vec<String>,
}

/// Types of patterns that can be detected
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum PatternType {
    /// Sequential pattern (A always follows B)
    Sequential,

    /// Frequency pattern (X happens frequently)
    Frequency,

    /// Correlation pattern (X often occurs with Y)
    Correlation,

    /// Anomaly pattern (X is unusual)
    Anomaly,

    /// Success pattern (X tends to succeed)
    Success,

    /// Failure pattern (X tends to fail)
    Failure,

    /// Temporal pattern (X happens at certain times)
    Temporal,

    /// Custom pattern type
    Custom(String),
}

impl Pattern {
    /// Create a new pattern with default values
    pub fn new(description: impl Into<String>) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4().to_string(),
            description: description.into(),
            occurrences: 1,
            confidence: 0.5,
            evidence: Vec::new(),
            first_observed: now,
            last_updated: now,
            pattern_type: PatternType::Frequency,
            tags: Vec::new(),
        }
    }

    /// Create a pattern with a specific type
    pub fn with_type(description: impl Into<String>, pattern_type: PatternType) -> Self {
        let mut pattern = Self::new(description);
        pattern.pattern_type = pattern_type;
        pattern
    }

    /// Add evidence (experience ID) to the pattern
    pub fn add_evidence(&mut self, experience_id: impl Into<String>) {
        let id = experience_id.into();
        if !self.evidence.contains(&id) {
            self.evidence.push(id);
            self.occurrences += 1;
            self.last_updated = Utc::now();
            self.recalculate_confidence();
        }
    }

    /// Remove evidence from the pattern
    pub fn remove_evidence(&mut self, experience_id: &str) {
        self.evidence.retain(|e| e != experience_id);
        if self.occurrences > 0 {
            self.occurrences -= 1;
        }
        self.last_updated = Utc::now();
        self.recalculate_confidence();
    }

    /// Recalculate confidence based on current evidence
    fn recalculate_confidence(&mut self) {
        // Simple confidence calculation:
        // - Base confidence from occurrence ratio
        // - More evidence = higher confidence
        let occurrence_factor = (self.occurrences as f32 / 10.0).min(1.0);
        let evidence_factor = (self.evidence.len() as f32 / 5.0).min(1.0);

        self.confidence = (occurrence_factor * 0.4 + evidence_factor * 0.6).min(1.0);
        self.last_updated = Utc::now();
    }

    /// Add a tag to the pattern
    pub fn add_tag(&mut self, tag: impl Into<String>) {
        let tag = tag.into();
        if !self.tags.contains(&tag) {
            self.tags.push(tag);
            self.last_updated = Utc::now();
        }
    }

    /// Check if the pattern is statistically significant
    pub fn is_significant(&self, min_confidence: f32, min_occurrences: u32) -> bool {
        self.confidence >= min_confidence && self.occurrences >= min_occurrences
    }

    /// Check if the pattern has matured to actionable level
    pub fn is_mature(&self) -> bool {
        self.confidence >= 0.8 && self.occurrences >= 5
    }

    /// Get the pattern as a reusable insight
    pub fn to_insight_statement(&self) -> String {
        format!(
            "Pattern detected: {} (confidence: {:.0}%, occurrences: {})",
            self.description,
            self.confidence * 100.0,
            self.occurrences
        )
    }

    /// Merge another pattern into this one
    pub fn merge(&mut self, other: &Pattern) {
        for evidence in &other.evidence {
            if !self.evidence.contains(evidence) {
                self.add_evidence(evidence.clone());
            }
        }

        // Update description if other has higher confidence
        if other.confidence > self.confidence {
            self.description = other.description.clone();
        }

        self.last_updated = Utc::now();
    }

    /// Update the pattern type
    pub fn set_type(&mut self, pattern_type: PatternType) {
        self.pattern_type = pattern_type;
        self.last_updated = Utc::now();
    }

    /// Get the age of the pattern in days
    pub fn age_days(&self) -> i64 {
        (Utc::now() - self.first_observed).num_days()
    }

    /// Check if pattern is stale (not updated in N days)
    pub fn is_stale(&self, days: i64) -> bool {
        let age = (Utc::now() - self.last_updated).num_days();
        age >= days
    }
}

/// Compares patterns by confidence for sorting
#[allow(clippy::derive_ord_xor_partial_ord)]
impl Ord for Pattern {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.confidence
            .partial_cmp(&other.confidence)
            .unwrap_or(std::cmp::Ordering::Equal)
    }
}

impl Eq for Pattern {}
