// /src/experience/reflection/services/repository.rs
// Repository for persisting and retrieving reflections

use anyhow::Result;
use std::collections::HashMap;
use std::sync::RwLock;

use super::super::{Reflection, ReflectionStatus, ReflectionType};

/// Thread-safe in-memory repository for reflections
pub struct ReflectionRepository {
    reflections: RwLock<HashMap<String, Reflection>>,
    by_type: RwLock<HashMap<ReflectionType, Vec<String>>>,
    by_status: RwLock<HashMap<ReflectionStatus, Vec<String>>>,
}

impl ReflectionRepository {
    /// Create a new empty repository
    pub fn new() -> Self {
        Self {
            reflections: RwLock::new(HashMap::new()),
            by_type: RwLock::new(HashMap::new()),
            by_status: RwLock::new(HashMap::new()),
        }
    }

    /// Save a reflection to the repository
    pub fn save(&self, reflection: Reflection) -> Result<()> {
        let id = reflection.id.clone();
        let reflection_type = reflection.reflection_type.clone();
        let status = reflection.status.clone();

        // Store the reflection
        {
            let mut reflections = self.reflections.write()
                .map_err(|_| anyhow::anyhow!("Lock poisoned"))?;
            reflections.insert(id.clone(), reflection);
        }

        // Update type index
        {
            let mut by_type = self.by_type.write()
                .map_err(|_| anyhow::anyhow!("Lock poisoned"))?;
            by_type.entry(reflection_type).or_insert_with(Vec::new).push(id.clone());
        }

        // Update status index
        {
            let mut by_status = self.by_status.write()
                .map_err(|_| anyhow::anyhow!("Lock poisoned"))?;
            by_status.entry(status).or_insert_with(Vec::new).push(id);
        }

        Ok(())
    }

    /// Get a reflection by ID
    pub fn get(&self, id: &str) -> Result<Option<Reflection>> {
        let reflections = self.reflections.read()
            .map_err(|_| anyhow::anyhow!("Lock poisoned"))?;
        Ok(reflections.get(id).cloned())
    }

    /// List all reflections
    pub fn list_all(&self) -> Result<Vec<Reflection>> {
        let reflections = self.reflections.read()
            .map_err(|_| anyhow::anyhow!("Lock poisoned"))?;
        Ok(reflections.values().cloned().collect())
    }

    /// List reflections by type
    pub fn list_by_type(&self, reflection_type: ReflectionType) -> Result<Vec<Reflection>> {
        let ids = {
            let by_type = self.by_type.read()
                .map_err(|_| anyhow::anyhow!("Lock poisoned"))?;
            by_type.get(&reflection_type).cloned().unwrap_or_default()
        };

        let reflections = self.reflections.read()
            .map_err(|_| anyhow::anyhow!("Lock poisoned"))?;
        
        Ok(ids.iter()
            .filter_map(|id| reflections.get(id).cloned())
            .collect())
    }

    /// List reflections by status
    pub fn list_by_status(&self, status: ReflectionStatus) -> Result<Vec<Reflection>> {
        let ids = {
            let by_status = self.by_status.read()
                .map_err(|_| anyhow::anyhow!("Lock poisoned"))?;
            by_status.get(&status).cloned().unwrap_or_default()
        };

        let reflections = self.reflections.read()
            .map_err(|_| anyhow::anyhow!("Lock poisoned"))?;
        
        Ok(ids.iter()
            .filter_map(|id| reflections.get(id).cloned())
            .collect())
    }

    /// Find reflections by title substring
    pub fn search_by_title(&self, query: &str) -> Result<Vec<Reflection>> {
        let reflections = self.reflections.read()
            .map_err(|_| anyhow::anyhow!("Lock poisoned"))?;
        
        let query_lower = query.to_lowercase();
        Ok(reflections.values()
            .filter(|r| r.title.to_lowercase().contains(&query_lower))
            .cloned()
            .collect())
    }

    /// Delete a reflection by ID
    pub fn delete(&self, id: &str) -> Result<Option<Reflection>> {
        let removed = {
            let mut reflections = self.reflections.write()
                .map_err(|_| anyhow::anyhow!("Lock poisoned"))?;
            reflections.remove(id)
        };

        if let Some(reflection) = &removed {
            // Remove from type index
            if let Ok(mut by_type) = self.by_type.write() {
                if let Some(ids) = by_type.get_mut(&reflection.reflection_type) {
                    ids.retain(|i| i != id);
                }
            }

            // Remove from status index
            if let Ok(mut by_status) = self.by_status.write() {
                if let Some(ids) = by_status.get_mut(&reflection.status) {
                    ids.retain(|i| i != id);
                }
            }
        }

        Ok(removed)
    }

    /// Update an existing reflection
    pub fn update(&self, reflection: &Reflection) -> Result<()> {
        let id = &reflection.id;

        // Check if exists
        {
            let reflections = self.reflections.read()
                .map_err(|_| anyhow::anyhow!("Lock poisoned"))?;
            if !reflections.contains_key(id) {
                return Err(anyhow::anyhow!("Reflection not found: {}", id));
            }
        }

        self.save(reflection.clone())
    }

    /// Get count of reflections
    pub fn count(&self) -> Result<usize> {
        let reflections = self.reflections.read()
            .map_err(|_| anyhow::anyhow!("Lock poisoned"))?;
        Ok(reflections.len())
    }

    /// Get validated reflections (high confidence)
    pub fn list_validated(&self, min_confidence: f32) -> Result<Vec<Reflection>> {
        let reflections = self.reflections.read()
            .map_err(|_| anyhow::anyhow!("Lock poisoned"))?;
        
        Ok(reflections.values()
            .filter(|r| r.status == ReflectionStatus::Validated && r.confidence.score >= min_confidence)
            .cloned()
            .collect())
    }
}

impl Default for ReflectionRepository {
    fn default() -> Self {
        Self::new()
    }
}
