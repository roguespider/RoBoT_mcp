// src/memory/types.rs
//! Memory types - Per Architecture §4.08, §6.3

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Memory layer type - Per Architecture §6.3
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum MemoryLayer {
    /// Working Memory - Temporary information used during active tasks
    Working,
    /// Permanent Memory - Curated knowledge retained after evaluation
    Permanent,
}

impl std::fmt::Display for MemoryLayer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MemoryLayer::Working => write!(f, "Working"),
            MemoryLayer::Permanent => write!(f, "Permanent"),
        }
    }
}

/// Memory item type classification
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum MemoryType {
    /// Experience-based memory
    Experience,
    /// Knowledge fact
    Knowledge,
    /// Procedural skill
    Skill,
    /// Workflow definition
    Workflow,
    /// User context
    Context,
    /// Observation record
    Observation,
}

impl std::fmt::Display for MemoryType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            MemoryType::Experience => "experience",
            MemoryType::Knowledge => "knowledge",
            MemoryType::Skill => "skill",
            MemoryType::Workflow => "workflow",
            MemoryType::Context => "context",
            MemoryType::Observation => "observation",
        };
        write!(f, "{}", s)
    }
}

/// Memory status - Per Architecture §6.3
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum MemoryStatus {
    /// Active and accessible
    Active,
    /// Inactive but retained
    Inactive,
    /// Archived for later retrieval
    Archived,
    /// Being garbage collected
    PendingDeletion,
}

impl Default for MemoryStatus {
    fn default() -> Self {
        MemoryStatus::Active
    }
}

/// A memory item - Per Architecture §4.08
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryItem {
    /// Unique identifier
    pub id: Uuid,

    /// Memory layer (Working or Permanent)
    pub layer: MemoryLayer,

    /// Memory type classification
    pub memory_type: MemoryType,

    /// Status of this memory
    pub status: MemoryStatus,

    /// Content/description of this memory
    pub content: String,

    /// Confidence level (0.0 - 1.0)
    pub confidence: f32,

    /// Importance level (0.0 - 1.0)
    pub importance: f32,

    /// When this memory was created
    pub created_at: DateTime<Utc>,

    /// When this memory was last accessed
    pub accessed_at: DateTime<Utc>,

    /// When this memory was last modified
    pub modified_at: DateTime<Utc>,

    /// Access count
    pub access_count: u32,

    /// Tags for categorization
    pub tags: Vec<String>,

    /// Source of this memory (experience, user, system)
    pub source: String,

    /// Related memory IDs
    pub related_ids: Vec<Uuid>,
}

impl MemoryItem {
    /// Create a new memory item
    pub fn new(
        layer: MemoryLayer,
        memory_type: MemoryType,
        content: String,
        source: String,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            layer,
            memory_type,
            status: MemoryStatus::Active,
            content,
            confidence: 0.5,
            importance: 0.5,
            created_at: now,
            accessed_at: now,
            modified_at: now,
            access_count: 0,
            tags: Vec::new(),
            source,
            related_ids: Vec::new(),
        }
    }

    /// Record an access to this memory
    pub fn record_access(&mut self) {
        self.access_count += 1;
        self.accessed_at = Utc::now();
    }

    /// Update confidence
    pub fn update_confidence(&mut self, confidence: f32) {
        self.confidence = confidence.clamp(0.0, 1.0);
        self.modified_at = Utc::now();
    }

    /// Archive this memory
    pub fn archive(&mut self) {
        self.status = MemoryStatus::Archived;
        self.modified_at = Utc::now();
    }

    /// Add a related memory
    pub fn add_related(&mut self, related_id: Uuid) {
        if !self.related_ids.contains(&related_id) {
            self.related_ids.push(related_id);
        }
    }

    /// Add a tag
    pub fn add_tag(&mut self, tag: impl Into<String>) {
        let tag = tag.into();
        if !self.tags.contains(&tag) {
            self.tags.push(tag);
        }
    }
}

impl Default for MemoryLayer {
    fn default() -> Self {
        MemoryLayer::Working
    }
}
