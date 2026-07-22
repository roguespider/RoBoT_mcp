// /src/experience/reflection/engine.rs
// The main Reflection Engine that orchestrates all reflection services

use chrono::Utc;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

use anyhow::Result;

use super::insight::Insight;
use super::pattern::{Pattern, PatternType};
use super::{Reflection, ReflectionStatus, ReflectionType};
use super::services::analyzer::ReflectionAnalyzer;
use super::services::generator::ReflectionGenerator;
use super::services::repository::ReflectionRepository;
use super::services::validator::ReflectionValidator;

/// Configuration for the reflection engine
#[derive(Debug, Clone)]
pub struct ReflectionEngineConfig {
    /// Minimum experiences before auto-generating reflection
    pub min_experiences_for_auto_reflection: usize,
    
    /// Minimum confidence for valid reflection
    pub min_confidence: f32,
    
    /// Auto-validate reflections above this confidence
    pub auto_validate_threshold: f32,
    
    /// Maximum reflections to keep in memory
    pub max_cached_reflections: usize,
}

impl Default for ReflectionEngineConfig {
    fn default() -> Self {
        Self {
            min_experiences_for_auto_reflection: 3,
            min_confidence: 0.5,
            auto_validate_threshold: 0.8,
            max_cached_reflections: 1000,
        }
    }
}

/// Main reflection engine that orchestrates all reflection services
pub struct ReflectionEngine {
    config: ReflectionEngineConfig,
    analyzer: Arc<ReflectionAnalyzer>,
    generator: Arc<ReflectionGenerator>,
    repository: Arc<ReflectionRepository>,
    validator: Arc<ReflectionValidator>,
    insights: Arc<RwLock<HashMap<String, Insight>>>,
    patterns: Arc<RwLock<HashMap<String, Pattern>>>,
}

impl ReflectionEngine {
    /// Create a new reflection engine with default settings
    pub fn new() -> Self {
        Self::with_config(ReflectionEngineConfig::default())
    }

    /// Create with custom configuration
    pub fn with_config(config: ReflectionEngineConfig) -> Self {
        Self {
            config: config.clone(),
            analyzer: Arc::new(ReflectionAnalyzer::with_threshold(config.min_confidence)),
            generator: Arc::new(ReflectionGenerator::with_min_experiences(
                config.min_experiences_for_auto_reflection,
            )),
            repository: Arc::new(ReflectionRepository::new()),
            validator: Arc::new(ReflectionValidator::new()),
            insights: Arc::new(RwLock::new(HashMap::new())),
            patterns: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Generate a reflection from a collection of experiences
    pub async fn generate_reflection(
        &self,
        experiences: &[crate::experience::types::Experience],
        title: impl Into<String>,
    ) -> Result<Option<Reflection>> {
        let mut reflection = self.generator.generate_from_experiences(experiences, title);
        
        if let Some(ref r) = reflection {
            // Validate the reflection
            let validation = self.validator.validate(r);
            
            if !validation.is_valid {
                tracing::warn!(
                    "Reflection validation failed: {:?}",
                    validation.issues
                );
            }
            
            // Auto-validate if threshold met
            if validation.score >= self.config.auto_validate_threshold {
                if let Some(ref mut r) = reflection {
                    r.validate();
                }
            }
            
            // Save to repository
            if let Some(ref r) = reflection {
                self.repository.save(r.clone())?;
                tracing::info!("Generated reflection: {}", r.id);
            }
        }
        
        Ok(reflection)
    }

    /// Generate a reflection from a single experience
    pub async fn generate_from_single(
        &self,
        experience: &crate::experience::types::Experience,
        title: impl Into<String>,
    ) -> Result<Reflection> {
        let mut reflection = self.generator.generate_from_single(experience, title);
        
        // Validate
        let validation = self.validator.validate(&reflection);
        if validation.score >= self.config.auto_validate_threshold {
            reflection.validate();
        }
        
        // Save
        self.repository.save(reflection.clone())?;
        
        Ok(reflection)
    }

    /// Analyze experiences and detect patterns
    pub async fn analyze_experiences(
        &self,
        experiences: &[crate::experience::types::Experience],
    ) -> Result<AnalysisReport> {
        // Use analyzer to find patterns and themes
        let analysis = self.analyzer.analyze_experiences(experiences);
        
        // Store detected patterns
        for pattern_name in &analysis.patterns {
            let pattern = Pattern::with_type(pattern_name.clone(), PatternType::Frequency);
            self.patterns.write().await.insert(pattern.id.clone(), pattern);
        }
        
        Ok(AnalysisReport {
            patterns: analysis.patterns,
            themes: analysis.themes,
            recommendations: analysis.recommendations,
            confidence: analysis.confidence_indicators.first().copied().unwrap_or(0.0),
        })
    }

    /// Validate a reflection
    pub async fn validate_reflection(&self, reflection: &Reflection) -> Result<ValidationReport> {
        let result = self.validator.validate(reflection);
        let quality = self.analyzer.analyze_reflection(reflection);
        
        Ok(ValidationReport {
            is_valid: result.is_valid,
            score: result.score,
            issues: result.issues.iter().map(|i| i.message.clone()).collect(),
            quality_score: quality.overall_score,
            suggestions: quality.suggestions,
        })
    }

    /// Create an insight from reflections
    pub async fn create_insight(
        &self,
        title: impl Into<String>,
        statement: impl Into<String>,
        reflection_ids: Vec<String>,
    ) -> Result<Insight> {
        let mut insight = Insight::new(
            Uuid::new_v4().to_string(),
            title,
            statement,
            super::insight::InsightType::General,
        );
        
        for rid in &reflection_ids {
            insight.add_reflection(rid);
        }
        
        self.insights.write().await.insert(insight.id.clone(), insight.clone());
        
        tracing::info!("Created insight: {}", insight.id);
        Ok(insight)
    }

    /// Get an insight by ID
    pub async fn get_insight(&self, id: &str) -> Option<Insight> {
        self.insights.read().await.get(id).cloned()
    }

    /// Get all insights
    pub async fn get_all_insights(&self) -> Vec<Insight> {
        self.insights.read().await.values().cloned().collect()
    }

    /// Get trusted insights (ready to influence behavior)
    pub async fn get_trusted_insights(&self) -> Vec<Insight> {
        self.insights
            .read()
            .await
            .values()
            .filter(|i| i.is_trusted())
            .cloned()
            .collect()
    }

    /// Add evidence to an insight
    pub async fn confirm_insight(&self, id: &str) -> Result<()> {
        if let Some(insight) = self.insights.write().await.get_mut(id) {
            insight.confirm();
        }
        Ok(())
    }

    /// Add contradiction to an insight
    pub async fn contradict_insight(&self, id: &str) -> Result<()> {
        if let Some(insight) = self.insights.write().await.get_mut(id) {
            insight.contradict();
        }
        Ok(())
    }

    /// Get a pattern by ID
    pub async fn get_pattern(&self, id: &str) -> Option<Pattern> {
        self.patterns.read().await.get(id).cloned()
    }

    /// Get all patterns
    pub async fn get_all_patterns(&self) -> Vec<Pattern> {
        self.patterns.read().await.values().cloned().collect()
    }

    /// Update pattern confidence
    pub async fn update_pattern_confidence(&self, id: &str, delta: f32) -> Result<()> {
        if let Some(pattern) = self.patterns.write().await.get_mut(id) {
            pattern.confidence = (pattern.confidence + delta).clamp(0.0, 1.0);
            pattern.last_updated = Utc::now();
        }
        Ok(())
    }

    /// Get a reflection by ID
    pub async fn get_reflection(&self, id: &str) -> Option<Reflection> {
        self.repository.get(id).ok().flatten()
    }

    /// List all reflections
    pub async fn list_reflections(&self) -> Vec<Reflection> {
        self.repository.list_all().unwrap_or_default()
    }

    /// List reflections by type
    pub async fn list_by_type(&self, reflection_type: ReflectionType) -> Vec<Reflection> {
        self.repository.list_by_type(reflection_type).unwrap_or_default()
    }

    /// List validated reflections
    pub async fn list_validated(&self) -> Vec<Reflection> {
        self.repository.list_validated(self.config.min_confidence).unwrap_or_default()
    }

    /// Search reflections
    pub async fn search(&self, query: &str) -> Vec<Reflection> {
        self.repository.search_by_title(query).unwrap_or_default()
    }

    /// Delete a reflection
    pub async fn delete_reflection(&self, id: &str) -> Result<()> {
        self.repository.delete(id)?;
        Ok(())
    }

    /// Archive old reflections
    pub async fn archive_old(&self, days: i64) -> Result<usize> {
        let cutoff = chrono::Utc::now() - chrono::Duration::days(days);
        let mut count = 0;
        
        let reflections = self.repository.list_all()?;
        for mut reflection in reflections {
            if reflection.metadata.updated_at < cutoff 
                && reflection.status == ReflectionStatus::Validated 
            {
                reflection.archive();
                self.repository.save(reflection)?;
                count += 1;
            }
        }
        
        tracing::info!("Archived {} old reflections", count);
        Ok(count)
    }

    /// Get engine statistics
    pub async fn get_stats(&self) -> EngineStats {
        let insights = self.insights.read().await;
        let patterns = self.patterns.read().await;
        
        EngineStats {
            total_reflections: self.repository.count().unwrap_or(0),
            total_insights: insights.len(),
            trusted_insights: insights.values().filter(|i| i.is_trusted()).count(),
            total_patterns: patterns.len(),
            mature_patterns: patterns.values().filter(|p| p.is_mature()).count(),
        }
    }
}

impl Default for ReflectionEngine {
    fn default() -> Self {
        Self::new()
    }
}

/// Report from analyzing experiences
#[derive(Debug, Clone)]
pub struct AnalysisReport {
    pub patterns: Vec<String>,
    pub themes: Vec<String>,
    pub recommendations: Vec<String>,
    pub confidence: f32,
}

/// Report from validating a reflection
#[derive(Debug, Clone)]
pub struct ValidationReport {
    pub is_valid: bool,
    pub score: f32,
    pub issues: Vec<String>,
    pub quality_score: f32,
    pub suggestions: Vec<String>,
}

/// Statistics about the reflection engine
#[derive(Debug)]
pub struct EngineStats {
    pub total_reflections: usize,
    pub total_insights: usize,
    pub trusted_insights: usize,
    pub total_patterns: usize,
    pub mature_patterns: usize,
}
