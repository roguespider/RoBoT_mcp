// src/learning/working_memory.rs
//! Working memory with state machine for active context

use std::sync::Arc;
use std::collections::HashMap;

use anyhow::Result;
use chrono::{DateTime, Utc, Duration};
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use uuid::Uuid;

pub mod memory_state;
pub mod promotion;

pub use memory_state::{MemoryState, StateTransition, StateTransitionRecord};
pub use promotion::{PromotionPolicy, PromotionEvaluation};

/// A piece of information in working memory with state machine support
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkingMemoryItem {
    pub id: String,
    pub key: String,
    pub value: String,
    pub item_type: MemoryItemType,
    pub importance: f32,
    pub confidence: f32,
    pub state: MemoryState,
    pub ttl_seconds: Option<u64>,
    pub created_at: DateTime<Utc>,
    pub accessed_at: DateTime<Utc>,
    pub access_count: u32,
    pub repeated_count: u32,
    pub confirmation_count: u32,
    pub contradicted: bool,
    pub transition_history: Vec<StateTransitionRecord>,
}

impl WorkingMemoryItem {
    /// Create a new working memory item
    pub fn new(
        key: String,
        value: String,
        item_type: MemoryItemType,
        importance: f32,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4().to_string(),
            key,
            value,
            item_type,
            importance,
            confidence: 0.5,
            state: MemoryState::Active,
            ttl_seconds: None,
            created_at: now,
            accessed_at: now,
            access_count: 1,
            repeated_count: 0,
            confirmation_count: 0,
            contradicted: false,
            transition_history: Vec::new(),
        }
    }
    
    /// Attempt a state transition
    pub fn transition(&mut self, transition: StateTransition, reason: Option<String>) -> bool {
        if !self.state.can_transition(&transition) {
            return false;
        }
        
        if let Some(new_state) = self.state.transition_to(&transition) {
            let record = StateTransitionRecord::new(
                self.state,
                new_state,
                transition,
                reason,
            );
            self.transition_history.push(record);
            self.state = new_state;
            return true;
        }
        
        false
    }
    
    /// Record an access (may trigger state transition)
    pub fn record_access(&mut self) {
        self.accessed_at = Utc::now();
        self.access_count += 1;
        
        // Repeated access tracking
        if self.state == MemoryState::Active {
            self.repeated_count += 1;
            if self.repeated_count > 1 {
                let _ = self.transition(StateTransition::Observe, Some("Repeated access".to_string()));
            }
        } else if self.state == MemoryState::Dormant {
            let _ = self.transition(StateTransition::Access, Some("Revived by access".to_string()));
        }
    }
    
    /// Record a confirmation
    pub fn record_confirmation(&mut self) {
        self.confirmation_count += 1;
        if self.state == MemoryState::Repeated {
            let _ = self.transition(StateTransition::Confirm, Some("Confirmed".to_string()));
        }
    }
    
    /// Record a contradiction
    pub fn record_contradiction(&mut self) {
        self.contradicted = true;
        if matches!(self.state, MemoryState::Active | MemoryState::Repeated | MemoryState::Confirmed) {
            let _ = self.transition(StateTransition::Contradict, Some("Contradicted".to_string()));
        }
    }
    
    /// Check if item has expired
    pub fn is_expired(&self) -> bool {
        if let Some(ttl) = self.ttl_seconds {
            let age = Utc::now() - self.created_at;
            return age > Duration::seconds(ttl as i64);
        }
        false
    }
    
    /// Check if item should be promoted based on policy
    pub fn should_promote(&self, policy: &PromotionPolicy) -> bool {
        let eval = policy.evaluate(self);
        eval.should_promote
    }
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

/// Working memory for active context with state machine
pub struct WorkingMemory {
    items: Arc<RwLock<HashMap<String, WorkingMemoryItem>>>,
    max_items: usize,
    policy: Arc<PromotionPolicy>,
}

impl WorkingMemory {
    /// Create new working memory with default policy
    pub fn new(max_items: usize) -> Self {
        Self::with_policy(max_items, PromotionPolicy::default())
    }
    
    /// Create new working memory with custom policy
    pub fn with_policy(max_items: usize, policy: PromotionPolicy) -> Self {
        Self {
            items: Arc::new(RwLock::new(HashMap::new())),
            max_items,
            policy: Arc::new(policy),
        }
    }
    
    /// Get the promotion policy
    pub fn policy(&self) -> &PromotionPolicy {
        &self.policy
    }
    
    /// Update the promotion policy
    pub fn set_policy(&self, policy: PromotionPolicy) {
        *Arc::make_mut(&mut self.policy.clone()) = policy;
    }

    /// Store an item in working memory
    pub async fn store(&self, key: impl Into<String>, value: impl Into<String>, item_type: MemoryItemType, importance: f32) -> Result<String> {
        let key_str = key.into();
        
        // Check if key already exists - update instead of create
        {
            let items = self.items.read().await;
            if items.contains_key(&key_str) {
                drop(items);
                return self.update(&key_str, value).await;
            }
        }
        
        let item = WorkingMemoryItem::new(key_str.clone(), value.into(), item_type, importance);

        let mut items = self.items.write().await;
        
        // Evict if at capacity
        if items.len() >= self.max_items {
            self.evict_low_importance(&mut items).await;
        }

        items.insert(key_str.clone(), item);

        Ok(key_str)
    }
    
    /// Update an existing item's value
    pub async fn update(&self, key: &str, value: impl Into<String>) -> Result<String> {
        let mut items = self.items.write().await;
        
        if let Some(item) = items.get_mut(key) {
            item.value = value.into();
            item.accessed_at = Utc::now();
            item.access_count += 1;
            Ok(item.id.clone())
        } else {
            anyhow::bail!("Item not found: {}", key)
        }
    }

    /// Retrieve an item by key
    pub async fn get(&self, key: &str) -> Option<WorkingMemoryItem> {
        let mut items = self.items.write().await;
        
        if let Some(item) = items.get_mut(key) {
            item.record_access();
            return Some(item.clone());
        }
        
        None
    }
    
    /// Get item without recording access
    pub async fn peek(&self, key: &str) -> Option<WorkingMemoryItem> {
        let items = self.items.read().await;
        items.get(key).cloned()
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
    
    /// Remove multiple items
    pub async fn remove_many(&self, keys: &[&str]) -> usize {
        let mut items = self.items.write().await;
        let mut removed = 0;
        for key in keys {
            if items.remove(*key).is_some() {
                removed += 1;
            }
        }
        removed
    }

    /// Clear all items of a type
    pub async fn clear_by_type(&self, item_type: MemoryItemType) {
        let mut items = self.items.write().await;
        items.retain(|_, item| item.item_type != item_type);
    }
    
    /// Clear all items in a state
    pub async fn clear_by_state(&self, state: MemoryState) {
        let mut items = self.items.write().await;
        items.retain(|_, item| item.state != state);
    }
    
    /// Clear expired items
    pub async fn clear_expired(&self) -> usize {
        let mut items = self.items.write().await;
        let initial_count = items.len();
        items.retain(|_, item| !item.is_expired());
        initial_count - items.len()
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
    
    /// List items by state
    pub async fn list_by_state(&self, state: MemoryState) -> Vec<WorkingMemoryItem> {
        let items = self.items.read().await;
        items.values()
            .filter(|i| i.state == state)
            .cloned()
            .collect()
    }
    
    /// Get items ready for promotion
    pub async fn get_promotable(&self) -> Vec<WorkingMemoryItem> {
        let items = self.items.read().await;
        items.values()
            .filter(|i| i.should_promote(&self.policy))
            .cloned()
            .collect()
    }

    /// Get most recently accessed items
    pub async fn get_recent(&self, limit: usize) -> Vec<WorkingMemoryItem> {
        let mut items: Vec<_> = {
            let items = self.items.read().await;
            items.values().cloned().collect()
        };
        
        items.sort_by_key(|b| std::cmp::Reverse(b.accessed_at));
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
        
        result.sort_by(|a, b| {
            b.importance.partial_cmp(&a.importance)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        result
    }
    
    /// Get items by key pattern
    pub async fn get_by_key_pattern(&self, pattern: &str) -> Vec<WorkingMemoryItem> {
        let items = self.items.read().await;
        let pattern_lower = pattern.to_lowercase();
        items.values()
            .filter(|i| i.key.to_lowercase().contains(&pattern_lower))
            .cloned()
            .collect()
    }
    
    /// Record confirmation for an item
    pub async fn confirm(&self, key: &str) -> bool {
        let mut items = self.items.write().await;
        if let Some(item) = items.get_mut(key) {
            item.record_confirmation();
            return true;
        }
        false
    }
    
    /// Record contradiction for an item
    pub async fn contradict(&self, key: &str) -> bool {
        let mut items = self.items.write().await;
        if let Some(item) = items.get_mut(key) {
            item.record_contradiction();
            return true;
        }
        false
    }
    
    /// Promote an item to long-term memory (returns the item if successful)
    pub async fn promote(&self, key: &str) -> Option<WorkingMemoryItem> {
        let mut items = self.items.write().await;
        if let Some(item) = items.get_mut(key) {
            if item.transition(StateTransition::Promote, Some("Manual promotion".to_string())) {
                item.confidence = self.policy.calculate_confidence(
                    item.access_count,
                    item.confirmation_count,
                );
                return Some(item.clone());
            }
        }
        None
    }
    
    /// Reject an item
    pub async fn reject(&self, key: &str) -> bool {
        let mut items = self.items.write().await;
        if let Some(item) = items.get_mut(key) {
            return item.transition(StateTransition::Reject, Some("Manual rejection".to_string()));
        }
        false
    }

    /// Update item importance
    pub async fn set_importance(&self, key: &str, importance: f32) -> bool {
        let mut items = self.items.write().await;
        if let Some(item) = items.get_mut(key) {
            item.importance = importance.clamp(0.0, 1.0);
            return true;
        }
        false
    }
    
    /// Set TTL for an item
    pub async fn set_ttl(&self, key: &str, ttl_seconds: Option<u64>) -> bool {
        let mut items = self.items.write().await;
        if let Some(item) = items.get_mut(key) {
            item.ttl_seconds = ttl_seconds;
            return true;
        }
        false
    }
    
    /// Get the state of an item
    pub async fn get_state(&self, key: &str) -> Option<MemoryState> {
        let items = self.items.read().await;
        items.get(key).map(|i| i.state)
    }
    
    /// Get transition history for an item
    pub async fn get_history(&self, key: &str) -> Option<Vec<StateTransitionRecord>> {
        let items = self.items.read().await;
        items.get(key).map(|i| i.transition_history.clone())
    }

    /// Evict lowest importance items to make room
    async fn evict_low_importance(&self, items: &mut HashMap<String, WorkingMemoryItem>) {
        let keys_to_remove: Vec<String> = {
            let mut sorted: Vec<_> = items.iter().collect();
            sorted.sort_by(|a, b| {
                a.1.importance.partial_cmp(&b.1.importance)
                    .unwrap_or(std::cmp::Ordering::Equal)
            });
            
            // Remove bottom 10%
            let to_remove = (items.len() / 10).max(1);
            sorted.into_iter().take(to_remove).map(|(k, _)| k.clone()).collect()
        };
        
        for key in keys_to_remove {
            items.remove(&key);
        }
    }
    
    /// Process all items - evaluate states and apply transitions
    pub async fn process_all(&self) -> usize {
        let mut items = self.items.write().await;
        let mut transitioned = 0;
        let now = Utc::now();
        
        for item in items.values_mut() {
            // Check TTL expiration
            if let Some(ttl) = item.ttl_seconds {
                let age = now - item.created_at;
                if age > Duration::seconds(ttl as i64)
                    && item.transition(StateTransition::Timeout, Some("TTL expired".to_string())) {
                        transitioned += 1;
                    }
            }
            
            // Apply promotion policy evaluation
            let eval = self.policy.evaluate(item);
            
            if eval.should_promote
                && item.transition(StateTransition::Promote, Some("Policy promotion".to_string())) {
                    item.confidence = self.policy.calculate_confidence(
                        item.access_count,
                        item.confirmation_count,
                    );
                    transitioned += 1;
                }
        }
        
        transitioned
    }

    /// Get statistics
    pub async fn stats(&self) -> MemoryStats {
        let items = self.items.read().await;
        
        let mut by_type: HashMap<MemoryItemType, usize> = HashMap::new();
        let mut by_state: HashMap<MemoryState, usize> = HashMap::new();
        
        for item in items.values() {
            *by_type.entry(item.item_type).or_insert(0) += 1;
            *by_state.entry(item.state).or_insert(0) += 1;
        }

        let avg_importance = if items.is_empty() {
            0.0
        } else {
            items.values().map(|i| i.importance).sum::<f32>() / items.len() as f32
        };
        
        let avg_confidence = if items.is_empty() {
            0.0
        } else {
            items.values().map(|i| i.confidence).sum::<f32>() / items.len() as f32
        };

        let total_accesses: u32 = items.values().map(|i| i.access_count).sum();
        let promotable: usize = items.values()
            .filter(|i| i.should_promote(&self.policy))
            .count();

        MemoryStats {
            total_items: items.len(),
            max_items: self.max_items,
            by_type,
            by_state,
            avg_importance,
            avg_confidence,
            total_accesses,
            promotable,
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
    pub by_state: HashMap<MemoryState, usize>,
    pub avg_importance: f32,
    pub avg_confidence: f32,
    pub total_accesses: u32,
    pub promotable: usize,
}
