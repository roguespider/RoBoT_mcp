// src/memory/working.rs
//! Working Memory - Per Architecture §6.3
//!
//! Working Memory contains temporary information used during active tasks.
//!
//! Characteristics:
//! - Short lifespan
//! - High volatility
//! - Context focused

#![allow(dead_code)]

use std::collections::HashMap;
use std::sync::Arc;

use chrono::{Duration, Utc};
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use uuid::Uuid;

use super::types::{MemoryItem, MemoryStatus, MemoryType};

/// Working memory statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkingMemoryStats {
    pub total_items: usize,
    pub active_items: usize,
    pub archived_items: usize,
    pub avg_access_count: f32,
}

/// Working memory - Per Architecture §6.3
///
/// Temporary information used during active tasks.
/// Characteristics: Short lifespan, high volatility, context focused.
pub struct WorkingMemory {
    /// In-memory storage for working memory items
    items: Arc<RwLock<HashMap<Uuid, MemoryItem>>>,
    
    /// Maximum items before eviction
    max_items: usize,
    
    /// Default TTL for items without explicit TTL
    default_ttl: Duration,
}

impl WorkingMemory {
    /// Create a new working memory
    pub fn new(max_items: usize) -> Self {
        Self {
            items: Arc::new(RwLock::new(HashMap::new())),
            max_items,
            default_ttl: Duration::minutes(30),
        }
    }

    /// Store an item in working memory
    pub async fn store(&self, item: MemoryItem) -> Uuid {
        let id = item.id;
        let mut items = self.items.write().await;
        
        // Evict old items if at capacity
        if items.len() >= self.max_items {
            self.evict_lru(&mut items).await;
        }
        
        items.insert(id, item);
        id
    }

    /// Retrieve an item from working memory
    pub async fn retrieve(&self, id: &Uuid) -> Option<MemoryItem> {
        let mut items = self.items.write().await;
        if let Some(item) = items.get_mut(id) {
            item.record_access();
            Some(item.clone())
        } else {
            None
        }
    }

    /// Find items by type
    pub async fn find_by_type(&self, memory_type: MemoryType) -> Vec<MemoryItem> {
        let items = self.items.read().await;
        items
            .values()
            .filter(|item| item.memory_type == memory_type && item.status == MemoryStatus::Active)
            .cloned()
            .collect()
    }

    /// Search working memory by content
    pub async fn search(&self, query: &str) -> Vec<MemoryItem> {
        let query_lower = query.to_lowercase();
        let items = self.items.read().await;
        items
            .values()
            .filter(|item| {
                item.status == MemoryStatus::Active && 
                item.content.to_lowercase().contains(&query_lower)
            })
            .cloned()
            .collect()
    }

    /// Get all active items
    pub async fn get_all(&self) -> Vec<MemoryItem> {
        let items = self.items.read().await;
        items
            .values()
            .filter(|item| item.status == MemoryStatus::Active)
            .cloned()
            .collect()
    }

    /// Archive an item (move to permanent memory conceptually)
    pub async fn archive(&self, id: &Uuid) -> bool {
        let mut items = self.items.write().await;
        if let Some(item) = items.get_mut(id) {
            item.archive();
            true
        } else {
            false
        }
    }

    /// Remove an item from working memory
    pub async fn remove(&self, id: &Uuid) -> bool {
        let mut items = self.items.write().await;
        items.remove(id).is_some()
    }

    /// Evict least recently used items
    async fn evict_lru(&self, items: &mut HashMap<Uuid, MemoryItem>) {
        // Collect IDs to remove (oldest accessed)
        let remove_count = (items.len() / 10).max(1);
        let mut sorted: Vec<_> = items
            .iter()
            .map(|(id, item)| (*id, item.accessed_at))
            .collect();
        sorted.sort_by_key(|(_, accessed)| *accessed);
        
        let ids_to_remove: Vec<Uuid> = sorted.into_iter().take(remove_count).map(|(id, _)| id).collect();
        
        for id in ids_to_remove {
            items.remove(&id);
        }
    }

    /// Clean up expired items
    pub async fn cleanup_expired(&self, max_age: Duration) -> usize {
        let cutoff = Utc::now() - max_age;
        let mut items = self.items.write().await;
        let initial_count = items.len();
        
        items.retain(|_, item| {
            item.accessed_at > cutoff || item.status == MemoryStatus::Active
        });
        
        initial_count - items.len()
    }

    /// Get statistics
    pub async fn stats(&self) -> WorkingMemoryStats {
        let items = self.items.read().await;
        let total = items.len();
        let active = items.values().filter(|i| i.status == MemoryStatus::Active).count();
        let archived = items.values().filter(|i| i.status == MemoryStatus::Archived).count();
        let avg_access = if total > 0 {
            items.values().map(|i| i.access_count as f32).sum::<f32>() / total as f32
        } else {
            0.0
        };
        
        WorkingMemoryStats {
            total_items: total,
            active_items: active,
            archived_items: archived,
            avg_access_count: avg_access,
        }
    }

    /// Clear all items
    pub async fn clear(&self) {
        let mut items = self.items.write().await;
        items.clear();
    }
}

impl Default for WorkingMemory {
    fn default() -> Self {
        Self::new(1000)
    }
}

#[cfg(test)]
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
}
