// /src/experience/events/builders.rs
#![allow(dead_code)]

use chrono::Utc;
use uuid::Uuid;

use super::{EventPayload, ExperienceEvent, ExperienceEventType};
use crate::experience::types::ExperienceScore;

impl ExperienceEvent {
    /// Create an event indicating a new experience was recorded.
    pub fn recorded(experience_id: Uuid) -> Self {
        Self {
            id: Uuid::new_v4(),
            experience_id,
            timestamp: Utc::now(),
            event_type: ExperienceEventType::ExperienceRecorded,
            payload: EventPayload::Experience { experience_id },
        }
    }

    /// Create an event after an experience has been scored.
    pub fn scored(experience_id: Uuid, score: ExperienceScore) -> Self {
        Self {
            id: Uuid::new_v4(),
            experience_id,
            timestamp: Utc::now(),
            event_type: ExperienceEventType::Scored,
            payload: EventPayload::Score {
                experience_id,
                score,
            },
        }
    }

    /// Create an event when reputation changes.
    pub fn reputation_updated(experience_id: Uuid, target_id: String, change: f32) -> Self {
        Self {
            id: Uuid::new_v4(),
            experience_id,
            timestamp: Utc::now(),
            event_type: ExperienceEventType::ReputationUpdated,
            payload: EventPayload::Reputation {
                entity_id: target_id,
                change,
            },
        }
    }

    /// Create an event when reflection completes.
    pub fn reflection_completed(experience_id: Uuid, reflection_id: Uuid) -> Self {
        Self {
            id: Uuid::new_v4(),
            experience_id,
            timestamp: Utc::now(),
            event_type: ExperienceEventType::ReflectionCompleted,
            payload: EventPayload::Reflection { reflection_id },
        }
    }

    /// Create an event when a hypothesis is generated.
    pub fn hypothesis_generated(experience_id: Uuid, hypothesis_id: Uuid) -> Self {
        Self {
            id: Uuid::new_v4(),
            experience_id,
            timestamp: Utc::now(),
            event_type: ExperienceEventType::HypothesisGenerated,
            payload: EventPayload::Hypothesis { hypothesis_id },
        }
    }

    /// Create an event when exploration finishes.
    pub fn exploration_completed(experience_id: Uuid, exploration_id: Uuid) -> Self {
        Self {
            id: Uuid::new_v4(),
            experience_id,
            timestamp: Utc::now(),
            event_type: ExperienceEventType::ExplorationCompleted,
            payload: EventPayload::Exploration { exploration_id },
        }
    }
}
