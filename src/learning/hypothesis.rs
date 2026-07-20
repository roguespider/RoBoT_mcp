// src/learning/hypothesis.rs
//! Hypothesis management for learning

use std::sync::Arc;

use anyhow::Result;
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use uuid::Uuid;

/// Hypothesis status
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum HypothesisStatus {
    Proposed,
    Testing,
    Supported,
    Refuted,
    Abandoned,
}

/// Evidence for a hypothesis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HypothesisEvidence {
    pub id: String,
    pub description: String,
    pub evidence_type: EvidenceType,
    pub strength: f32,
    pub source: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum EvidenceType {
    Observation,
    Experiment,
    External,
    Reasoning,
}

/// A hypothesis for learning
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Hypothesis {
    pub id: String,
    pub statement: String,
    pub description: String,
    pub status: HypothesisStatus,
    pub confidence: f32,
    pub supporting_evidence: Vec<HypothesisEvidence>,
    pub contradicting_evidence: Vec<HypothesisEvidence>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
    pub tested_at: Option<chrono::DateTime<chrono::Utc>>,
}

impl Hypothesis {
    /// Create a new hypothesis
    pub fn new(statement: impl Into<String>, description: impl Into<String>) -> Self {
        let now = chrono::Utc::now();
        Self {
            id: Uuid::new_v4().to_string(),
            statement: statement.into(),
            description: description.into(),
            status: HypothesisStatus::Proposed,
            confidence: 0.5,
            supporting_evidence: Vec::new(),
            contradicting_evidence: Vec::new(),
            created_at: now,
            updated_at: now,
            tested_at: None,
        }
    }

    /// Add supporting evidence
    pub fn add_supporting(&mut self, evidence: HypothesisEvidence) {
        self.supporting_evidence.push(evidence);
        self.recalculate_confidence();
        self.updated_at = chrono::Utc::now();
    }

    /// Add contradicting evidence
    pub fn add_contradicting(&mut self, evidence: HypothesisEvidence) {
        self.contradicting_evidence.push(evidence);
        self.recalculate_confidence();
        self.updated_at = chrono::Utc::now();
    }

    /// Recalculate confidence based on evidence
    fn recalculate_confidence(&mut self) {
        let supporting_strength: f32 = self.supporting_evidence.iter().map(|e| e.strength).sum();
        let contradicting_strength: f32 = self.contradicting_evidence.iter().map(|e| e.strength).sum();
        
        let total = supporting_strength + contradicting_strength;
        if total > 0.0 {
            self.confidence = supporting_strength / total;
        }
    }

    /// Mark as being tested
    pub fn start_testing(&mut self) {
        self.status = HypothesisStatus::Testing;
        self.updated_at = chrono::Utc::now();
    }

    /// Mark as supported
    pub fn support(&mut self) {
        self.status = HypothesisStatus::Supported;
        self.tested_at = Some(chrono::Utc::now());
        self.updated_at = chrono::Utc::now();
    }

    /// Mark as refuted
    pub fn refute(&mut self) {
        self.status = HypothesisStatus::Refuted;
        self.tested_at = Some(chrono::Utc::now());
        self.updated_at = chrono::Utc::now();
    }

    /// Abandon the hypothesis
    pub fn abandon(&mut self) {
        self.status = HypothesisStatus::Abandoned;
        self.updated_at = chrono::Utc::now();
    }
}

/// Evidence builder
pub struct EvidenceBuilder {
    description: String,
    evidence_type: EvidenceType,
    strength: f32,
    source: String,
}

impl EvidenceBuilder {
    pub fn new(description: impl Into<String>) -> Self {
        Self {
            description: description.into(),
            evidence_type: EvidenceType::Observation,
            strength: 0.5,
            source: String::new(),
        }
    }

    pub fn with_type(mut self, evidence_type: EvidenceType) -> Self {
        self.evidence_type = evidence_type;
        self
    }

    pub fn with_strength(mut self, strength: f32) -> Self {
        self.strength = strength.clamp(0.0, 1.0);
        self
    }

    pub fn from_source(mut self, source: impl Into<String>) -> Self {
        self.source = source.into();
        self
    }

    pub fn build(self) -> HypothesisEvidence {
        HypothesisEvidence {
            id: Uuid::new_v4().to_string(),
            description: self.description,
            evidence_type: self.evidence_type,
            strength: self.strength,
            source: self.source,
            created_at: chrono::Utc::now(),
        }
    }
}

/// Hypothesis manager
pub struct HypothesisManager {
    hypotheses: Arc<RwLock<Vec<Hypothesis>>>,
}

impl HypothesisManager {
    /// Create a new hypothesis manager
    pub fn new() -> Self {
        Self {
            hypotheses: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Create a new hypothesis
    pub async fn create(&self, statement: impl Into<String>, description: impl Into<String>) -> Hypothesis {
        let hypothesis = Hypothesis::new(statement, description);
        let mut hypotheses = self.hypotheses.write().await;
        hypotheses.push(hypothesis.clone());
        hypothesis
    }

    /// Get a hypothesis by ID
    pub async fn get(&self, id: &str) -> Option<Hypothesis> {
        let hypotheses = self.hypotheses.read().await;
        hypotheses.iter().find(|h| h.id == id).cloned()
    }

    /// List all hypotheses
    pub async fn list(&self) -> Vec<Hypothesis> {
        let hypotheses = self.hypotheses.read().await;
        hypotheses.clone()
    }

    /// List by status
    pub async fn list_by_status(&self, status: HypothesisStatus) -> Vec<Hypothesis> {
        let hypotheses = self.hypotheses.read().await;
        hypotheses.iter().filter(|h| h.status == status).cloned().collect()
    }

    /// Get supported hypotheses
    pub async fn get_supported(&self) -> Vec<Hypothesis> {
        self.list_by_status(HypothesisStatus::Supported).await
    }

    /// Get high confidence hypotheses
    pub async fn get_high_confidence(&self, threshold: f32) -> Vec<Hypothesis> {
        let hypotheses = self.hypotheses.read().await;
        let mut result: Vec<_> = hypotheses.iter()
            .filter(|h| h.confidence >= threshold)
            .cloned()
            .collect();
        
        result.sort_by(|a, b| b.confidence.partial_cmp(&a.confidence).unwrap());
        result
    }

    /// Update a hypothesis
    pub async fn update(&self, hypothesis: &Hypothesis) -> Result<()> {
        let mut hypotheses = self.hypotheses.write().await;
        if let Some(existing) = hypotheses.iter_mut().find(|h| h.id == hypothesis.id) {
            *existing = hypothesis.clone();
        }
        Ok(())
    }

    /// Delete a hypothesis
    pub async fn delete(&self, id: &str) -> Option<Hypothesis> {
        let mut hypotheses = self.hypotheses.write().await;
        let idx = hypotheses.iter().position(|h| h.id == id)?;
        Some(hypotheses.remove(idx))
    }

    /// Get statistics
    pub async fn stats(&self) -> HypothesisStats {
        let hypotheses = self.hypotheses.read().await;
        
        let mut by_status: std::collections::HashMap<HypothesisStatus, usize> = std::collections::HashMap::new();
        for h in hypotheses.iter() {
            *by_status.entry(h.status).or_insert(0) += 1;
        }

        let avg_confidence = if hypotheses.is_empty() {
            0.0
        } else {
            hypotheses.iter().map(|h| h.confidence).sum::<f32>() / hypotheses.len() as f32
        };

        let total_evidence: usize = hypotheses.iter()
            .map(|h| h.supporting_evidence.len() + h.contradicting_evidence.len())
            .sum();

        HypothesisStats {
            total: hypotheses.len(),
            by_status,
            avg_confidence,
            total_evidence,
        }
    }
}

impl Default for HypothesisManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Statistics about hypotheses
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HypothesisStats {
    pub total: usize,
    pub by_status: std::collections::HashMap<HypothesisStatus, usize>,
    pub avg_confidence: f32,
    pub total_evidence: usize,
}
