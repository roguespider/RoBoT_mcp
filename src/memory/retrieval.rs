// src/memory/retrieval.rs
//! Memory Retrieval - Per Architecture §6.3
//!
//! Provides retrieval capabilities for memory items across both
//! working and permanent memory layers.

use std::sync::Arc;

use super::types::{MemoryItem, MemoryLayer, MemoryType};
use super::working::WorkingMemory;
use super::permanent::PermanentMemory;

/// Memory retrieval result with source information
#[derive(Debug, Clone)]
pub struct RetrievalResult {
    pub item: MemoryItem,
    pub relevance_score: f32,
    pub source_layer: MemoryLayer,
}

/// Query parameters for memory retrieval
#[derive(Debug, Clone)]
pub struct RetrievalQuery {
    pub query: String,
    pub memory_types: Vec<MemoryType>,
    pub min_confidence: Option<f32>,
    pub tags: Vec<String>,
    pub limit: usize,
}

impl Default for RetrievalQuery {
    fn default() -> Self {
        Self {
            query: String::new(),
            memory_types: Vec::new(),
            min_confidence: None,
            tags: Vec::new(),
            limit: 10,
        }
    }
}

/// Memory retrieval service - Per Architecture §6.3
///
/// Provides unified retrieval across working and permanent memory.
pub struct MemoryRetrieval {
    working: Arc<WorkingMemory>,
    permanent: Arc<PermanentMemory>,
}

impl MemoryRetrieval {
    /// Create a new memory retrieval service
    pub fn new(working: Arc<WorkingMemory>, permanent: Arc<PermanentMemory>) -> Self {
        Self { working, permanent }
    }

    /// Retrieve from working memory only
    pub async fn from_working(&self, query: &str) -> Vec<RetrievalResult> {
        let items = self.working.search(query).await;
        items
            .into_iter()
            .map(|item| RetrievalResult {
                relevance_score: self.calculate_relevance(&item, query),
                item,
                source_layer: MemoryLayer::Working,
            })
            .collect()
    }

    /// Retrieve from permanent memory only
    pub async fn from_permanent(&self, query: &str) -> Vec<RetrievalResult> {
        let items = self.permanent.search(query).await;
        items
            .into_iter()
            .map(|item| RetrievalResult {
                relevance_score: self.calculate_relevance(&item, query),
                item,
                source_layer: MemoryLayer::Permanent,
            })
            .collect()
    }

    /// Unified retrieval across all memory layers
    pub async fn retrieve(&self, query: &str) -> Vec<RetrievalResult> {
        let mut results = Vec::new();
        
        // Search working memory
        let working_results = self.from_working(query).await;
        results.extend(working_results);
        
        // Search permanent memory
        let permanent_results = self.from_permanent(query).await;
        results.extend(permanent_results);
        
        // Sort by relevance
        results.sort_by(|a, b| {
            b.relevance_score
                .partial_cmp(&a.relevance_score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        
        results
    }

    /// Retrieve with full query parameters
    pub async fn retrieve_with_query(&self, query: &RetrievalQuery) -> Vec<RetrievalResult> {
        let mut results = self.retrieve(&query.query).await;
        
        // Filter by type
        if !query.memory_types.is_empty() {
            results.retain(|r| query.memory_types.contains(&r.item.memory_type));
        }
        
        // Filter by confidence
        if let Some(min_conf) = query.min_confidence {
            results.retain(|r| r.item.confidence >= min_conf);
        }
        
        // Filter by tags
        if !query.tags.is_empty() {
            results.retain(|r| {
                r.item.tags.iter().any(|t| query.tags.contains(t))
            });
        }
        
        // Limit results
        results.truncate(query.limit);
        
        results
    }

    /// Get context from memory (recent working items)
    pub async fn get_context(&self, limit: usize) -> Vec<MemoryItem> {
        let mut items = self.working.get_all().await;
        items.sort_by(|a, b| b.accessed_at.cmp(&a.accessed_at));
        items.truncate(limit);
        items
    }

    /// Get related memories
    pub async fn get_related(&self, memory_id: &uuid::Uuid) -> Vec<MemoryItem> {
        self.permanent.get_related(memory_id).await
    }

    /// Calculate relevance score for a memory item
    fn calculate_relevance(&self, item: &MemoryItem, query: &str) -> f32 {
        let query_lower = query.to_lowercase();
        let content_lower = item.content.to_lowercase();
        
        // Base score from content match
        let content_match = if content_lower.contains(&query_lower) {
            1.0
        } else {
            0.0
        };
        
        // Confidence contribution
        let confidence_score = item.confidence;
        
        // Importance contribution
        let importance_score = item.importance;
        
        // Access recency (more recent = higher score)
        let now = chrono::Utc::now();
        let age_hours = (now - item.accessed_at).num_hours() as f32;
        let recency_score = (1.0 / (1.0 + age_hours / 24.0)).min(1.0);
        
        // Weighted combination
        (content_match * 0.4) + 
        (confidence_score * 0.2) + 
        (importance_score * 0.2) + 
        (recency_score * 0.2)
    }

    /// Get statistics from both memory layers
    pub async fn stats(&self) -> MemoryRetrievalStats {
        let working_stats = self.working.stats().await;
        let permanent_stats = self.permanent.stats().await;
        
        MemoryRetrievalStats {
            working_items: working_stats.total_items,
            permanent_items: permanent_stats.total_items,
            total_items: working_stats.total_items + permanent_stats.total_items,
            working_active: working_stats.active_items,
            permanent_avg_confidence: permanent_stats.avg_confidence,
        }
    }
}

/// Statistics for the retrieval system
#[derive(Debug, Clone)]
pub struct MemoryRetrievalStats {
    pub working_items: usize,
    pub permanent_items: usize,
    pub total_items: usize,
    pub working_active: usize,
    pub permanent_avg_confidence: f32,
}

#[cfg(test)]
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
}
