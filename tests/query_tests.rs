mod tests {
    use super::*;
    use super::super::types::KnowledgeSource;
    use uuid::Uuid;
    use chrono::Utc;

    fn make_test_item(statement: &str, confidence: f32, status: KnowledgeStatus) -> KnowledgeItem {
        KnowledgeItem {
            id: Uuid::new_v4(),
            statement: statement.to_string(),
            knowledge_type: KnowledgeType::Fact,
            confidence: super::super::types::KnowledgeConfidence::new(confidence),
            status,
            source: KnowledgeSource::User,
            supporting_evidence: vec![],
            contradicting_evidence: vec![],
            relations: vec![],
            created_at: Utc::now(),
            updated_at: Utc::now(),
            success_count: 0,
            failure_count: 0,
            tags: vec!["test".to_string()],
            metadata: std::collections::HashMap::new(),
        }
    }

    #[test]
    fn test_text_filter() {
        let items = vec![
            make_test_item("Rust is fast", 0.8, KnowledgeStatus::Active),
            make_test_item("Python is easy", 0.7, KnowledgeStatus::Active),
        ];
        
        let query = KnowledgeQuery {
            text: Some("rust".to_string()),
            ..Default::default()
        };
        
        let filtered = apply_query(&items, &query);
        assert_eq!(filtered.len(), 1);
        assert!(filtered[0].statement.contains("Rust"));
    }

    #[test]
    fn test_confidence_filter() {
        let items = vec![
            make_test_item("High confidence", 0.9, KnowledgeStatus::Active),
            make_test_item("Low confidence", 0.3, KnowledgeStatus::Active),
        ];
        
        let query = KnowledgeQuery {
            min_confidence: Some(0.7),
            ..Default::default()
        };
        
        let filtered = apply_query(&items, &query);
        assert_eq!(filtered.len(), 1);
    }

    #[test]
    fn test_ranking() {
        let items = vec![
            make_test_item("Low match", 0.3, KnowledgeStatus::Active),
            make_test_item("High match", 0.9, KnowledgeStatus::Active),
        ];
        
        let query = KnowledgeQuery {
            text: Some("match".to_string()),
            min_confidence: Some(0.2),
            ..Default::default()
        };
        
        let ranked = rank_items(items, &query);
        assert_eq!(ranked[0].statement, "High match");
    }
