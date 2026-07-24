// /src/experience/reflection/services/generator.rs
// Generates reflections from experiences
#![allow(dead_code)]

use uuid::Uuid;

use super::super::{Reflection, ReflectionType};
use crate::experience::types::Experience;

/// Generates reflections from collections of experiences
pub struct ReflectionGenerator {
    min_experiences: usize,
    auto_validate_threshold: f32,
}

impl ReflectionGenerator {
    /// Create a new generator with default settings
    pub fn new() -> Self {
        Self {
            min_experiences: 2,
            auto_validate_threshold: 0.8,
        }
    }

    /// Create a generator with custom minimum experience count
    pub fn with_min_experiences(min: usize) -> Self {
        Self {
            min_experiences: min,
            auto_validate_threshold: 0.8,
        }
    }

    /// Generate a reflection from multiple experiences
    pub fn generate_from_experiences(
        &self,
        experiences: &[Experience],
        title: impl Into<String>,
    ) -> Option<Reflection> {
        if experiences.len() < self.min_experiences {
            return None;
        }

        let reflection_type = self.infer_reflection_type(experiences);
        let mut reflection = Reflection::new(Uuid::new_v4().to_string(), reflection_type, title);

        // Add all experiences to the reflection
        for exp in experiences {
            reflection.add_experience(exp.id.to_string());
        }

        // Calculate confidence based on experiences
        let avg_confidence: f32 =
            experiences.iter().map(|e| e.confidence).sum::<f32>() / experiences.len() as f32;

        reflection.set_confidence(avg_confidence);

        // Auto-validate if threshold is met
        if avg_confidence >= self.auto_validate_threshold {
            reflection.validate();
        }

        // Generate summary from experience titles
        let summary = experiences
            .iter()
            .map(|e| e.title.as_str())
            .collect::<Vec<_>>()
            .join(", ");
        reflection.summary = summary;

        // Generate description from outcomes
        let descriptions: Vec<String> = experiences
            .iter()
            .filter_map(|e| e.outcome.message.clone())
            .collect();
        reflection.description = descriptions.join(" ");

        Some(reflection)
    }

    /// Generate a reflection from a single experience
    pub fn generate_from_single(
        &self,
        experience: &Experience,
        title: impl Into<String>,
    ) -> Reflection {
        let reflection_type = self.infer_single_reflection_type(experience);
        let mut reflection = Reflection::new(Uuid::new_v4().to_string(), reflection_type, title);

        reflection.add_experience(experience.id.to_string());
        reflection.set_confidence(experience.confidence);
        reflection.summary = experience.title.clone();
        reflection.description = experience.description.clone();

        if let Some(msg) = &experience.outcome.message {
            reflection.description = msg.clone();
        }

        reflection
    }

    /// Infer the reflection type from a collection of experiences
    fn infer_reflection_type(&self, experiences: &[Experience]) -> ReflectionType {
        // Count outcome kinds
        let success_count = experiences
            .iter()
            .filter(|e| {
                matches!(
                    e.outcome.kind,
                    crate::experience::types::OutcomeKind::Success
                        | crate::experience::types::OutcomeKind::Partial
                )
            })
            .count();

        let failure_count = experiences.len() - success_count;
        let success_ratio = success_count as f32 / experiences.len() as f32;

        if failure_count > success_count {
            ReflectionType::Failure
        } else if success_ratio > 0.8 {
            ReflectionType::Success
        } else if success_ratio > 0.5 {
            ReflectionType::Improvement
        } else {
            ReflectionType::General
        }
    }

    /// Infer reflection type from a single experience
    fn infer_single_reflection_type(&self, experience: &Experience) -> ReflectionType {
        match experience.experience_type {
            crate::experience::types::ExperienceType::Error => ReflectionType::Failure,
            crate::experience::types::ExperienceType::Reflection => ReflectionType::Pattern,
            _ => match experience.outcome.kind {
                crate::experience::types::OutcomeKind::Success => ReflectionType::Success,
                crate::experience::types::OutcomeKind::Failure => ReflectionType::Failure,
                _ => ReflectionType::General,
            },
        }
    }
}

impl Default for ReflectionGenerator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::experience::types::{ExperienceOutcome, ExperienceType, OutcomeKind};

    fn create_test_experience(outcome_kind: OutcomeKind) -> Experience {
        let mut exp = Experience::new(
            "Test experience".to_string(),
            "Test description".to_string(),
            ExperienceType::ToolExecution,
            vec![], // observations
        );
        exp.outcome = ExperienceOutcome {
            kind: outcome_kind,
            message: None,
            error: None,
            duration_ms: None,
        };
        exp.confidence = 0.8;
        exp.evidence_count = 1;
        exp.tags = vec!["test".to_string()];
        exp
    }

    #[test]
    fn test_generate_from_multiple_successes() {
        let generator = ReflectionGenerator::new();
        let experiences = vec![
            create_test_experience(OutcomeKind::Success),
            create_test_experience(OutcomeKind::Success),
        ];

        let reflection = generator.generate_from_experiences(&experiences, "All successful");
        assert!(reflection.is_some());
        let r = reflection.expect("Reflection should be generated for test data");
        assert_eq!(r.reflection_type, ReflectionType::Success);
        assert_eq!(r.experience_ids.len(), 2);
    }

    #[test]
    fn test_generate_from_failures() {
        let generator = ReflectionGenerator::new();
        let experiences = vec![
            create_test_experience(OutcomeKind::Failure),
            create_test_experience(OutcomeKind::Failure),
        ];

        let reflection = generator.generate_from_experiences(&experiences, "All failures");
        assert!(reflection.is_some());
        let r = reflection.expect("Reflection should be generated for test data");
        assert_eq!(r.reflection_type, ReflectionType::Failure);
    }

    #[test]
    fn test_requires_min_experiences() {
        let generator = ReflectionGenerator::new();
        let experiences = vec![create_test_experience(OutcomeKind::Success)];

        let reflection = generator.generate_from_experiences(&experiences, "Only one");
        assert!(reflection.is_none());
    }
}
