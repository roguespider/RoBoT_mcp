// src/learning/lineage.rs
//! Memory lineage tracking - stores the full history and evolution of memories

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use std::collections::HashMap;

/// Represents the origin of a memory
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryLineage {
    /// Current memory ID
    pub memory_id: Uuid,
    
    /// Chain of evidence IDs that support this memory
    pub supporting_evidence: Vec<EvidenceRef>,
    
    /// Chain of observation IDs that observed this fact
    pub observations: Vec<ObservationRef>,
    
    /// Refinements this memory has undergone
    pub refinements: Vec<Refinement>,
    
    /// Memories that supersede this one (if any)
    pub superseded_by: Option<Uuid>,
    
    /// Memories this one supersedes
    pub supersedes: Vec<Uuid>,
    
    /// Contradictions that challenge this memory
    pub contradictions: Vec<Contradiction>,
    
    /// Confirmations from external sources
    pub confirmations: Vec<Confirmation>,
    
    /// When this memory was created
    pub created_at: DateTime<Utc>,
    
    /// When this memory was last updated
    pub updated_at: DateTime<Utc>,
}

impl MemoryLineage {
    /// Create new lineage for a memory
    pub fn new(memory_id: Uuid) -> Self {
        let now = Utc::now();
        Self {
            memory_id,
            supporting_evidence: Vec::new(),
            observations: Vec::new(),
            refinements: Vec::new(),
            superseded_by: None,
            supersedes: Vec::new(),
            contradictions: Vec::new(),
            confirmations: Vec::new(),
            created_at: now,
            updated_at: now,
        }
    }
    
    /// Check if this memory has been superseded
    pub fn is_superseded(&self) -> bool {
        self.superseded_by.is_some()
    }
    
    /// Check if this memory contradicts another
    pub fn has_contradiction(&self) -> bool {
        !self.contradictions.is_empty()
    }
    
    /// Get the confidence boost from supporting evidence
    pub fn evidence_confidence_boost(&self) -> f32 {
        self.supporting_evidence.len() as f32 * 0.05
    }
    
    /// Get the confidence penalty from contradictions
    pub fn contradiction_confidence_penalty(&self) -> f32 {
        self.contradictions.len() as f32 * 0.15
    }
    
    /// Calculate final confidence based on lineage
    pub fn calculate_lineage_confidence(&self, base_confidence: f32) -> f32 {
        let mut confidence = base_confidence;
        confidence += self.evidence_confidence_boost();
        confidence += self.confirmations.len() as f32 * 0.1;
        confidence -= self.contradiction_confidence_penalty();
        confidence -= self.refinements.len() as f32 * 0.02;
        confidence.clamp(0.0, 1.0)
    }
}

/// Reference to supporting evidence
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidenceRef {
    pub id: Uuid,
    pub evidence_type: EvidenceType,
    pub confidence: f32,
    pub added_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum EvidenceType {
    Experience,
    Observation,
    Deduction,
    External,
}

/// Reference to an observation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObservationRef {
    pub id: Uuid,
    pub observation_type: ObservationType,
    pub timestamp: DateTime<Utc>,
    pub outcome: ObservationOutcome,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum ObservationType {
    Direct,
    Indirect,
    Inferred,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum ObservationOutcome {
    Positive,
    Negative,
    Neutral,
}

/// A refinement of the memory content
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Refinement {
    pub id: Uuid,
    pub previous_content: String,
    pub new_content: String,
    pub reason: String,
    pub refinement_type: RefinementType,
    pub confidence_change: f32,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum RefinementType {
    Correction,
    Expansion,
    Clarification,
    Generalization,
    Specialization,
}

/// A contradiction challenging this memory
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Contradiction {
    pub id: Uuid,
    pub contradicting_memory_id: Uuid,
    pub description: String,
    pub strength: f32,  // How strong is the contradiction (0-1)
    pub resolved: bool,
    pub resolution: Option<ContradictionResolution>,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ContradictionResolution {
    /// Memory was wrong, superseded by new memory
    MemoryWasWrong { superseding_memory: Uuid },
    /// Contradiction was wrong, memory confirmed
    ContradictionWasWrong { reason: String },
    /// Both were partially correct, merged into new memory
    Merged { new_memory: Uuid },
    /// Both can coexist (context-dependent truth)
    Contextual { explanation: String },
}

/// A confirmation from an external source
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Confirmation {
    pub id: Uuid,
    pub source: String,
    pub source_type: ConfirmationSource,
    pub description: String,
    pub confidence_boost: f32,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum ConfirmationSource {
    User,
    External,
    CrossReference,
    Deduction,
}

/// Lineage tracker for managing memory histories
pub struct LineageTracker {
    lineages: HashMap<Uuid, MemoryLineage>,
}

impl LineageTracker {
    pub fn new() -> Self {
        Self {
            lineages: HashMap::new(),
        }
    }
    
    /// Create lineage for a new memory
    pub fn create_lineage(&mut self, memory_id: Uuid) -> &mut MemoryLineage {
        let lineage = MemoryLineage::new(memory_id);
        self.lineages.insert(memory_id, lineage);
        self.lineages.get_mut(&memory_id)
            .expect("Lineage was just inserted, should always exist")
    }
    
    /// Get lineage for a memory
    pub fn get_lineage(&self, memory_id: &Uuid) -> Option<&MemoryLineage> {
        self.lineages.get(memory_id)
    }
    
    /// Get mutable lineage for a memory
    pub fn get_lineage_mut(&mut self, memory_id: &Uuid) -> Option<&mut MemoryLineage> {
        self.lineages.get_mut(memory_id)
    }
    
    /// Add supporting evidence to a memory
    pub fn add_evidence(&mut self, memory_id: Uuid, evidence: EvidenceRef) {
        if let Some(lineage) = self.lineages.get_mut(&memory_id) {
            lineage.supporting_evidence.push(evidence);
            lineage.updated_at = Utc::now();
        }
    }
    
    /// Add an observation
    pub fn add_observation(&mut self, memory_id: Uuid, observation: ObservationRef) {
        if let Some(lineage) = self.lineages.get_mut(&memory_id) {
            lineage.observations.push(observation);
            lineage.updated_at = Utc::now();
        }
    }
    
    /// Record a refinement
    pub fn add_refinement(&mut self, memory_id: Uuid, refinement: Refinement) {
        if let Some(lineage) = self.lineages.get_mut(&memory_id) {
            lineage.refinements.push(refinement);
            lineage.updated_at = Utc::now();
        }
    }
    
    /// Record a contradiction
    pub fn add_contradiction(&mut self, memory_id: Uuid, contradiction: Contradiction) {
        if let Some(lineage) = self.lineages.get_mut(&memory_id) {
            lineage.contradictions.push(contradiction);
            lineage.updated_at = Utc::now();
        }
    }
    
    /// Record a confirmation
    pub fn add_confirmation(&mut self, memory_id: Uuid, confirmation: Confirmation) {
        if let Some(lineage) = self.lineages.get_mut(&memory_id) {
            lineage.confirmations.push(confirmation);
            lineage.updated_at = Utc::now();
        }
    }
    
    /// Mark memory as superseded by another
    pub fn mark_superseded(&mut self, memory_id: Uuid, superseding_memory_id: Uuid) {
        if let Some(lineage) = self.lineages.get_mut(&memory_id) {
            lineage.superseded_by = Some(superseding_memory_id);
            lineage.updated_at = Utc::now();
        }
        if let Some(lineage) = self.lineages.get_mut(&superseding_memory_id) {
            lineage.supersedes.push(memory_id);
            lineage.updated_at = Utc::now();
        }
    }
    
    /// Resolve a contradiction
    pub fn resolve_contradiction(&mut self, memory_id: Uuid, contradiction_id: Uuid, resolution: ContradictionResolution) {
        if let Some(lineage) = self.lineages.get_mut(&memory_id) {
            if let Some(contradiction) = lineage.contradictions.iter_mut().find(|c| c.id == contradiction_id) {
                contradiction.resolved = true;
                contradiction.resolution = Some(resolution);
                lineage.updated_at = Utc::now();
            }
        }
    }
    
    /// Get all unresolved contradictions for a memory
    pub fn get_unresolved_contradictions(&self, memory_id: &Uuid) -> Vec<&Contradiction> {
        if let Some(lineage) = self.lineages.get(memory_id) {
            lineage.contradictions.iter()
                .filter(|c| !c.resolved)
                .collect()
        } else {
            Vec::new()
        }
    }
    
    /// Get memories that supersede a given memory
    pub fn get_superseding_chain(&self, memory_id: &Uuid) -> Vec<Uuid> {
        let mut chain = Vec::new();
        let mut current = Some(*memory_id);
        
        while let Some(id) = current {
            if let Some(lineage) = self.lineages.get(&id) {
                if let Some(superseded_by) = lineage.superseded_by {
                    chain.push(superseded_by);
                    current = Some(superseded_by);
                } else {
                    break;
                }
            } else {
                break;
            }
        }
        
        chain
    }
    
    /// Get the current (most recent) memory in a supersession chain
    pub fn get_current_memory(&self, memory_id: &Uuid) -> Option<Uuid> {
        let chain = self.get_superseding_chain(memory_id);
        chain.last().copied().or(Some(*memory_id))
    }
    
    /// Calculate confidence for a memory based on lineage
    pub fn calculate_confidence(&self, memory_id: &Uuid, base_confidence: f32) -> f32 {
        if let Some(lineage) = self.lineages.get(memory_id) {
            lineage.calculate_lineage_confidence(base_confidence)
        } else {
            base_confidence
        }
    }
    
    /// Get lineage summary for a memory
    pub fn get_summary(&self, memory_id: &Uuid) -> Option<LineageSummary> {
        self.lineages.get(memory_id).map(|lineage| LineageSummary {
            memory_id: lineage.memory_id,
            evidence_count: lineage.supporting_evidence.len(),
            observation_count: lineage.observations.len(),
            refinement_count: lineage.refinements.len(),
            contradiction_count: lineage.contradictions.len(),
            unresolved_contradictions: lineage.contradictions.iter().filter(|c| !c.resolved).count(),
            confirmation_count: lineage.confirmations.len(),
            is_superseded: lineage.is_superseded(),
            supersedes_count: lineage.supersedes.len(),
            lineage_confidence: lineage.calculate_lineage_confidence(0.5),
        })
    }
    
    /// Get all lineages that have unresolved contradictions
    pub fn get_memories_with_contradictions(&self) -> Vec<Uuid> {
        self.lineages.iter()
            .filter(|(_, lineage)| lineage.has_contradiction() && !lineage.contradictions.iter().all(|c| c.resolved))
            .map(|(id, _)| *id)
            .collect()
    }
    
    /// Get all superseded memories
    pub fn get_superseded_memories(&self) -> Vec<Uuid> {
        self.lineages.iter()
            .filter(|(_, lineage)| lineage.is_superseded())
            .map(|(id, _)| *id)
            .collect()
    }
}

impl Default for LineageTracker {
    fn default() -> Self {
        Self::new()
    }
}

/// Summary of a memory's lineage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LineageSummary {
    pub memory_id: Uuid,
    pub evidence_count: usize,
    pub observation_count: usize,
    pub refinement_count: usize,
    pub contradiction_count: usize,
    pub unresolved_contradictions: usize,
    pub confirmation_count: usize,
    pub is_superseded: bool,
    pub supersedes_count: usize,
    pub lineage_confidence: f32,
}

impl std::fmt::Display for LineageSummary {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Memory {}: ", self.memory_id)?;
        write!(f, "{} evidence, ", self.evidence_count)?;
        write!(f, "{} observations, ", self.observation_count)?;
        write!(f, "{} refinements, ", self.refinement_count)?;
        if self.unresolved_contradictions > 0 {
            write!(f, "{} unresolved contradictions, ", self.unresolved_contradictions)?;
        }
        if self.confirmation_count > 0 {
            write!(f, "{} confirmations, ", self.confirmation_count)?;
        }
        if self.is_superseded {
            write!(f, "SUPERSEDED, ")?;
        }
        write!(f, "confidence: {:.1}%", self.lineage_confidence * 100.0)
    }
}
