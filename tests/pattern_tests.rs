mod tests {
    use super::*;
    use crate::experience::types::ExperienceType;

    fn create_test_experience(title: &str, tags: Vec<&str>) -> Experience {
        let mut exp = Experience::new(
            title.to_string(),
            "Test description".to_string(),
            ExperienceType::ToolExecution,
            vec![], // observations
        );
        exp.confidence = 0.8;
        exp.tags = tags.into_iter().map(String::from).collect();
        exp
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
        // Action includes first 3 words of title
        assert_eq!(pattern.action, "File read operation");
        assert!(pattern.common_tags.contains(&"file".to_string()));
        assert_eq!(pattern.success_rate, 1.0);
    }
