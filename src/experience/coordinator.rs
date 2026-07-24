// /src/experience/coordinator.rs
// Experience system coordinator per Architecture §07
#![allow(dead_code)]

use crate::experience::{
    bus::ExperienceBus,
    events::ExperienceEvent,
    scorer::ExperienceScorer, types::*,
};
use std::sync::Arc;
use uuid::Uuid;

/// Coordinates the experience system.
///
/// The manager does not contain business logic.
/// Instead it orchestrates the specialized components.
pub struct ExperienceCoordinator {
    scorer: ExperienceScorer,
    bus: Arc<ExperienceBus>,
}

impl ExperienceCoordinator {
    pub fn new(scorer: ExperienceScorer, bus: Arc<ExperienceBus>) -> Self {
        Self { scorer, bus }
    }

    /// Process a completed experience through the learning pipeline.
    pub fn process(&self, mut experience: Experience) -> Experience {
        // Score it.
        let score = self.scorer.score(&experience);
        experience.score = Some(score.clone());

        // Publish scored event using builder
        let event = ExperienceEvent::scored(experience.id, score);
        let _ = self.bus.publish(event);

        experience
    }

    /// Record that an experience was created
    pub fn record_experience(&self, id: Uuid) {
        let event = ExperienceEvent::recorded(id);
        let _ = self.bus.publish(event);
    }

    /// Record that reflection was completed
    pub fn complete_reflection(&self, id: Uuid) {
        let reflection_id = Uuid::new_v4();
        let event = ExperienceEvent::reflection_completed(id, reflection_id);
        let _ = self.bus.publish(event);
    }

    /// Record that a hypothesis was generated
    pub fn generate_hypothesis(&self, id: Uuid) {
        let hypothesis_id = Uuid::new_v4();
        let event = ExperienceEvent::hypothesis_generated(id, hypothesis_id);
        let _ = self.bus.publish(event);
    }

    /// Record that exploration was completed
    pub fn complete_exploration(&self, id: Uuid) {
        let exploration_id = Uuid::new_v4();
        let event = ExperienceEvent::exploration_completed(id, exploration_id);
        let _ = self.bus.publish(event);
    }
}
