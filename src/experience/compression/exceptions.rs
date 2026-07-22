// src/experience/compression/exceptions.rs
//! Exception tracking for experiences that deviate from patterns

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// An exception represents an experience that deviates from a pattern
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Exception {
    /// Unique identifier for this exception
    pub id: Uuid,
    
    /// The experience ID that caused this exception
    pub experience_id: Uuid,
    
    /// The pattern ID this is an exception to
    pub pattern_id: Uuid,
    
    /// How much this experience deviates from the pattern (0.0 to 1.0)
    pub deviation_score: f32,
    
    /// Human-readable reason for the deviation
    pub reason: String,
    
    /// When this exception was recorded
    pub recorded_at: DateTime<Utc>,
    
    /// Type of deviation
    pub deviation_type: DeviationType,
    
    /// Whether this is a false positive (should match but doesn't)
    pub is_false_positive: bool,
}

impl Exception {
    /// Create a new exception
    pub fn new(experience_id: Uuid, pattern_id: Uuid, deviation_score: f32, reason: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            experience_id,
            pattern_id,
            deviation_score,
            reason,
            recorded_at: Utc::now(),
            deviation_type: DeviationType::Unknown,
            is_false_positive: false,
        }
    }

    /// Set the deviation type
    pub fn set_deviation_type(&mut self, deviation_type: DeviationType) {
        self.deviation_type = deviation_type;
    }

    /// Mark this as a false positive
    pub fn mark_false_positive(&mut self) {
        self.is_false_positive = true;
    }

    /// Check if this exception is significant
    pub fn is_significant(&self, threshold: f32) -> bool {
        !self.is_false_positive && self.deviation_score >= threshold
    }
}

/// Types of deviations from patterns
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum DeviationType {
    /// Missing expected elements
    MissingElements,
    
    /// Contains unexpected elements
    UnexpectedElements,
    
    /// Different outcome than expected
    DifferentOutcome,
    
    /// Different context than expected
    DifferentContext,
    
    /// Different tags than expected
    DifferentTags,
    
    /// Unknown deviation type
    Unknown,
}

impl DeviationType {
    /// Get a human-readable description
    pub fn description(&self) -> &'static str {
        match self {
            Self::MissingElements => "Missing expected elements",
            Self::UnexpectedElements => "Contains unexpected elements",
            Self::DifferentOutcome => "Different outcome than expected",
            Self::DifferentContext => "Different context than expected",
            Self::DifferentTags => "Different tags than expected",
            Self::Unknown => "Unknown deviation",
        }
    }
}

impl std::fmt::Display for DeviationType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.description())
    }
}

/// Tracks exceptions for patterns
pub struct ExceptionTracker {
    /// Exceptions grouped by pattern ID
    exceptions: Vec<Exception>,
    
    /// Minimum deviation score to be recorded
    min_deviation_threshold: f32,
}

impl ExceptionTracker {
    /// Create a new exception tracker
    pub fn new() -> Self {
        Self {
            exceptions: Vec::new(),
            min_deviation_threshold: 0.2,
        }
    }

    /// Create with custom threshold
    pub fn with_threshold(threshold: f32) -> Self {
        Self {
            exceptions: Vec::new(),
            min_deviation_threshold: threshold,
        }
    }

    /// Add an exception
    pub fn add_exception(&mut self, exception: Exception) {
        if exception.deviation_score >= self.min_deviation_threshold {
            self.exceptions.push(exception);
        }
    }

    /// Get exceptions for a specific pattern
    pub fn get_for_pattern(&self, pattern_id: &Uuid) -> Vec<&Exception> {
        self.exceptions.iter()
            .filter(|e| e.pattern_id == *pattern_id)
            .collect()
    }

    /// Get significant exceptions (above threshold)
    pub fn get_significant(&self, threshold: f32) -> Vec<&Exception> {
        self.exceptions.iter()
            .filter(|e| e.is_significant(threshold))
            .collect()
    }

    /// Get total exception count
    pub fn count(&self) -> usize {
        self.exceptions.len()
    }

    /// Get count for a pattern
    pub fn count_for_pattern(&self, pattern_id: &Uuid) -> usize {
        self.exceptions.iter()
            .filter(|e| e.pattern_id == *pattern_id)
            .count()
    }

    /// Clear exceptions for a pattern
    pub fn clear_for_pattern(&mut self, pattern_id: &Uuid) {
        self.exceptions.retain(|e| e.pattern_id != *pattern_id);
    }

    /// Mark an exception as false positive
    pub fn mark_false_positive(&mut self, exception_id: &Uuid) {
        if let Some(exception) = self.exceptions.iter_mut().find(|e| e.id == *exception_id) {
            exception.mark_false_positive();
        }
    }

    /// Get exceptions by type
    pub fn get_by_type(&self, deviation_type: DeviationType) -> Vec<&Exception> {
        self.exceptions.iter()
            .filter(|e| e.deviation_type == deviation_type)
            .collect()
    }

    /// Calculate exception rate for a pattern
    pub fn exception_rate(&self, pattern_id: &Uuid, total_experiences: usize) -> f32 {
        if total_experiences == 0 {
            return 0.0;
        }
        
        let pattern_exceptions = self.count_for_pattern(pattern_id);
        pattern_exceptions as f32 / total_experiences as f32
    }

    /// Get top exceptions by deviation score
    pub fn get_top(&self, limit: usize) -> Vec<&Exception> {
        let mut sorted: Vec<_> = self.exceptions.iter().collect();
        sorted.sort_by(|a, b| b.deviation_score.partial_cmp(&a.deviation_score).unwrap_or(std::cmp::Ordering::Equal));
        sorted.into_iter().take(limit).collect()
    }
}

impl Default for ExceptionTracker {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_and_retrieve_exception() {
        let mut tracker = ExceptionTracker::new();
        let exp_id = Uuid::new_v4();
        let pattern_id = Uuid::new_v4();
        
        let exception = Exception::new(
            exp_id,
            pattern_id,
            0.5,
            "Test deviation".to_string(),
        );
        
        tracker.add_exception(exception);
        
        let exceptions = tracker.get_for_pattern(&pattern_id);
        assert_eq!(exceptions.len(), 1);
    }

    #[test]
    fn test_exception_threshold() {
        let mut tracker = ExceptionTracker::with_threshold(0.5);
        let exp_id = Uuid::new_v4();
        let pattern_id = Uuid::new_v4();
        
        // Below threshold
        let low_exception = Exception::new(exp_id, pattern_id, 0.3, "Low deviation".to_string());
        tracker.add_exception(low_exception);
        
        assert_eq!(tracker.count(), 0);
        
        // Above threshold
        let high_exception = Exception::new(exp_id, pattern_id, 0.6, "High deviation".to_string());
        tracker.add_exception(high_exception);
        
        assert_eq!(tracker.count(), 1);
    }
}
