mod tests {
    use super::*;

    #[tokio::test]
    async fn test_retrieve_working() {
        let working = Arc::new(WorkingMemory::new(100));
        let permanent = Arc::new(PermanentMemory::new(100));
        let retrieval = MemoryRetrieval::new(working.clone(), permanent.clone());
        
        let item = MemoryItem::new(
            MemoryLayer::Working,
            MemoryType::Context,
            "Python is a great language".to_string(),
            "test".to_string(),
        );
        working.store(item).await;
        
        let results = retrieval.from_working("Python").await;
        assert_eq!(results.len(), 1);
        assert!(results[0].item.content.contains("Python"));
    }

    #[tokio::test]
    async fn test_retrieve_permanent() {
        let working = Arc::new(WorkingMemory::new(100));
        let permanent = Arc::new(PermanentMemory::new(100));
        let retrieval = MemoryRetrieval::new(working.clone(), permanent.clone());
        
        let mut item = MemoryItem::new(
            MemoryLayer::Permanent,
            MemoryType::Knowledge,
            "Rust is a systems language".to_string(),
            "test".to_string(),
        );
        item.add_tag("rust");
        permanent.store(item).await;
        
        let results = retrieval.from_permanent("Rust").await;
        assert_eq!(results.len(), 1);
        assert!(results[0].item.content.contains("Rust"));
    }

    #[tokio::test]
    async fn test_unified_retrieve() {
        let working = Arc::new(WorkingMemory::new(100));
        let permanent = Arc::new(PermanentMemory::new(100));
        let retrieval = MemoryRetrieval::new(working.clone(), permanent.clone());
        
        let item1 = MemoryItem::new(
            MemoryLayer::Working,
            MemoryType::Context,
            "Temporary context".to_string(),
            "test".to_string(),
        );
        working.store(item1).await;
        
        let item2 = MemoryItem::new(
            MemoryLayer::Permanent,
            MemoryType::Knowledge,
            "Knowledge about context".to_string(),
            "test".to_string(),
        );
        permanent.store(item2).await;
        
        let results = retrieval.retrieve("context").await;
        assert_eq!(results.len(), 2); // Both contain "context"
    }

    #[tokio::test]
    async fn test_confidence_filtering() {
        let working = Arc::new(WorkingMemory::new(100));
        let permanent = Arc::new(PermanentMemory::new(100));
        let retrieval = MemoryRetrieval::new(working.clone(), permanent.clone());
        
        let mut low_conf = MemoryItem::new(
            MemoryLayer::Permanent,
            MemoryType::Knowledge,
            "Low confidence item".to_string(),
            "test".to_string(),
        );
        low_conf.update_confidence(0.2);
        permanent.store(low_conf).await;
        
        let mut high_conf = MemoryItem::new(
            MemoryLayer::Permanent,
            MemoryType::Knowledge,
            "High confidence item".to_string(),
            "test".to_string(),
        );
        high_conf.update_confidence(0.9);
        permanent.store(high_conf).await;
        
        let query = RetrievalQuery {
            query: "item".to_string(),
            memory_types: Vec::new(),
            min_confidence: Some(0.5),
            tags: Vec::new(),
            limit: 10,
        };
        
        let results = retrieval.retrieve_with_query(&query).await;
        assert_eq!(results.len(), 1);
        assert!(results[0].item.content.contains("High"));
    }
