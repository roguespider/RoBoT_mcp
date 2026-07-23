// /src/experience/reflection/services/analyzer.rs
// Analyzes experiences to identify patterns, themes, and insights

use crate::experience::types::{Experience, ExperienceType};
use super::super::Reflection;

/// Analysis result containing detected patterns and themes
#[derive(Debug, Clone)]
pub struct AnalysisResult {
    pub patterns: Vec<String>,
    pub themes: Vec<String>,
    pub recommendations: Vec<String>,
    pub confidence_indicators: Vec<f32>,
}

/// Analyzes experiences to extract meaningful patterns and insights
pub struct ReflectionAnalyzer {
    min_confidence_threshold: f32,
}

impl ReflectionAnalyzer {
    /// Create a new analyzer with default settings
    pub fn new() -> Self {
        Self {
            min_confidence_threshold: 0.5,
        }
    }

    /// Create an analyzer with custom threshold
    pub fn with_threshold(threshold: f32) -> Self {
        Self {
            min_confidence_threshold: threshold,
        }
    }

    /// Analyze a collection of experiences for patterns
    pub fn analyze_experiences(&self, experiences: &[Experience]) -> AnalysisResult {
        let mut patterns = Vec::new();
        let mut themes = Vec::new();
        let mut recommendations = Vec::new();
        let mut confidence_sum = 0.0f32;

        // Group by experience type
        let mut type_counts: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
        for exp in experiences {
            *type_counts.entry(Self::experience_type_name(&exp.experience_type)).or_insert(0) += 1;
            confidence_sum += exp.confidence;
        }

        // Detect type patterns
        for (type_name, count) in &type_counts {
            if *count > 3 {
                patterns.push(format!("frequent_{}: {} occurrences", type_name.to_lowercase(), count));
            }
        }

        // Detect theme patterns from tags
        let mut tag_counts: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
        for exp in experiences {
            for tag in &exp.tags {
                *tag_counts.entry(tag.clone()).or_insert(0) += 1;
            }
        }

        for (tag, count) in &tag_counts {
            if *count > 2 {
                themes.push(format!("{}: {} occurrences", tag, count));
            }
        }

        // Generate recommendations based on patterns
        if let Some((most_common, _)) = type_counts.iter().max_by_key(|(_, v)| *v) {
            recommendations.push(format!(
                "Consider exploring different {} scenarios to avoid overfitting",
                most_common
            ));
        }

        // Calculate average confidence
        let avg_confidence = if experiences.is_empty() {
            0.0
        } else {
            confidence_sum / experiences.len() as f32
        };

        AnalysisResult {
            patterns,
            themes,
            recommendations,
            confidence_indicators: vec![avg_confidence],
        }
    }

    /// Analyze a single reflection for quality
    pub fn analyze_reflection(&self, reflection: &Reflection) -> ReflectionQuality {
        let indicators = ReflectionQualityIndicators {
            has_description: !reflection.description.is_empty(),
            has_summary: !reflection.summary.is_empty(),
            experience_count: reflection.experience_ids.len(),
            confidence_score: reflection.confidence.score,
            is_actionable: reflection.confidence.score >= 0.7,
        };

        let overall_score = self.calculate_quality_score(&indicators);

        ReflectionQuality {
            indicators: indicators.clone(),
            overall_score,
            suggestions: self.generate_suggestions(&indicators),
        }
    }

    fn experience_type_name(et: &ExperienceType) -> String {
        match et {
            ExperienceType::ToolExecution => "tool_execution",
            ExperienceType::MemoryLookup => "memory_lookup",
            ExperienceType::MemoryStore => "memory_store",
            ExperienceType::Workflow => "workflow",
            ExperienceType::Planning => "planning",
            ExperienceType::Exploration => "exploration",
            ExperienceType::Hypothesis => "hypothesis",
            ExperienceType::Reflection => "reflection",
            ExperienceType::Learning => "learning",
            ExperienceType::Conversation => "conversation",
            ExperienceType::UserFeedback => "user_feedback",
            ExperienceType::ModelInference => "model_inference",
            ExperienceType::Error => "error",
            ExperienceType::System => "system",
            ExperienceType::Custom(s) => s.as_str(),
        }.to_string()
    }

    fn calculate_quality_score(&self, indicators: &ReflectionQualityIndicators) -> f32 {
        let mut score = 0.0;

        if indicators.has_description {
            score += 0.2;
        }
        if indicators.has_summary {
            score += 0.2;
        }
        if indicators.experience_count > 0 {
            score += 0.2;
        }
        if indicators.confidence_score > 0.5 {
            score += 0.2;
        }
        if indicators.is_actionable {
            score += 0.2;
        }

        score
    }

    fn generate_suggestions(&self, indicators: &ReflectionQualityIndicators) -> Vec<String> {
        let mut suggestions = Vec::new();

        if !indicators.has_description {
            suggestions.push("Add a detailed description explaining the reasoning".to_string());
        }
        if !indicators.has_summary {
            suggestions.push("Include a brief summary of key findings".to_string());
        }
        if indicators.experience_count < 2 {
            suggestions.push("Consider referencing more experiences for stronger evidence".to_string());
        }
        if indicators.confidence_score < 0.5 {
            suggestions.push("Confidence is low - seek additional supporting evidence".to_string());
        }

        suggestions
    }
}

impl Default for ReflectionAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

/// Quality indicators for a reflection
#[derive(Debug, Clone)]
pub struct ReflectionQualityIndicators {
    pub has_description: bool,
    pub has_summary: bool,
    pub experience_count: usize,
    pub confidence_score: f32,
    pub is_actionable: bool,
}

/// Overall reflection quality assessment
#[derive(Debug, Clone)]
pub struct ReflectionQuality {
    pub indicators: ReflectionQualityIndicators,
    pub overall_score: f32,
    pub suggestions: Vec<String>,
}
