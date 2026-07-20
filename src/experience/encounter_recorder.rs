// \src\experience\encounter_recorder.rs

use std::sync::Arc;

use anyhow::Result;
use chrono::Utc;
use uuid::Uuid;

use crate::database::models::MemoryCard;
use crate::database::queries;
use crate::database::sqlite::SqliteDatabase;
use crate::experience::types::{Experience, ExperienceContext, ExperienceOutcome, ExperienceType, KnowledgeMaturity};

/// Records experiences to storage
pub struct ExperienceRecorder {
    database: Arc<SqliteDatabase>,
}

impl ExperienceRecorder {
    pub fn new(database: Arc<SqliteDatabase>) -> Self {
        Self { database }
    }

    /// Record a completed experience.
    pub fn record(
        &self,
        experience_type: ExperienceType,
        title: impl Into<String>,
        description: impl Into<String>,
        context: ExperienceContext,
        outcome: ExperienceOutcome,
    ) -> Result<String> {
        let id = Uuid::new_v4();

        let experience = Experience {
            id,
            timestamp: Utc::now(),
            experience_type,
            title: title.into(),
            description: description.into(),
            context,
            outcome,
            score: None,
            encounter_ids: Vec::new(),
            maturity: KnowledgeMaturity::Emerging,
            confidence: 0.5,
            lessons: Vec::new(),
            evidence_count: 0,
            tags: Vec::new(),
            metadata: std::collections::HashMap::new(),
        };

        // Store in database
        let conn = self.database.connection()?;
        let memory = MemoryCard::from_experience(&experience);
        queries::insert_memory(&conn, &memory)?;
        
        tracing::info!("Recorded experience: {}", id);

        Ok(id.to_string())
    }

    /// Convenience helper for successful actions.
    pub fn success(
        &self,
        experience_type: ExperienceType,
        title: impl Into<String>,
        description: impl Into<String>,
    ) -> Result<String> {
        self.record(
            experience_type,
            title,
            description,
            ExperienceContext::default(),
            ExperienceOutcome::success(),
        )
    }

    /// Convenience helper for failed actions.
    pub fn failure(
        &self,
        experience_type: ExperienceType,
        title: impl Into<String>,
        description: impl Into<String>,
        reason: impl Into<String>,
    ) -> Result<String> {
        self.record(
            experience_type,
            title,
            description,
            ExperienceContext::default(),
            ExperienceOutcome::failure(reason),
        )
    }
}
