mod tests {
    use super::*;

    #[tokio::test]
    async fn test_add_and_get() {
        let store = KnowledgeStore::new(100);
        let item = KnowledgeItem::from_reflection("Test knowledge", 0.8, Uuid::new_v4());
        let id = store.add(item.clone()).await;
        
        let retrieved = store.get(id).await;
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().statement, "Test knowledge");
    }

    #[tokio::test]
    async fn test_get_mature() {
        let store = KnowledgeStore::new(100);
        
        // Add items with varying confidence
        let mut item1 = KnowledgeItem::from_reflection("Low confidence", 0.3, Uuid::new_v4());
        item1.status = KnowledgeStatus::Active;
        
        let mut item2 = KnowledgeItem::from_reflection("High confidence", 0.8, Uuid::new_v4());
        item2.status = KnowledgeStatus::Active;
        
        store.add(item1).await;
        let id2 = store.add(item2).await;
        
        let mature = store.get_mature().await;
        assert_eq!(mature.len(), 1);
        assert_eq!(mature[0].statement, "High confidence");
    }
