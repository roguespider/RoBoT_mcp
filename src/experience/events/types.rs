// /src/experience/events/types.rs
//! Event types - Per Architecture §4.04
//!
//! Event catalog per architecture:
//! - ExperienceRecorded → Reflection observes → Hypothesis evaluates → Knowledge updates → Reputation adjusts

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::payload::EventPayload;

/// ============================================================================
/// EXPERIENCE EVENT
/// ============================================================================
/// A signal emitted by the experience system.
///
/// Events are not memories themselves.
/// They notify subsystems that something happened.
///
/// Per Architecture §4.04:
/// - ExperienceRecorded: Experience records an event
/// - Reflection observes: Reflection observes the event
/// - Hypothesis evaluates: Hypothesis evaluates possible explanations
/// - Knowledge updates: Knowledge updates confidence
/// - Reputation adjusts: Reputation adjusts trust metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExperienceEvent {
    /// Unique event identifier.
    pub id: Uuid,

    /// Experience this event belongs to.
    pub experience_id: Uuid,

    /// When the event occurred.
    pub timestamp: DateTime<Utc>,

    /// Category of event.
    pub event_type: ExperienceEventType,

    /// Data associated with the event.
    pub payload: EventPayload,
}

/// ============================================================================
/// EXPERIENCE EVENT TYPES
/// ============================================================================
/// Types of signals flowing through the experience system.
///
/// Per Architecture §4.04:
/// ExperienceRecorded → Reflection observes → Hypothesis evaluates → Knowledge updates → Reputation adjusts
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ExperienceEventType {
    // ===== Core Lifecycle Events (Architecture §4.04) =====
    
    /// A new experience was recorded (Architecture §4.04).
    ExperienceRecorded,
    
    /// Observation detected (Architecture §5.2).
    ObservationRecorded,
    
    /// Evidence added to experience.
    EvidenceAdded,
    
    // ===== Reflection Events (Architecture §4.04)
    
    /// Reflection completed (Architecture §4.04).
    ReflectionCompleted,
    
    /// Pattern detected during reflection.
    PatternDetected,
    
    /// Lesson extracted from experience.
    LessonLearned,
    
    // ===== Hypothesis Events (Architecture §4.04)
    
    /// A new hypothesis was created (Architecture §4.04).
    HypothesisGenerated,
    
    /// Hypothesis updated.
    HypothesisUpdated,
    
    /// Hypothesis validated or invalidated.
    HypothesisValidated,
    
    // ===== Knowledge Events (Architecture §4.04)
    
    /// Knowledge updated (Architecture §4.04).
    KnowledgeUpdated,
    
    /// Knowledge promoted to higher confidence.
    KnowledgePromoted,
    
    /// Knowledge deprecated or weakened.
    KnowledgeDeprecated,
    
    // ===== Reputation Events (Architecture §4.04)
    
    /// Reputation changed for a target (Architecture §4.04).
    ReputationUpdated,
    
    /// Source trust level changed.
    SourceTrustChanged,
    
    // ===== Exploration Events (Architecture §4.04)
    
    /// Exploration started.
    ExplorationStarted,
    
    /// Exploration finished (Architecture §4.04).
    ExplorationCompleted,
    
    // ===== Planning Events (Architecture §4.03.5, §10)
    
    /// New plan created.
    PlanCreated,
    
    /// Plan status changed.
    PlanStatusChanged,
    
    /// Plan completed.
    PlanCompleted,
    
    // ===== Memory Events (Architecture §6.3)
    
    /// Memory item created.
    MemoryStored,
    
    /// Memory item archived.
    MemoryArchived,
    
    /// Memory item retrieved.
    MemoryRetrieved,
    
    // ===== Scoring Events
    
    /// An experience was evaluated by the scorer.
    Scored,
    
    /// Confidence level changed.
    ConfidenceChanged,
    
    // ===== Generic Events
    
    /// Generic system event.
    System,
    
    /// Custom extension point.
    Custom(String),
}

impl ExperienceEventType {
    /// Get a human-readable name for this event type
    pub fn name(&self) -> &'static str {
        match self {
            ExperienceEventType::ExperienceRecorded => "Experience Recorded",
            ExperienceEventType::ObservationRecorded => "Observation Recorded",
            ExperienceEventType::EvidenceAdded => "Evidence Added",
            ExperienceEventType::ReflectionCompleted => "Reflection Completed",
            ExperienceEventType::PatternDetected => "Pattern Detected",
            ExperienceEventType::LessonLearned => "Lesson Learned",
            ExperienceEventType::HypothesisGenerated => "Hypothesis Generated",
            ExperienceEventType::HypothesisUpdated => "Hypothesis Updated",
            ExperienceEventType::HypothesisValidated => "Hypothesis Validated",
            ExperienceEventType::KnowledgeUpdated => "Knowledge Updated",
            ExperienceEventType::KnowledgePromoted => "Knowledge Promoted",
            ExperienceEventType::KnowledgeDeprecated => "Knowledge Deprecated",
            ExperienceEventType::ReputationUpdated => "Reputation Updated",
            ExperienceEventType::SourceTrustChanged => "Source Trust Changed",
            ExperienceEventType::ExplorationStarted => "Exploration Started",
            ExperienceEventType::ExplorationCompleted => "Exploration Completed",
            ExperienceEventType::PlanCreated => "Plan Created",
            ExperienceEventType::PlanStatusChanged => "Plan Status Changed",
            ExperienceEventType::PlanCompleted => "Plan Completed",
            ExperienceEventType::MemoryStored => "Memory Stored",
            ExperienceEventType::MemoryArchived => "Memory Archived",
            ExperienceEventType::MemoryRetrieved => "Memory Retrieved",
            ExperienceEventType::Scored => "Experience Scored",
            ExperienceEventType::ConfidenceChanged => "Confidence Changed",
            ExperienceEventType::System => "System Event",
            ExperienceEventType::Custom(_) => "Custom Event",
        }
    }
}
