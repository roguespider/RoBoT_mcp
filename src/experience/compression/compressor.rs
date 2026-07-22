// src/experience/compression/compressor.rs
//! Core experience compression logic

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::pattern::{Pattern, PatternDetector};
use super::exceptions::Exception;
use crate::experience::types::Experience;

/// Result of compressing multiple experiences
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompressionResult {
    /// Unique identifier for the compressed experience
    pub id: Uuid,
    
    /// The extracted pattern
    pub pattern: Pattern,
    
    /// Aggregated confidence (mean of all experiences)
    pub confidence: f32,
    
    /// Confidence range (±)
    pub confidence_range: f32,
    
    /// Number of experiences compressed
    pub experience_count: usize,
    
    /// Exceptions that don't fit the pattern
    pub exceptions: Vec<Exception>,
    
    /// When this compression was created
    pub compressed_at: DateTime<Utc>,
    
    /// Original experience IDs that were compressed
    pub source_experience_ids: Vec<Uuid>,
}

/// A compressed representation of multiple similar experiences
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompressedExperience {
    /// Unique identifier
    pub id: Uuid,
    
    /// Human-readable summary
    pub summary: String,
    
    /// The extracted pattern
    pub pattern: Pattern,
    
    /// Aggregated confidence score
    pub confidence: f32,
    
    /// Confidence standard deviation
    pub confidence_std: f32,
    
    /// Total experiences represented
    pub experience_count: usize,
    
    /// Exceptions that deviate from the pattern
    pub exceptions: Vec<Exception>,
    
    /// When compressed
    pub compressed_at: DateTime<Utc>,
}

impl CompressedExperience {
    /// Create a new compressed experience
    pub fn new(
        pattern: Pattern,
        confidence: f32,
        confidence_std: f32,
        experience_count: usize,
        exceptions: Vec<Exception>,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            summary: pattern.to_summary(),
            pattern,
            confidence,
            confidence_std,
            experience_count,
            exceptions,
            compressed_at: Utc::now(),
        }
    }
}

/// Experience compressor that reduces similar experiences into patterns
pub struct ExperienceCompressor {
    /// Minimum experiences needed before compression
    min_experiences: usize,
    
    /// Minimum similarity threshold (0.0 to 1.0)
    similarity_threshold: f32,
    
    /// Pattern detector
    pattern_detector: PatternDetector,
}

impl ExperienceCompressor {
    /// Create a new compressor with default settings
    pub fn new() -> Self {
        Self {
            min_experiences: 3,
            similarity_threshold: 0.7,
            pattern_detector: PatternDetector::new(),
        }
    }

    /// Create a compressor with custom settings
    pub fn with_config(min_experiences: usize, similarity_threshold: f32) -> Self {
        Self {
            min_experiences,
            similarity_threshold,
            pattern_detector: PatternDetector::new(),
        }
    }

    /// Compress multiple similar experiences into a single representation
    pub fn compress(&self, experiences: &[Experience]) -> Option<CompressionResult> {
        if experiences.len() < self.min_experiences {
            return None;
        }

        // Detect common pattern across experiences
        let pattern = self.pattern_detector.detect_pattern(experiences)?;
        
        // Check if experiences are similar enough
        if pattern.confidence < self.similarity_threshold {
            return None;
        }

        // Calculate aggregated confidence
        let (confidence, confidence_range) = self.calculate_confidence_stats(experiences);
        
        // Identify exceptions (experiences that don't fit the pattern)
        let exceptions = self.identify_exceptions(experiences, &pattern);
        
        // Filter out exceptions that are too similar
        let exceptions: Vec<_> = exceptions.into_iter()
            .filter(|e| e.deviation_score > 0.3)
            .collect();

        Some(CompressionResult {
            id: Uuid::new_v4(),
            pattern,
            confidence,
            confidence_range,
            experience_count: experiences.len(),
            exceptions,
            compressed_at: Utc::now(),
            source_experience_ids: experiences.iter().map(|e| e.id).collect(),
        })
    }

    /// Calculate confidence statistics across experiences
    fn calculate_confidence_stats(&self, experiences: &[Experience]) -> (f32, f32) {
        if experiences.is_empty() {
            return (0.0, 0.0);
        }

        let confidences: Vec<f32> = experiences.iter()
            .map(|e| e.confidence)
            .collect();

        let mean = confidences.iter().sum::<f32>() / confidences.len() as f32;
        
        // Calculate standard deviation
        let variance = confidences.iter()
            .map(|c| {
                let diff = c - mean;
                diff * diff
            })
            .sum::<f32>() / confidences.len() as f32;
        
        let std = variance.sqrt();
        
        // Confidence range is ±1 standard deviation
        (mean, std)
    }

    /// Identify experiences that don't fit the pattern
    fn identify_exceptions(&self, experiences: &[Experience], pattern: &Pattern) -> Vec<Exception> {
        let mut exceptions = Vec::new();

        for exp in experiences {
            let deviation = self.pattern_detector.calculate_deviation(exp, pattern);
            
            if deviation > 0.3 {
                exceptions.push(Exception::new(
                    exp.id,
                    pattern.id,
                    deviation,
                    format!("Deviation from pattern: {:.1}%", deviation * 100.0),
                ));
            }
        }

        exceptions.sort_by(|a, b| b.deviation_score.partial_cmp(a.deviation_score)).unwrap_or(std::cmp::Ordering::Equal);
        exceptions
    }

    /// Update compression with new experiences
    pub fn update(&self, compressed: &CompressionResult, new_experiences: &[Experience]) -> CompressionResult {
        let mut all_experiences: Vec<Experience> = Vec::new();
        
        // Note: We'd need to fetch the original experiences to do true incremental update
        // For now, merge with new experiences
        
        for exp in new_experiences {
            if !compressed.source_experience_ids.contains(&exp.id) {
                all_experiences.push(exp.clone());
            }
        }

        let result = self.compress(&all_experiences);
        
        result.unwrap_or_else(|| compressed.clone())
    }
}

impl Default for ExperienceCompressor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_experience(id: Uuid, title: &str, confidence: f32) -> Experience {
        Experience {
            id,
            timestamp: Utc::now(),
            experience_type: ExperienceType::ToolExecution,
            title: title.to_string(),
            description: "Test description".to_string(),
            context: crate::experience::types::ExperienceContext::default(),
            outcome: ExperienceOutcome::Success,
            score: None,
            encounter_ids: vec![],
            maturity: crate::experience::types::KnowledgeMaturity::Emerging,
            confidence,
            lessons: vec![],
            evidence_count: 0,
            tags: vec![],
            metadata: HashMap::new(),
        }
    }

    #[test]
    fn test_compress_single_experience_returns_none() {
        let compressor = ExperienceCompressor::new();
        let experiences = vec![create_test_experience(Uuid::new_v4(), "Test", 0.8)];
        
        let result = compressor.compress(&experiences);
        assert!(result.is_none());
    }

    #[test]
    fn test_compress_multiple_similar_experiences() {
        let compressor = ExperienceCompressor::new();
        let experiences = vec![
            create_test_experience(Uuid::new_v4(), "File read operation", 0.8),
            create_test_experience(Uuid::new_v4(), "File read operation", 0.85),
            create_test_experience(Uuid::new_v4(), "File read operation", 0.9),
        ];
        
        let result = compressor.compress(&experiences);
        assert!(result.is_some());
        
        let compressed = result.expect("Compression should succeed for test data");
        assert_eq!(compressed.experience_count, 3);
        assert!((compressed.confidence - 0.85).abs() < 0.01);
    }
}
