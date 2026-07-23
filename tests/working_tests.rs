mod tests {
    use super::*;
    use crate::memory::types::MemoryLayer;

    #[tokio::test]
    async fn test_store_and_retrieve() {
        let memory = WorkingMemory::new(100);
        let item = MemoryItem::new(
            MemoryLayer::Working,
            MemoryType::Context,
            "Test content".to_string(),
            "test".to_string(),
        );
        
        let id = memory.store(item).await;
        let retrieved = memory.retrieve(&id).await;
        
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().content, "Test content");
    }

    #[tokio::test]
    async fn test_search() {
        let memory = WorkingMemory::new(100);
        
        let item1 = MemoryItem::new(
            MemoryLayer::Working,
            MemoryType::Context,
            "The quick brown fox".to_string(),
            "test".to_string(),
        );
        let item2 = MemoryItem::new(
            MemoryLayer::Working,
            MemoryType::Context,
            "Jumps over the lazy dog".to_string(),
            "test".to_string(),
        );
        
        memory.store(item1).await;
        memory.store(item2).await;
        
        let results = memory.search("quick").await;
        assert_eq!(results.len(), 1);
        assert!(results[0].content.contains("quick"));
    }

    #[tokio::test]
    async fn test_eviction() {
        let memory = WorkingMemory::new(5);
        
        for i in 0..10 {
            let item = MemoryItem::new(
                MemoryLayer::Working,
                MemoryType::Context,
                format!("Item {}", i),
                "test".to_string(),
            );
            memory.store(item).await;
        }
        
        let stats = memory.stats().await;
        assert!(stats.total_items <= 5);
    }
