// src/learning/mod.rs
//! Learning module for experience-based learning

#![allow(dead_code, unused_imports)]

pub mod working_memory;
pub mod hypothesis;
pub mod candidates;

pub use working_memory::WorkingMemory;
pub use hypothesis::{Hypothesis, HypothesisEvidence, HypothesisStatus};
pub use candidates::{Candidate, CandidateGenerator, CandidateScore};
