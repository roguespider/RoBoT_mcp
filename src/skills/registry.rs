// src/skills/registry.rs
//! Skill registry for managing available skills

use std::sync::Arc;

use anyhow::Result;
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use uuid::Uuid;

/// Skill category
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum SkillCategory {
    FileOperation,
    CodeAnalysis,
    Search,
    Memory,
    Learning,
    Planning,
    Communication,
    Web,
    Database,
    System,
    Custom,
}

impl SkillCategory {
    pub fn as_str(&self) -> &'static str {
        match self {
            SkillCategory::FileOperation => "file_operation",
            SkillCategory::CodeAnalysis => "code_analysis",
            SkillCategory::Search => "search",
            SkillCategory::Memory => "memory",
            SkillCategory::Learning => "learning",
            SkillCategory::Planning => "planning",
            SkillCategory::Communication => "communication",
            SkillCategory::Web => "web",
            SkillCategory::Database => "database",
            SkillCategory::System => "system",
            SkillCategory::Custom => "custom",
        }
    }
}

/// Metadata about a skill
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillMetadata {
    pub name: String,
    pub description: String,
    pub category: SkillCategory,
    pub version: String,
    pub author: Option<String>,
    pub tags: Vec<String>,
    pub examples: Vec<String>,
}

/// A registered skill with execution capability
#[derive(Debug, Clone)]
pub struct Skill {
    pub id: String,
    pub metadata: SkillMetadata,
    pub enabled: bool,
    pub usage_count: u64,
    pub success_count: u64,
    pub last_used: Option<chrono::DateTime<chrono::Utc>>,
}

impl Skill {
    /// Create a new skill
    pub fn new(metadata: SkillMetadata) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            metadata,
            enabled: true,
            usage_count: 0,
            success_count: 0,
            last_used: None,
        }
    }

    /// Record usage
    pub fn record_usage(&mut self, success: bool) {
        self.usage_count += 1;
        if success {
            self.success_count += 1;
        }
        self.last_used = Some(chrono::Utc::now());
    }

    /// Get success rate
    pub fn success_rate(&self) -> f32 {
        if self.usage_count == 0 {
            return 1.0;
        }
        self.success_count as f32 / self.usage_count as f32
    }
}

/// Skill registry for managing available skills
pub struct SkillRegistry {
    skills: Arc<RwLock<Vec<Skill>>>,
}

impl SkillRegistry {
    /// Create a new skill registry
    pub fn new() -> Self {
        Self {
            skills: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Register a new skill
    pub async fn register(&self, skill: Skill) -> Result<String> {
        let mut skills = self.skills.write().await;
        
        // Check for duplicate name
        if skills.iter().any(|s| s.metadata.name == skill.metadata.name) {
            anyhow::bail!("Skill '{}' is already registered", skill.metadata.name);
        }
        
        skills.push(skill);
        Ok(skills.last().unwrap().id.clone())
    }

    /// Unregister a skill by ID
    pub async fn unregister(&self, skill_id: &str) -> Result<()> {
        let mut skills = self.skills.write().await;
        skills.retain(|s| s.id != skill_id);
        Ok(())
    }

    /// Enable a skill
    pub async fn enable(&self, skill_id: &str) -> Result<()> {
        let mut skills = self.skills.write().await;
        if let Some(skill) = skills.iter_mut().find(|s| s.id == skill_id) {
            skill.enabled = true;
        }
        Ok(())
    }

    /// Disable a skill
    pub async fn disable(&self, skill_id: &str) -> Result<()> {
        let mut skills = self.skills.write().await;
        if let Some(skill) = skills.iter_mut().find(|s| s.id == skill_id) {
            skill.enabled = false;
        }
        Ok(())
    }

    /// Get a skill by ID
    pub async fn get(&self, skill_id: &str) -> Option<Skill> {
        let skills = self.skills.read().await;
        skills.iter().find(|s| s.id == skill_id).cloned()
    }

    /// Get a skill by name
    pub async fn get_by_name(&self, name: &str) -> Option<Skill> {
        let skills = self.skills.read().await;
        skills.iter().find(|s| s.metadata.name == name).cloned()
    }

    /// List all skills
    pub async fn list(&self) -> Vec<Skill> {
        let skills = self.skills.read().await;
        skills.clone()
    }

    /// List enabled skills
    pub async fn list_enabled(&self) -> Vec<Skill> {
        let skills = self.skills.read().await;
        skills.iter().filter(|s| s.enabled).cloned().collect()
    }

    /// List skills by category
    pub async fn list_by_category(&self, category: SkillCategory) -> Vec<Skill> {
        let skills = self.skills.read().await;
        skills.iter().filter(|s| s.metadata.category == category).cloned().collect()
    }

    /// Search skills by tag
    pub async fn search_by_tag(&self, tag: &str) -> Vec<Skill> {
        let skills = self.skills.read().await;
        skills.iter()
            .filter(|s| s.metadata.tags.iter().any(|t| t.contains(tag)))
            .cloned()
            .collect()
    }

    /// Record skill usage
    pub async fn record_usage(&self, skill_id: &str, success: bool) -> Result<()> {
        let mut skills = self.skills.write().await;
        if let Some(skill) = skills.iter_mut().find(|s| s.id == skill_id) {
            skill.record_usage(success);
        }
        Ok(())
    }

    /// Get most used skills
    pub async fn get_most_used(&self, limit: usize) -> Vec<Skill> {
        let mut skills = self.skills.read().await.clone();
        skills.sort_by(|a, b| b.usage_count.cmp(&a.usage_count));
        skills.truncate(limit);
        skills
    }

    /// Get most successful skills
    pub async fn get_most_successful(&self, min_uses: u64) -> Vec<Skill> {
        let skills = self.skills.read().await;
        let mut result: Vec<Skill> = skills.iter()
            .filter(|s| s.usage_count >= min_uses)
            .cloned()
            .collect();
        result.sort_by(|a, b| b.success_rate().partial_cmp(&a.success_rate()).unwrap());
        result
    }

    /// Load default skills
    pub async fn load_defaults(&self) {
        let defaults = vec![
            Skill::new(SkillMetadata {
                name: "file_read".to_string(),
                description: "Read contents of a file".to_string(),
                category: SkillCategory::FileOperation,
                version: "1.0.0".to_string(),
                author: Some("RoBoT".to_string()),
                tags: vec!["file".to_string(), "read".to_string(), "io".to_string()],
                examples: vec!["Read file at path /src/main.rs".to_string()],
            }),
            Skill::new(SkillMetadata {
                name: "file_write".to_string(),
                description: "Write contents to a file".to_string(),
                category: SkillCategory::FileOperation,
                version: "1.0.0".to_string(),
                author: Some("RoBoT".to_string()),
                tags: vec!["file".to_string(), "write".to_string(), "io".to_string()],
                examples: vec!["Write content to /src/output.txt".to_string()],
            }),
            Skill::new(SkillMetadata {
                name: "search".to_string(),
                description: "Search for patterns in files".to_string(),
                category: SkillCategory::Search,
                version: "1.0.0".to_string(),
                author: Some("RoBoT".to_string()),
                tags: vec!["search".to_string(), "grep".to_string(), "find".to_string()],
                examples: vec!["Search for 'TODO' in all .rs files".to_string()],
            }),
            Skill::new(SkillMetadata {
                name: "memory_store".to_string(),
                description: "Store information in memory".to_string(),
                category: SkillCategory::Memory,
                version: "1.0.0".to_string(),
                author: Some("RoBoT".to_string()),
                tags: vec!["memory".to_string(), "store".to_string(), "persist".to_string()],
                examples: vec!["Store that project uses Rust edition 2024".to_string()],
            }),
            Skill::new(SkillMetadata {
                name: "memory_recall".to_string(),
                description: "Recall information from memory".to_string(),
                category: SkillCategory::Memory,
                version: "1.0.0".to_string(),
                author: Some("RoBoT".to_string()),
                tags: vec!["memory".to_string(), "recall".to_string(), "retrieve".to_string()],
                examples: vec!["Recall all information about the database schema".to_string()],
            }),
        ];

        let mut skills = self.skills.write().await;
        *skills = defaults;
    }
}

impl Default for SkillRegistry {
    fn default() -> Self {
        Self::new()
    }
}
