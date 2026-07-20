// src/learning/candidates.rs
//! Candidate actions and behavior generation

use std::sync::Arc;

use anyhow::Result;
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use uuid::Uuid;

/// A candidate action or behavior
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Candidate {
    pub id: String,
    pub name: String,
    pub description: String,
    pub action_type: CandidateType,
    pub score: CandidateScore,
    pub prerequisites: Vec<String>,
    pub expected_outcome: String,
    pub risk_level: RiskLevel,
    pub estimated_cost: f32,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum CandidateType {
    Behavior,
    Strategy,
    Tool,
    Workflow,
    Exploration,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CandidateScore {
    pub overall: f32,
    pub novelty: f32,
    pub expected_value: f32,
    pub feasibility: f32,
    pub safety: f32,
}

impl CandidateScore {
    /// Create a new score
    pub fn new(overall: f32) -> Self {
        Self {
            overall,
            novelty: 0.5,
            expected_value: 0.5,
            feasibility: 0.5,
            safety: 1.0,
        }
    }

    /// Calculate weighted overall score
    pub fn calculate_weighted(&self) -> f32 {
        self.overall * 0.4 +
        self.expected_value * 0.3 +
        self.feasibility * 0.2 +
        self.safety * 0.1
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum RiskLevel {
    Low,
    Medium,
    High,
    Critical,
}

impl RiskLevel {
    pub fn as_str(&self) -> &'static str {
        match self {
            RiskLevel::Low => "low",
            RiskLevel::Medium => "medium",
            RiskLevel::High => "high",
            RiskLevel::Critical => "critical",
        }
    }
}

/// Candidate generator for learning
pub struct CandidateGenerator {
    candidates: Arc<RwLock<Vec<Candidate>>>,
    history: Arc<RwLock<Vec<String>>>,
}

impl CandidateGenerator {
    /// Create a new candidate generator
    pub fn new() -> Self {
        Self {
            candidates: Arc::new(RwLock::new(Vec::new())),
            history: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Generate a new candidate
    pub async fn generate(
        &self,
        name: impl Into<String>,
        description: impl Into<String>,
        action_type: CandidateType,
    ) -> Candidate {
        let candidate = Candidate {
            id: Uuid::new_v4().to_string(),
            name: name.into(),
            description: description.into(),
            action_type,
            score: CandidateScore::new(0.5),
            prerequisites: Vec::new(),
            expected_outcome: String::new(),
            risk_level: RiskLevel::Medium,
            estimated_cost: 1.0,
            created_at: chrono::Utc::now(),
        };

        let mut candidates = self.candidates.write().await;
        candidates.push(candidate.clone());

        candidate
    }

    /// Add a candidate
    pub async fn add(&self, candidate: Candidate) {
        let mut candidates = self.candidates.write().await;
        candidates.push(candidate);
    }

    /// Get a candidate by ID
    pub async fn get(&self, id: &str) -> Option<Candidate> {
        let candidates = self.candidates.read().await;
        candidates.iter().find(|c| c.id == id).cloned()
    }

    /// List all candidates
    pub async fn list(&self) -> Vec<Candidate> {
        let candidates = self.candidates.read().await;
        candidates.clone()
    }

    /// Get top candidates by score
    pub async fn get_top(&self, limit: usize) -> Vec<Candidate> {
        let candidates = self.candidates.read().await;
        let mut sorted: Vec<_> = candidates.clone();
        sorted.sort_by(|a, b| {
            b.score.calculate_weighted()
                .partial_cmp(&a.score.calculate_weighted())
                .unwrap()
        });
        sorted.truncate(limit);
        sorted
    }

    /// Get candidates by type
    pub async fn get_by_type(&self, action_type: CandidateType) -> Vec<Candidate> {
        let candidates = self.candidates.read().await;
        candidates.iter()
            .filter(|c| c.action_type == action_type)
            .cloned()
            .collect()
    }

    /// Get low-risk candidates
    pub async fn get_low_risk(&self) -> Vec<Candidate> {
        let candidates = self.candidates.read().await;
        candidates.iter()
            .filter(|c| matches!(c.risk_level, RiskLevel::Low | RiskLevel::Medium))
            .cloned()
            .collect()
    }

    /// Update candidate score
    pub async fn update_score(&self, id: &str, score: CandidateScore) -> Result<()> {
        let mut candidates = self.candidates.write().await;
        if let Some(candidate) = candidates.iter_mut().find(|c| c.id == id) {
            candidate.score = score;
        }
        Ok(())
    }

    /// Mark candidate as selected
    pub async fn select(&self, id: &str) -> Result<()> {
        let mut history = self.history.write().await;
        history.push(id.to_string());
        Ok(())
    }

    /// Get selection history
    pub async fn get_history(&self) -> Vec<String> {
        let history = self.history.read().await;
        history.clone()
    }

    /// Remove a candidate
    pub async fn remove(&self, id: &str) -> Option<Candidate> {
        let mut candidates = self.candidates.write().await;
        let idx = candidates.iter().position(|c| c.id == id)?;
        Some(candidates.remove(idx))
    }

    /// Clear all candidates
    pub async fn clear(&self) {
        let mut candidates = self.candidates.write().await;
        candidates.clear();
    }

    /// Get statistics
    pub async fn stats(&self) -> CandidateStats {
        let candidates = self.candidates.read().await;
        let history = self.history.read().await;

        let mut by_type: std::collections::HashMap<CandidateType, usize> = std::collections::HashMap::new();
        for c in candidates.iter() {
            *by_type.entry(c.action_type).or_insert(0) += 1;
        }

        let avg_score = if candidates.is_empty() {
            0.0
        } else {
            candidates.iter()
                .map(|c| c.score.calculate_weighted())
                .sum::<f32>() / candidates.len() as f32
        };

        CandidateStats {
            total: candidates.len(),
            selected: history.len(),
            by_type,
            avg_score,
        }
    }
}

impl Default for CandidateGenerator {
    fn default() -> Self {
        Self::new()
    }
}

/// Candidate statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CandidateStats {
    pub total: usize,
    pub selected: usize,
    pub by_type: std::collections::HashMap<CandidateType, usize>,
    pub avg_score: f32,
}
