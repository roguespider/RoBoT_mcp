mod tests {
    use super::*;
    use crate::memory::types::MemoryLayer;

    #[tokio::test]
    async fn test_store_and_retrieve() {
        let memory = PermanentMemory::new(100);
        let mut item = MemoryItem::new(
            MemoryLayer::Permanent,
            MemoryType::Knowledge,
            "Important fact".to_string(),
            "test".to_string(),
        );
        item.add_tag("fact");
        item.add_tag("important");
        
        let id = memory.store(item).await;
        let retrieved = memory.retrieve(&id).await;
        
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().content, "Important fact");
    }

    #[tokio::test]
    async fn test_find_by_type() {
        let memory = PermanentMemory::new(100);
        
        let item1 = MemoryItem::new(
            MemoryLayer::Permanent,
            MemoryType::Knowledge,
            "Knowledge item".to_string(),
            "test".to_string(),
        );
        let item2 = MemoryItem::new(
            MemoryLayer::Permanent,
            MemoryType::Skill,
            "Skill item".to_string(),
            "test".to_string(),
        );
        
        memory.store(item1).await;
        memory.store(item2).await;
        
        let knowledge_items = memory.find_by_type(MemoryType::Knowledge).await;
        assert_eq!(knowledge_items.len(), 1);
        
        let skill_items = memory.find_by_type(MemoryType::Skill).await;
        assert_eq!(skill_items.len(), 1);
    }

    #[tokio::test]
    async fn test_find_by_tag() {
        let memory = PermanentMemory::new(100);
        
        let mut item = MemoryItem::new(
            MemoryLayer::Permanent,
            MemoryType::Knowledge,
            "Tagged item".to_string(),
            "test".to_string(),
        );
        item.add_tag("rust");
        item.add_tag("programming");
        
        memory.store(item).await;
        
        let rust_items = memory.find_by_tag("rust").await;
        assert_eq!(rust_items.len(), 1);
        
        let rust_items = memory.find_by_tag("programming").await;
        assert_eq!(rust_items.len(), 1);
    }

    #[tokio::test]
    async fn test_confidence_filtering() {
        let memory = PermanentMemory::new(100);
        
        let mut item1 = MemoryItem::new(
            MemoryLayer::Permanent,
            MemoryType::Knowledge,
            "High confidence".to_string(),
            "test".to_string(),
        );
        item1.update_confidence(0.9);
        
        let mut item2 = MemoryItem::new(
            MemoryLayer::Permanent,
            MemoryType::Knowledge,
            "Low confidence".to_string(),
            "test".to_string(),
        );
        item2.update_confidence(0.3);
        
        memory.store(item1).await;
        memory.store(item2).await;
        
        let confident_items = memory.find_confident(0.8).await;
        assert_eq!(confident_items.len(), 1);
        assert!(confident_items[0].content.contains("High"));
    }
