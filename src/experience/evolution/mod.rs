//! ============================================================================
//! Evolution System
//! ============================================================================
//!
//! The evolution system transforms insights and reflections into actual
//! behavioral changes in the agent.
//!
//! Evolution **changes behavior directly** based on validated learnings.
//!
//! The pipeline is:
//!
//! Insights → Behavior Candidates → Evaluation → Adoption/Rejection
//!
//! Key concepts:
//! - Behavior: An actionable change derived from validated insights
//! - Adoption: Accepting a behavior as part of the agent's repertoire
//! - Rejection: Discarding a behavior candidate that fails validation
//! - Decay: Behaviors that aren't practiced over time fade away

pub mod behavior;
pub mod engine;
pub mod evidence;

// Re-exports
pub use engine::EvolutionEngine;

