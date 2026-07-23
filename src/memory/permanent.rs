// src/memory/permanent.rs
//! Permanent Memory - Per Architecture §6.3
//!
//! Permanent Memory contains curated knowledge retained after evaluation.
//!
//! Characteristics:
//! - Indexed
//! - Connected
//! - Confidence weighted
//! - Relationship aware

#![allow(dead_code)]

use std::collections::HashMap;
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use uuid::Uuid;

use super::types::{MemoryItem, MemoryStatus, MemoryType};

/// Permanent memory statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermanentMemoryStats {
    pub total_items: usize,
    pub by_type: HashMap<String, usize>,
    pub avg_confidence: f32,
    pub avg_importance: f32,
}

/// Permanent Memory - Per Architecture §6.3
///
/// Curated knowledge retained after evaluation.
/// Characteristics: Indexed, connected, confidence weighted, relationship aware.
pub struct PermanentMemory {
    /// In-memory cache for permanent memory items
    cache: Arc<RwLock<HashMap<Uuid, MemoryItem>>>,
    
    /// Index by type
    type_index: Arc<RwLock<HashMap<MemoryType, Vec<Uuid>>>>,
    
    /// Index by tag
    tag_index: Arc<RwLock<HashMap<String, Vec<Uuid>>>>,
    
    /// Maximum cached items
    max_cache_size: usize,
}

impl PermanentMemory {
    /// Create a new permanent memory
    pub fn new(max_cache_size: usize) -> Self {
        Self {
            cache: Arc::new(RwLock::new(HashMap::new())),
            type_index: Arc::new(RwLock::new(HashMap::new())),
            tag_index: Arc::new(RwLock::new(HashMap::new())),
            max_cache_size,
        }
    }

    /// Store an item in permanent memory
    pub async fn store(&self, item: MemoryItem) -> Uuid {
        let id = item.id;
        
        // Store in cache
        let mut cache = self.cache.write().await;
        cache.insert(id, item.clone());
        
        // Update type index
        {
            let mut type_index = self.type_index.write().await;
            type_index
                .entry(item.memory_type)
                .or_insert_with(Vec::new)
                .push(id);
        }
        
        // Update tag index
        for tag in &item.tags {
            let mut tag_index = self.tag_index.write().await;
            tag_index
                .entry(tag.clone())
                .or_insert_with(Vec::new)
                .push(id);
        }
        
        id
    }

    /// Retrieve an item from permanent memory
    pub async fn retrieve(&self, id: &Uuid) -> Option<MemoryItem> {
        let mut cache = self.cache.write().await;
        if let Some(item) = cache.get_mut(id) {
            item.record_access();
            Some(item.clone())
        } else {
            None
        }
    }

    /// Find items by type
    pub async fn find_by_type(&self, memory_type: MemoryType) -> Vec<MemoryItem> {
        let type_index = self.type_index.read().await;
        let cache = self.cache.read().await;
        
        type_index
            .get(&memory_type)
            .map(|ids| {
                ids.iter()
                    .filter_map(|id| cache.get(id).cloned())
                    .filter(|item| item.status == MemoryStatus::Active)
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Find items by tag
    pub async fn find_by_tag(&self, tag: &str) -> Vec<MemoryItem> {
        let tag_index = self.tag_index.read().await;
        let cache = self.cache.read().await;
        
        tag_index
            .get(tag)
            .map(|ids| {
                ids.iter()
                    .filter_map(|id| cache.get(id).cloned())
                    .filter(|item| item.status == MemoryStatus::Active)
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Search permanent memory by content
    pub async fn search(&self, query: &str) -> Vec<MemoryItem> {
        let query_lower = query.to_lowercase();
        let cache = self.cache.read().await;
        cache
            .values()
            .filter(|item| {
                item.status == MemoryStatus::Active && 
                item.content.to_lowercase().contains(&query_lower)
            })
            .cloned()
            .collect()
    }

    /// Find high-confidence items
    pub async fn find_confident(&self, min_confidence: f32) -> Vec<MemoryItem> {
        let cache = self.cache.read().await;
        cache
            .values()
            .filter(|item| {
                item.status == MemoryStatus::Active && 
                item.confidence >= min_confidence
            })
            .cloned()
            .collect()
    }

    /// Get related items
    pub async fn get_related(&self, id: &Uuid) -> Vec<MemoryItem> {
        let cache = self.cache.read().await;
        
        if let Some(item) = cache.get(id) {
            item.related_ids
                .iter()
                .filter_map(|rid| cache.get(rid).cloned())
                .collect()
        } else {
            Vec::new()
        }
    }

    /// Update item confidence
    pub async fn update_confidence(&self, id: &Uuid, confidence: f32) -> bool {
        let mut cache = self.cache.write().await;
        if let Some(item) = cache.get_mut(id) {
            item.update_confidence(confidence);
            true
        } else {
            false
        }
    }

    /// Archive an item (historical data is never destroyed - per architecture)
    pub async fn archive(&self, id: &Uuid) -> bool {
        let mut cache = self.cache.write().await;
        if let Some(item) = cache.get_mut(id) {
            item.archive();
            true
        } else {
            false
        }
    }

    /// Get all active items
    pub async fn get_all(&self) -> Vec<MemoryItem> {
        let cache = self.cache.read().await;
        cache
            .values()
            .filter(|item| item.status == MemoryStatus::Active)
            .cloned()
            .collect()
    }

    /// Get statistics
    pub async fn stats(&self) -> PermanentMemoryStats {
        let cache = self.cache.read().await;
        let mut by_type = HashMap::new();
        let mut total_confidence = 0.0;
        let mut total_importance = 0.0;
        let mut count = 0;
        
        for item in cache.values().filter(|i| i.status == MemoryStatus::Active) {
            *by_type.entry(item.memory_type.to_string()).or_insert(0) += 1;
            total_confidence += item.confidence;
            total_importance += item.importance;
            count += 1;
        }
        
        PermanentMemoryStats {
            total_items: cache.len(),
            by_type,
            avg_confidence: if count > 0 { total_confidence / count as f32 } else { 0.0 },
            avg_importance: if count > 0 { total_importance / count as f32 } else { 0.0 },
        }
    }

    /// Clear all items
    pub async fn clear(&self) {
        let mut cache = self.cache.write().await;
        let mut type_index = self.type_index.write().await;
        let mut tag_index = self.tag_index.write().await;
        
        cache.clear();
        type_index.clear();
        tag_index.clear();
    }
}

impl Default for PermanentMemory {
    fn default() -> Self {
        Self::new(10000)
    }
}

#[cfg(test)]
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
}
