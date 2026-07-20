// src/learning/working_memory.rs
//! Working memory for active context

use std::sync::Arc;
use std::collections::HashMap;

use anyhow::Result;
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use uuid::Uuid;

/// A piece of information in working memory
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkingMemoryItem {
    pub id: String,
    pub key: String,
    pub value: String,
    pub item_type: MemoryItemType,
    pub importance: f32,
    pub ttl_seconds: Option<u64>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub accessed_at: chrono::DateTime<chrono::Utc>,
    pub access_count: u32,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum MemoryItemType {
    Context,
    Task,
    Result,
    Error,
    Metadata,
    State,
}

/// Working memory for active context
pub struct WorkingMemory {
    items: Arc<RwLock<HashMap<String, WorkingMemoryItem>>>,
    max_items: usize,
}

impl WorkingMemory {
    /// Create new working memory
    pub fn new(max_items: usize) -> Self {
        Self {
            items: Arc::new(RwLock::new(HashMap::new())),
            max_items,
        }
    }

    /// Store an item in working memory
    pub async fn store(&self, key: impl Into<String>, value: impl Into<String>, item_type: MemoryItemType, importance: f32) -> Result<String> {
        let key_str = key.into();
        let id = Uuid::new_v4().to_string();
        let now = chrono::Utc::now();

        let item = WorkingMemoryItem {
            id: id.clone(),
            key: key_str.clone(),
            value: value.into(),
            item_type,
            importance,
            ttl_seconds: None,
            created_at: now,
            accessed_at: now,
            access_count: 0,
        };

        let mut items = self.items.write().await;
        
        // Evict if at capacity
        if items.len() >= self.max_items {
            self.evict_low_importance(&mut items).await;
        }

        items.insert(key_str, item);

        Ok(id)
    }

    /// Retrieve an item by key
    pub async fn get(&self, key: &str) -> Option<WorkingMemoryItem> {
        let mut items = self.items.write().await;
        
        if let Some(item) = items.get_mut(key) {
            item.accessed_at = chrono::Utc::now();
            item.access_count += 1;
            return Some(item.clone());
        }
        
        None
    }

    /// Check if key exists
    pub async fn contains(&self, key: &str) -> bool {
        let items = self.items.read().await;
        items.contains_key(key)
    }

    /// Remove an item
    pub async fn remove(&self, key: &str) -> Option<WorkingMemoryItem> {
        let mut items = self.items.write().await;
        items.remove(key)
    }

    /// Clear all items of a type
    pub async fn clear_by_type(&self, item_type: MemoryItemType) {
        let mut items = self.items.write().await;
        items.retain(|_, item| item.item_type != item_type);
    }

    /// Clear all items
    pub async fn clear(&self) {
        let mut items = self.items.write().await;
        items.clear();
    }

    /// List all items
    pub async fn list(&self) -> Vec<WorkingMemoryItem> {
        let items = self.items.read().await;
        items.values().cloned().collect()
    }

    /// List items by type
    pub async fn list_by_type(&self, item_type: MemoryItemType) -> Vec<WorkingMemoryItem> {
        let items = self.items.read().await;
        items.values()
            .filter(|i| i.item_type == item_type)
            .cloned()
            .collect()
    }

    /// Get most recently accessed items
    pub async fn get_recent(&self, limit: usize) -> Vec<WorkingMemoryItem> {
        let mut items: Vec<_> = {
            let items = self.items.read().await;
            items.values().cloned().collect()
        };
        
        items.sort_by(|a, b| b.accessed_at.cmp(&a.accessed_at));
        items.truncate(limit);
        items
    }

    /// Get most important items
    pub async fn get_important(&self, threshold: f32) -> Vec<WorkingMemoryItem> {
        let items = self.items.read().await;
        let mut result: Vec<_> = items.values()
            .filter(|i| i.importance >= threshold)
            .cloned()
            .collect();
        
        result.sort_by(|a, b| b.importance.partial_cmp(&a.importance).unwrap());
        result
    }

    /// Update item importance
    pub async fn set_importance(&self, key: &str, importance: f32) -> Option<()> {
        let mut items = self.items.write().await;
        if let Some(item) = items.get_mut(key) {
            item.importance = importance;
            return Some(());
        }
        None
    }

    /// Evict lowest importance items to make room
    async fn evict_low_importance(&self, items: &mut HashMap<String, WorkingMemoryItem>) {
        let keys_to_remove: Vec<String> = {
            let mut sorted: Vec<_> = items.iter().collect();
            sorted.sort_by(|a, b| a.1.importance.partial_cmp(&b.1.importance).unwrap());
            
            // Remove bottom 10%
            let to_remove = (items.len() / 10).max(1);
            sorted.into_iter().take(to_remove).map(|(k, _)| k.clone()).collect()
        };
        
        for key in keys_to_remove {
            items.remove(&key);
        }
    }

    /// Get statistics
    pub async fn stats(&self) -> MemoryStats {
        let items = self.items.read().await;
        
        let mut by_type: HashMap<MemoryItemType, usize> = HashMap::new();
        for item in items.values() {
            *by_type.entry(item.item_type).or_insert(0) += 1;
        }

        let avg_importance = if items.is_empty() {
            0.0
        } else {
            items.values().map(|i| i.importance).sum::<f32>() / items.len() as f32
        };

        let total_accesses: u32 = items.values().map(|i| i.access_count).sum();

        MemoryStats {
            total_items: items.len(),
            max_items: self.max_items,
            by_type,
            avg_importance,
            total_accesses,
        }
    }
}

impl Default for WorkingMemory {
    fn default() -> Self {
        Self::new(1000)
    }
}

/// Statistics about working memory
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryStats {
    pub total_items: usize,
    pub max_items: usize,
    pub by_type: HashMap<MemoryItemType, usize>,
    pub avg_importance: f32,
    pub total_accesses: u32,
}
