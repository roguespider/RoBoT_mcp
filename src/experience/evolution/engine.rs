// /src/experience/evolution/engine.rs
// The main engine that transforms insights into behaviors

use anyhow::Result;
use chrono::Utc;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

use super::behavior::{Behavior, BehaviorAction, BehaviorPriority, BehaviorStatus};
use super::evidence::{EvolutionEvidence, EvidenceVerdict};
use crate::experience::reflection::insight::Insight;

/// Configuration for the evolution engine
#[derive(Debug, Clone)]
pub struct EvolutionConfig {
    /// Minimum applications before promotion
    pub min_applications_for_promotion: u32,
    
    /// Minimum confidence before promotion
    pub min_confidence_for_promotion: f32,
    
    /// Failure rate threshold for deprecation
    pub failure_threshold: f32,
    
    /// Days unused before deprecation
    pub unused_threshold_days: i64,
    
    /// Applications before practice phase
    pub applications_before_practice: u32,
    
    /// Applications before integration
    pub applications_before_integration: u32,
}

impl Default for EvolutionConfig {
    fn default() -> Self {
        Self {
            min_applications_for_promotion: 5,
            min_confidence_for_promotion: 0.7,
            failure_threshold: 0.5,
            unused_threshold_days: 30,
            applications_before_practice: 10,
            applications_before_integration: 20,
        }
    }
}

/// Trait for evolution engine implementations
pub trait EvolutionEngineTrait: Send + Sync {
    /// Create a behavior from an insight
    fn create_behavior_from_insight(&self, insight: &Insight) -> impl std::future::Future<Output = Result<Behavior>> + Send;
    
    /// Record a behavior application result
    fn record_result(&self, behavior_id: &str, success: bool) -> impl std::future::Future<Output = Result<()>> + Send;
    
    /// Get active behaviors for a context
    fn get_active_behaviors(&self, context: &str) -> impl std::future::Future<Output = Vec<Behavior>> + Send;
}

/// The evolution engine transforms insights into behaviors
pub struct EvolutionEngine {
    behaviors: Arc<RwLock<HashMap<String, Behavior>>>,
    evidence: Arc<RwLock<HashMap<String, Vec<EvolutionEvidence>>>>,
    config: EvolutionConfig,
}

impl EvolutionEngine {
    /// Create a new evolution engine
    pub fn new() -> Self {
        Self {
            behaviors: Arc::new(RwLock::new(HashMap::new())),
            evidence: Arc::new(RwLock::new(HashMap::new())),
            config: EvolutionConfig::default(),
        }
    }

    /// Create with custom configuration
    pub fn with_config(config: EvolutionConfig) -> Self {
        Self {
            behaviors: Arc::new(RwLock::new(HashMap::new())),
            evidence: Arc::new(RwLock::new(HashMap::new())),
            config,
        }
    }

    /// Create a behavior from an insight
    pub async fn create_behavior_from_insight(&self, insight: &Insight) -> Result<Behavior> {
        let behavior = Behavior::new(
            Uuid::new_v4().to_string(),
            format!("Behavior from insight: {}", insight.title),
            insight.statement.clone(),
            BehaviorAction::ApplyHeuristic {
                rule: insight.statement.clone(),
                priority: 50,
            },
        );

        let mut behaviors = self.behaviors.write().await;
        let behavior_id = behavior.id.clone();
        behaviors.insert(behavior_id.clone(), behavior.clone());

        // Add evidence linking to insight
        let evidence = EvolutionEvidence::supporting(
            Uuid::new_v4().to_string(),
            &behavior_id,
            super::evidence::EvidenceType::Observation,
            format!("Derived from insight: {}", insight.title),
        );
        
        let mut evidence_store = self.evidence.write().await;
        evidence_store.entry(behavior_id).or_insert_with(Vec::new).push(evidence);

        tracing::info!("Created behavior from insight: {}", insight.id);
        Ok(behavior)
    }

    /// Get a behavior by ID
    pub async fn get_behavior(&self, id: &str) -> Option<Behavior> {
        let behaviors = self.behaviors.read().await;
        behaviors.get(id).cloned()
    }

    /// List all behaviors
    pub async fn list_behaviors(&self) -> Vec<Behavior> {
        let behaviors = self.behaviors.read().await;
        behaviors.values().cloned().collect()
    }

    /// List active behaviors sorted by priority
    pub async fn list_active_behaviors(&self) -> Vec<Behavior> {
        let behaviors = self.behaviors.read().await;
        let mut active: Vec<_> = behaviors
            .values()
            .filter(|b| b.status == BehaviorStatus::Active || b.status == BehaviorStatus::Practicing)
            .cloned()
            .collect();
        active.sort_by_key(|b| std::cmp::Reverse(b.priority));
        active
    }

    /// Create a behavior directly
    pub async fn create_behavior(
        &self,
        name: impl Into<String>,
        description: impl Into<String>,
        action: BehaviorAction,
    ) -> Result<Behavior> {
        let behavior = Behavior::new(
            Uuid::new_v4().to_string(),
            name,
            description,
            action,
        );
        
        let mut behaviors = self.behaviors.write().await;
        let behavior_id = behavior.id.clone();
        behaviors.insert(behavior_id.clone(), behavior.clone());
        
        tracing::info!("Created behavior: {}", behavior_id);
        Ok(behavior)
    }

    /// Record application result
    pub async fn record_result(&self, behavior_id: &str, success: bool) -> Result<()> {
        let mut behaviors = self.behaviors.write().await;
        if let Some(behavior) = behaviors.get_mut(behavior_id) {
            if success {
                behavior.record_success();
                
                // Check for promotion to practicing
                if behavior.status == BehaviorStatus::Active 
                    && behavior.application_count >= self.config.applications_before_practice
                {
                    behavior.start_practicing();
                    tracing::info!("Behavior {} promoted to practicing", behavior_id);
                }
            } else {
                behavior.record_failure();
                
                // Check for deprecation
                if behavior.should_deprecate(self.config.failure_threshold, self.config.unused_threshold_days) {
                    behavior.deprecate();
                    tracing::warn!("Behavior {} deprecated due to failures", behavior_id);
                }
            }
            
            // Check for promotion from candidate
            if behavior.is_ready_for_promotion(
                self.config.min_applications_for_promotion,
                self.config.min_confidence_for_promotion,
            ) && behavior.status == BehaviorStatus::Candidate
            {
                behavior.activate();
                tracing::info!("Behavior {} promoted to active", behavior_id);
            }
            
            // Check for integration from practicing
            if behavior.status == BehaviorStatus::Practicing 
                && behavior.application_count >= self.config.applications_before_integration
                && behavior.confidence >= 0.9
            {
                behavior.integrate();
                tracing::info!("Behavior {} integrated", behavior_id);
            }
        }
        Ok(())
    }

    /// Add evidence for a behavior
    pub async fn add_evidence(&self, evidence: EvolutionEvidence) -> Result<()> {
        let mut evidence_store = self.evidence.write().await;
        evidence_store
            .entry(evidence.behavior_id.clone())
            .or_insert_with(Vec::new)
            .push(evidence);
        Ok(())
    }

    /// Get evidence for a behavior
    pub async fn get_evidence(&self, behavior_id: &str) -> Vec<EvolutionEvidence> {
        let evidence_store = self.evidence.read().await;
        evidence_store.get(behavior_id).cloned().unwrap_or_default()
    }

    /// Evaluate all behaviors and apply maintenance
    pub async fn evaluate_and_maintain(&self) -> Result<EvaluationSummary> {
        let mut summary = EvaluationSummary::default();
        let mut behaviors = self.behaviors.write().await;

        for behavior in behaviors.values_mut() {
            // Check deprecation conditions
            if behavior.should_deprecate(self.config.failure_threshold, self.config.unused_threshold_days)
                && behavior.status != BehaviorStatus::Deprecated {
                    behavior.deprecate();
                    summary.deprecated += 1;
                }

            // Check promotion conditions
            if behavior.status == BehaviorStatus::Candidate 
                && behavior.is_ready_for_promotion(
                    self.config.min_applications_for_promotion,
                    self.config.min_confidence_for_promotion,
                ) 
            {
                behavior.activate();
                summary.promoted += 1;
            }

            // Check integration conditions (high confidence + many applications)
            if behavior.status == BehaviorStatus::Practicing 
                && behavior.application_count >= 20 
                && behavior.confidence >= 0.9 
            {
                behavior.integrate();
                summary.integrated += 1;
            }
        }

        summary.total_behaviors = behaviors.len();
        Ok(summary)
    }

    /// Get behavior suggestions based on context
    pub async fn suggest_behaviors(&self, context: &str) -> Vec<Behavior> {
        let active = self.list_active_behaviors().await;
        let context_lower = context.to_lowercase();
        
        active
            .into_iter()
            .filter(|b| {
                b.name.to_lowercase().contains(&context_lower)
                    || b.description.to_lowercase().contains(&context_lower)
            })
            .take(5)
            .collect()
    }

    /// Calculate overall evolution metrics
    pub async fn get_metrics(&self) -> EvolutionMetrics {
        let behaviors = self.behaviors.read().await;
        let evidence_store = self.evidence.read().await;
        
        let total = behaviors.len();
        let by_status: HashMap<_, _> = behaviors
            .values()
            .fold(HashMap::new(), |mut acc, b| {
                *acc.entry(b.status).or_insert(0) += 1;
                acc
            });

        let total_evidence: usize = evidence_store.values().map(|v| v.len()).sum();
        let supporting_evidence: usize = evidence_store
            .values()
            .flat_map(|v| v.iter())
            .filter(|e| e.verdict == EvidenceVerdict::Supports)
            .count();

        let avg_confidence = if total > 0 {
            behaviors.values().map(|b| b.confidence).sum::<f32>() / total as f32
        } else {
            0.0
        };

        EvolutionMetrics {
            total_behaviors: total,
            behaviors_by_status: by_status,
            total_evidence,
            supporting_evidence,
            average_confidence: avg_confidence,
        }
    }

    /// Get integrated behaviors (fully learned)
    pub async fn get_integrated_behaviors(&self) -> Vec<Behavior> {
        let behaviors = self.behaviors.read().await;
        behaviors
            .values()
            .filter(|b| b.status == BehaviorStatus::Integrated)
            .cloned()
            .collect()
    }

    /// Get deprecated behaviors
    pub async fn get_deprecated_behaviors(&self) -> Vec<Behavior> {
        let behaviors = self.behaviors.read().await;
        behaviors
            .values()
            .filter(|b| b.status == BehaviorStatus::Deprecated)
            .cloned()
            .collect()
    }

    /// Update behavior priority
    pub async fn update_priority(&self, behavior_id: &str, priority: BehaviorPriority) -> Result<()> {
        if let Some(behavior) = self.behaviors.write().await.get_mut(behavior_id) {
            behavior.priority = priority;
            behavior.updated_at = Utc::now();
        }
        Ok(())
    }

    /// Archive deprecated behaviors
    pub async fn archive_deprecated(&self) -> Result<usize> {
        let mut count = 0;
        let mut behaviors = self.behaviors.write().await;
        
        for behavior in behaviors.values_mut() {
            if behavior.status == BehaviorStatus::Deprecated {
                behavior.status = BehaviorStatus::Deprecated;
                count += 1;
            }
        }
        
        tracing::info!("Archived {} deprecated behaviors", count);
        Ok(count)
    }

    /// Merge similar behaviors
    pub async fn merge_behaviors(&self, source_id: &str, target_id: &str) -> Result<()> {
        let mut behaviors = self.behaviors.write().await;
        
        let source = behaviors.remove(source_id);
        if let Some(source) = source {
            if let Some(target) = behaviors.get_mut(target_id) {
                // Transfer evidence from source to target
                if let Some(evidence) = self.evidence.read().await.get(source_id) {
                    let mut evidence_store = self.evidence.write().await;
                    evidence_store
                        .entry(target_id.to_string())
                        .or_insert_with(Vec::new)
                        .extend(evidence.clone());
                }
                
                // Transfer applications
                target.application_count += source.application_count;
                target.success_count += source.success_count;
                target.updated_at = Utc::now();
                
                // Recalculate confidence
                if target.application_count > 0 {
                    target.confidence = target.success_count as f32 / target.application_count as f32;
                }
                
                tracing::info!("Merged behavior {} into {}", source_id, target_id);
            }
        }
        
        Ok(())
    }

    /// Get behavior effectiveness score
    pub async fn get_effectiveness(&self, behavior_id: &str) -> Option<f32> {
        self.behaviors.read().await
            .get(behavior_id)
            .map(|b| b.success_rate())
    }

    /// Check if a behavior should be recommended
    pub async fn should_recommend(&self, behavior_id: &str) -> bool {
        if let Some(behavior) = self.behaviors.read().await.get(behavior_id) {
            match behavior.status {
                BehaviorStatus::Active | BehaviorStatus::Practicing | BehaviorStatus::Integrated => {
                    behavior.confidence >= 0.6 && !behavior.should_deprecate(0.5, 30)
                }
                _ => false,
            }
        } else {
            false
        }
    }
}

impl Default for EvolutionEngine {
    fn default() -> Self {
        Self::new()
    }
}

/// Summary of evaluation results
#[derive(Debug, Default)]
pub struct EvaluationSummary {
    pub total_behaviors: usize,
    pub promoted: usize,
    pub deprecated: usize,
    pub integrated: usize,
}

/// Metrics about the evolution system
#[derive(Debug)]
pub struct EvolutionMetrics {
    pub total_behaviors: usize,
    pub behaviors_by_status: HashMap<BehaviorStatus, usize>,
    pub total_evidence: usize,
    pub supporting_evidence: usize,
    pub average_confidence: f32,
}
