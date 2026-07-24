// robot_mcp/src/experience/scorer.rs

use anyhow::Result;

use crate::experience::{
    events::ExperienceEvent,
    observer::ExperienceObserver,
    types::{Experience, ExperienceScore, OutcomeKind},
};

/// Calculates learning signals for experiences.
///
/// The scorer does not decide what is true.
/// It only evaluates the usefulness and quality
/// of recorded experiences.
pub struct ExperienceScorer;

/// Scores individual encounters.
#[allow(dead_code)]
pub struct EncounterScore {
    pub success: f32,
    pub quality: f32,
    pub reliability: f32,
}

impl ExperienceScorer {
    pub fn new() -> Self {
        Self
    }

    /// Generate a score for an experience.
    pub fn score(&self, experience: &Experience) -> ExperienceScore {
        ExperienceScore {
            importance: self.calculate_importance(experience),
            confidence: self.calculate_confidence(experience),
            novelty: self.calculate_novelty(experience),
            reliability: self.calculate_reliability(experience),
        }
    }

    fn calculate_importance(&self, experience: &Experience) -> f32 {
        let mut score: f32 = 0.5;

        // Errors and failures are valuable learning events.
        match experience.outcome.kind {
            OutcomeKind::Failure | OutcomeKind::Timeout => {
                score += 0.25;
            }

            OutcomeKind::Success => {
                score += 0.10;
            }

            _ => {}
        }

        // User feedback is highly valuable.
        if matches!(
            experience.experience_type,
            crate::experience::types::ExperienceType::UserFeedback
        ) {
            score += 0.25;
        }

        score.clamp(0.0, 1.0)
    }

    fn calculate_confidence(&self, experience: &Experience) -> f32 {
        let mut score: f32 = 0.5;

        if experience.context.tool.is_some() {
            score += 0.1;
        }

        if experience.context.model.is_some() {
            score += 0.1;
        }

        if experience.outcome.error.is_some() {
            score += 0.1;
        }

        score.clamp(0.0, 1.0)
    }

    fn calculate_novelty(&self, _experience: &Experience) -> f32 {
        // Future:
        // Compare embeddings against previous experiences.
        //
        // This will eventually use memory/vector search.

        0.5
    }

    fn calculate_reliability(&self, experience: &Experience) -> f32 {
        match experience.outcome.kind {
            OutcomeKind::Success => 0.8,
            OutcomeKind::Partial => 0.5,
            OutcomeKind::Failure => 0.2,
            OutcomeKind::Timeout => 0.1,
            OutcomeKind::Interrupted => 0.3,
        }
    }
}

impl ExperienceObserver for ExperienceScorer {
    fn name(&self) -> &'static str {
        "ExperienceScorer"
    }

    fn observe(&self, event: &ExperienceEvent) -> Result<()> {
        // For now, just log the event
        println!("Scorer received event: {:?}", event);
        Ok(())
    }
}
