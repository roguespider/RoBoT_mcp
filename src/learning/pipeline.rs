// src/learning/pipeline.rs
//! Learning Pipeline - Per Architecture §9
//!
//! The learning pipeline transforms raw input into learned knowledge through multiple stages:
//! Input → Observation → Memory → Experience → Knowledge → Planning → Decision → Action → Reflection
//!
//! This module orchestrates the flow of information through these stages.

use std::sync::Arc;

use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Pipeline stage types
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum PipelineStage {
    /// Raw input received
    Input,
    /// Observation detected and classified
    Observation,
    /// Stored in memory
    Memory,
    /// Created as experience
    Experience,
    /// Processed into knowledge
    Knowledge,
    /// Used for planning
    Planning,
    /// Decision made
    Decision,
    /// Action taken
    Action,
    /// Reflected upon
    Reflection,
}

impl std::fmt::Display for PipelineStage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            PipelineStage::Input => "Input",
            PipelineStage::Observation => "Observation",
            PipelineStage::Memory => "Memory",
            PipelineStage::Experience => "Experience",
            PipelineStage::Knowledge => "Knowledge",
            PipelineStage::Planning => "Planning",
            PipelineStage::Decision => "Decision",
            PipelineStage::Action => "Action",
            PipelineStage::Reflection => "Reflection",
        };
        write!(f, "{}", s)
    }
}

/// A record of an item passing through the learning pipeline
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineRecord {
    /// Unique identifier for this record
    pub id: Uuid,
    
    /// Source input that started this pipeline
    pub source_id: Uuid,
    
    /// Current stage in the pipeline
    pub current_stage: PipelineStage,
    
    /// Stages that have been completed
    pub completed_stages: Vec<PipelineStage>,
    
    /// When the pipeline started
    pub started_at: DateTime<Utc>,
    
    /// When the current stage was entered
    pub stage_entered_at: DateTime<Utc>,
    
    /// Metadata from each stage
    pub stage_data: Vec<StageData>,
}

/// Data captured at each stage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StageData {
    pub stage: PipelineStage,
    pub timestamp: DateTime<Utc>,
    pub summary: String,
    pub confidence: Option<f32>,
}

/// The learning pipeline coordinator
pub struct LearningPipeline {
    /// Maximum items in the pipeline
    max_items: usize,
    
    /// Active pipeline records
    records: std::collections::HashMap<Uuid, PipelineRecord>,
}

impl LearningPipeline {
    /// Create a new learning pipeline
    pub fn new(max_items: usize) -> Self {
        Self {
            max_items,
            records: std::collections::HashMap::new(),
        }
    }
    
    /// Start a new pipeline record from input
    pub fn start_from_input(&mut self, source_id: Uuid, summary: &str) -> Uuid {
        let id = Uuid::new_v4();
        let now = Utc::now();
        
        let record = PipelineRecord {
            id,
            source_id,
            current_stage: PipelineStage::Input,
            completed_stages: Vec::new(),
            started_at: now,
            stage_entered_at: now,
            stage_data: vec![StageData {
                stage: PipelineStage::Input,
                timestamp: now,
                summary: summary.to_string(),
                confidence: None,
            }],
        };
        
        self.records.insert(id, record);
        id
    }
    
    /// Advance to the next stage
    pub fn advance_stage(&mut self, record_id: &Uuid, stage: PipelineStage, summary: &str, confidence: Option<f32>) -> bool {
        if let Some(record) = self.records.get_mut(record_id) {
            // Mark current stage as completed
            record.completed_stages.push(record.current_stage);
            
            // Update to new stage
            record.current_stage = stage;
            record.stage_entered_at = Utc::now();
            
            // Add stage data
            record.stage_data.push(StageData {
                stage,
                timestamp: Utc::now(),
                summary: summary.to_string(),
                confidence,
            });
            
            true
        } else {
            false
        }
    }
    
    /// Get a pipeline record
    pub fn get(&self, record_id: &Uuid) -> Option<&PipelineRecord> {
        self.records.get(record_id)
    }
    
    /// Get all records in a specific stage
    pub fn get_by_stage(&self, stage: PipelineStage) -> Vec<&PipelineRecord> {
        self.records
            .values()
            .filter(|r| r.current_stage == stage)
            .collect()
    }
    
    /// Get pipeline statistics
    pub fn stats(&self) -> PipelineStats {
        let mut stage_counts = std::collections::HashMap::new();
        for record in self.records.values() {
            *stage_counts.entry(record.current_stage).or_insert(0) += 1;
        }
        
        PipelineStats {
            total_records: self.records.len(),
            stage_counts,
        }
    }
    
    /// Clean up old records
    pub fn cleanup(&mut self, max_age: chrono::Duration) {
        let cutoff = Utc::now() - max_age;
        self.records.retain(|_, record| record.started_at > cutoff);
    }
}

/// Pipeline statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineStats {
    pub total_records: usize,
    pub stage_counts: std::collections::HashMap<PipelineStage, usize>,
}

impl Default for LearningPipeline {
    fn default() -> Self {
        Self::new(1000)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_start_pipeline() {
        let mut pipeline = LearningPipeline::new(100);
        let source_id = Uuid::new_v4();
        
        let record_id = pipeline.start_from_input(source_id, "Test input");
        
        let record = pipeline.get(&record_id).unwrap();
        assert_eq!(record.current_stage, PipelineStage::Input);
        assert_eq!(record.completed_stages.len(), 0);
    }

    #[test]
    fn test_advance_stage() {
        let mut pipeline = LearningPipeline::new(100);
        let source_id = Uuid::new_v4();
        
        let record_id = pipeline.start_from_input(source_id, "Test input");
        pipeline.advance_stage(&record_id, PipelineStage::Observation, "Observation made", Some(0.8));
        
        let record = pipeline.get(&record_id).unwrap();
        assert_eq!(record.current_stage, PipelineStage::Observation);
        assert!(record.completed_stages.contains(&PipelineStage::Input));
    }

    #[test]
    fn test_pipeline_stats() {
        let mut pipeline = LearningPipeline::new(100);
        
        let id1 = pipeline.start_from_input(Uuid::new_v4(), "Input 1");
        pipeline.advance_stage(&id1, PipelineStage::Observation, "Obs 1", None);
        
        let _id2 = pipeline.start_from_input(Uuid::new_v4(), "Input 2");
        
        let stats = pipeline.stats();
        assert_eq!(stats.total_records, 2);
        // id1 moved to Observation, id2 still in Input
        assert_eq!(stats.stage_counts.get(&PipelineStage::Input), Some(&1));
        assert_eq!(stats.stage_counts.get(&PipelineStage::Observation), Some(&1));
    }
}
