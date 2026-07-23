mod tests {
    use super::*;
    use crate::experience::types::ExperienceType;

    fn create_test_experience(id: Uuid, title: &str, confidence: f32, tags: Vec<&str>) -> Experience {
        let mut exp = Experience::new(
            title.to_string(),
            "Test description".to_string(),
            ExperienceType::ToolExecution,
            vec![], // observations
        );
        exp.id = id;
        exp.confidence = confidence;
        exp.tags = tags.into_iter().map(String::from).collect();
        exp
    }

    #[test]
    fn test_compress_single_experience_returns_none() {
        let compressor = ExperienceCompressor::new();
        let experiences = vec![create_test_experience(Uuid::new_v4(), "Test", 0.8, vec![])];
        
        let result = compressor.compress(&experiences);
        assert!(result.is_none());
    }

    #[test]
    fn test_compress_multiple_similar_experiences() {
        // Use lower threshold for test to ensure compression succeeds
        let compressor = ExperienceCompressor::with_config(3, 0.5);
        let experiences = vec![
            create_test_experience(Uuid::new_v4(), "File read operation", 0.8, vec!["file", "read"]),
            create_test_experience(Uuid::new_v4(), "File read operation", 0.85, vec!["file", "read"]),
            create_test_experience(Uuid::new_v4(), "File read operation", 0.9, vec!["file", "read"]),
        ];
        
        let result = compressor.compress(&experiences);
        assert!(result.is_some());
        
        let compressed = result.expect("Compression should succeed for test data");
        assert_eq!(compressed.experience_count, 3);
        assert!((compressed.confidence - 0.85).abs() < 0.01);
    }
