mod tests {
    use super::*;
    use crate::experience::types::{ExperienceOutcome, OutcomeKind, ExperienceType};

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
            duration_ms: None 
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
