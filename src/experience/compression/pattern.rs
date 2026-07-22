// src/experience/compression/pattern.rs
//! Pattern detection for experience compression

use std::collections::HashMap;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::experience::types::Experience;

/// A detected pattern across multiple experiences
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pattern {
    /// Unique identifier for this pattern
    pub id: Uuid,
    
    /// Common action/workflow name
    pub action: String,
    
    /// Common keywords extracted
    pub keywords: Vec<String>,
    
    /// Common tags across experiences
    pub common_tags: Vec<String>,
    
    /// Experience type this pattern applies to
    pub experience_type: String,
    
    /// Success rate across experiences
    pub success_rate: f32,
    
    /// Pattern confidence (0.0 to 1.0)
    pub confidence: f32,
    
    /// When pattern was detected
    pub detected_at: DateTime<Utc>,
}

impl Pattern {
    /// Create a new pattern
    pub fn new(action: String, experience_type: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            action,
            keywords: Vec::new(),
            common_tags: Vec::new(),
            experience_type,
            success_rate: 0.0,
            confidence: 0.0,
            detected_at: Utc::now(),
        }
    }

    /// Add a keyword to the pattern
    pub fn add_keyword(&mut self, keyword: String) {
        if !self.keywords.contains(&keyword) {
            self.keywords.push(keyword);
        }
    }

    /// Set the common tags
    pub fn set_common_tags(&mut self, tags: Vec<String>) {
        self.common_tags = tags;
    }

    /// Set the success rate
    pub fn set_success_rate(&mut self, rate: f32) {
        self.success_rate = rate.clamp(0.0, 1.0);
    }

    /// Generate a human-readable summary
    pub fn to_summary(&self) -> String {
        let mut summary = format!("Pattern: {}", self.action);
        
        if !self.keywords.is_empty() {
            summary.push_str(&format!(" (keywords: {})", self.keywords.join(", ")));
        }
        
        if !self.common_tags.is_empty() {
            summary.push_str(&format!(" [tags: {}]", self.common_tags.join(", ")));
        }
        
        summary
    }
}

/// Match information between an experience and a pattern
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternMatch {
    /// The matched pattern
    pub pattern_id: Uuid,
    
    /// Match score (0.0 to 1.0)
    pub score: f32,
    
    /// Matching keywords
    pub matched_keywords: Vec<String>,
    
    /// Missing keywords
    pub missing_keywords: Vec<String>,
}

impl PatternMatch {
    /// Create a new pattern match
    pub fn new(pattern_id: Uuid, score: f32) -> Self {
        Self {
            pattern_id,
            score,
            matched_keywords: Vec::new(),
            missing_keywords: Vec::new(),
        }
    }
}

/// Pattern detector for finding commonalities across experiences
pub struct PatternDetector {
    /// Minimum keyword frequency to be considered common
    min_keyword_frequency: f32,
}

impl PatternDetector {
    /// Create a new pattern detector
    pub fn new() -> Self {
        Self {
            min_keyword_frequency: 0.5,
        }
    }

    /// Detect the common pattern across experiences
    pub fn detect_pattern(&self, experiences: &[Experience]) -> Option<Pattern> {
        if experiences.is_empty() {
            return None;
        }

        // Extract common elements
        let experience_type = self.extract_common_type(experiences)?;
        let common_tags = self.extract_common_tags(experiences)?;
        let keywords = self.extract_keywords(experiences)?;
        let success_rate = self.calculate_success_rate(experiences);
        let action = self.extract_action(experiences)?;

        let confidence = self.calculate_pattern_confidence(experiences, &common_tags, &keywords);

        let mut pattern = Pattern::new(action, experience_type);
        pattern.set_common_tags(common_tags);
        pattern.set_success_rate(success_rate);
        
        for keyword in keywords {
            pattern.add_keyword(keyword);
        }
        
        pattern.confidence = confidence;

        Some(pattern)
    }

    /// Extract the common experience type
    fn extract_common_type(&self, experiences: &[Experience]) -> Option<String> {
        let mut type_counts: HashMap<String, usize> = HashMap::new();
        
        for exp in experiences {
            let type_name = format!("{:?}", exp.experience_type);
            *type_counts.entry(type_name).or_insert(0) += 1;
        }
        
        type_counts.into_iter()
            .max_by_key(|(_, count)| *count)
            .map(|(type_name, _)| type_name)
    }

    /// Extract tags that appear in multiple experiences
    fn extract_common_tags(&self, experiences: &[Experience]) -> Option<Vec<String>> {
        let mut tag_counts: HashMap<String, usize> = HashMap::new();
        
        for exp in experiences {
            for tag in &exp.tags {
                *tag_counts.entry(tag.clone()).or_insert(0) += 1;
            }
        }
        
        let threshold = (experiences.len() as f32 * self.min_keyword_frequency) as usize;
        
        let common: Vec<String> = tag_counts.iter()
            .filter(|(_, count)| **count >= threshold)
            .map(|(tag, _)| tag.clone())
            .collect();
        
        if common.is_empty() && experiences.len() >= 3 {
            // Fall back to most common tag
            tag_counts.iter()
                .max_by_key(|(_, count)| *count)
                .map(|(_, _)| common.clone())
        } else {
            Some(common)
        }
    }

    /// Extract keywords from titles and descriptions
    fn extract_keywords(&self, experiences: &[Experience]) -> Option<Vec<String>> {
        let mut word_counts: HashMap<String, usize> = HashMap::new();
        
        for exp in experiences {
            // Extract words from title and description
            let text = format!("{} {}", exp.title, exp.description);
            let words: Vec<String> = text.to_lowercase()
                .split(|c: char| !c.is_alphanumeric())
                .filter(|w| w.len() > 3)
                .map(String::from)
                .collect();
            
            for word in words {
                *word_counts.entry(word).or_insert(0) += 1;
            }
        }
        
        let threshold = (experiences.len() as f32 * self.min_keyword_frequency) as usize;
        
        let keywords: Vec<String> = word_counts.into_iter()
            .filter(|(_, count)| *count >= threshold)
            .map(|(word, _)| word)
            .take(10) // Limit to top 10 keywords
            .collect();
        
        Some(keywords)
    }

    /// Extract common action from titles
    fn extract_action(&self, experiences: &[Experience]) -> Option<String> {
        if experiences.is_empty() {
            return None;
        }

        // Use the most common first word(s) as the action
        let mut action_counts: HashMap<String, usize> = HashMap::new();
        
        for exp in &experiences[0..experiences.len().min(10)] {
            let words: Vec<&str> = exp.title.split_whitespace().take(3).collect();
            if !words.is_empty() {
                let action = words.join(" ");
                *action_counts.entry(action).or_insert(0) += 1;
            }
        }
        
        action_counts.into_iter()
            .max_by_key(|(_, count)| *count)
            .map(|(action, _)| action)
    }

    /// Calculate the success rate across experiences
    fn calculate_success_rate(&self, experiences: &[Experience]) -> f32 {
        if experiences.is_empty() {
            return 0.0;
        }

        use crate::experience::types::OutcomeKind;
        let successes = experiences.iter()
            .filter(|e| matches!(e.outcome.kind, OutcomeKind::Success))
            .count();

        successes as f32 / experiences.len() as f32
    }

    /// Calculate pattern confidence based on consistency
    fn calculate_pattern_confidence(
        &self,
        experiences: &[Experience],
        common_tags: &[String],
        keywords: &[String],
    ) -> f32 {
        if experiences.is_empty() {
            return 0.0;
        }

        // Tag consistency (how many experiences share common tags)
        let tag_score = if common_tags.is_empty() {
            0.5
        } else {
            let experiences_with_tags = experiences.iter()
                .filter(|e| e.tags.iter().any(|t| common_tags.contains(t)))
                .count();
            experiences_with_tags as f32 / experiences.len() as f32
        };

        // Keyword consistency
        let keyword_score = if keywords.is_empty() {
            0.5_f32
        } else {
            // Calculate based on keyword distribution
            let total_keywords = experiences.len() * keywords.len();
            if total_keywords == 0 {
                0.5_f32
            } else {
                // Rough estimate based on keyword presence
                0.7_f32.min(0.3 + (keywords.len() as f32 / 10.0))
            }
        };

        // Outcome consistency
        let success_rate = self.calculate_success_rate(experiences);
        let outcome_score = 0.5 + (success_rate * 0.5);

        // Weighted average
        (tag_score * 0.3 + keyword_score * 0.3 + outcome_score * 0.4).min(1.0)
    }

    /// Calculate how much an experience deviates from a pattern
    pub fn calculate_deviation(&self, experience: &Experience, pattern: &Pattern) -> f32 {
        let mut deviation = 0.0;
        let mut factors = 0;

        // Check tag deviation
        if !pattern.common_tags.is_empty() {
            let has_common_tag = experience.tags.iter()
                .any(|t| pattern.common_tags.contains(t));
            if !has_common_tag {
                deviation += 0.3;
            }
            factors += 1;
        }

        // Check keyword deviation in title
        let title_lower = experience.title.to_lowercase();
        let keyword_matches = pattern.keywords.iter()
            .filter(|k| title_lower.contains(k.as_str()))
            .count();
        
        if !pattern.keywords.is_empty() {
            let keyword_ratio = keyword_matches as f32 / pattern.keywords.len() as f32;
            deviation += (1.0 - keyword_ratio) * 0.3;
            factors += 1;
        }

        // Check experience type deviation
        let type_name = format!("{:?}", experience.experience_type);
        if type_name != pattern.experience_type {
            deviation += 0.2;
        }
        factors += 1;

        if factors == 0 {
            0.0
        } else {
            (deviation / factors as f32).min(1.0)
        }
    }
}

impl Default for PatternDetector {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_experience(title: &str, tags: Vec<&str>) -> Experience {
        Experience {
            id: Uuid::new_v4(),
            timestamp: Utc::now(),
            experience_type: ExperienceType::ToolExecution,
            title: title.to_string(),
            description: "Test description".to_string(),
            context: crate::experience::types::ExperienceContext::default(),
            outcome: crate::experience::types::ExperienceOutcome::Success,
            score: None,
            encounter_ids: vec![],
            maturity: crate::experience::types::KnowledgeMaturity::Emerging,
            confidence: 0.8,
            lessons: vec![],
            evidence_count: 0,
            tags: tags.into_iter().map(String::from).collect(),
            metadata: HashMap::new(),
        }
    }

    #[test]
    fn test_detect_pattern_with_similar_experiences() {
        let detector = PatternDetector::new();
        let experiences = vec![
            create_test_experience("File read operation", vec!["file", "read"]),
            create_test_experience("File read operation", vec!["file", "read"]),
            create_test_experience("File read operation", vec!["file", "read"]),
        ];
        
        let pattern = detector.detect_pattern(&experiences);
        assert!(pattern.is_some());
        
        let pattern = pattern.expect("Pattern detection should succeed for test data");
        assert_eq!(pattern.action, "File read");
        assert!(pattern.common_tags.contains(&"file".to_string()));
        assert_eq!(pattern.success_rate, 1.0);
    }
}
