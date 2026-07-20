// robot/src/experience/hypothesis/support/graph.rs

//! ============================================================================
//! HYPOTHESIS GRAPH
//! ============================================================================
//!
//! Dependency and relationship graph for hypotheses.
//!
//! This module allows RoBoT to understand connections between beliefs,
//! find relationships, detect cycles, and perform graph analysis.

use std::collections::{HashMap, HashSet, VecDeque};

use serde::{Deserialize, Serialize};

use crate::experience::hypothesis::core::{HypothesisId, Hypothesis, HypothesisStatus};

/// ============================================================================
/// HYPOTHESIS GRAPH
/// ============================================================================

/// A directed graph representing relationships between hypotheses.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct HypothesisGraph {
    /// Map from hypothesis ID to node index for O(1) lookups
    #[serde(skip)]
    node_index: HashMap<String, usize>,
    
    /// All nodes in the graph
    nodes: Vec<HypothesisNode>,
    
    /// All edges in the graph
    edges: Vec<HypothesisEdge>,
}

impl HypothesisGraph {
    /// Create a new empty graph
    pub fn new() -> Self {
        Self {
            node_index: HashMap::new(),
            nodes: Vec::new(),
            edges: Vec::new(),
        }
    }

    /// Add a node for a hypothesis
    pub fn add_node(&mut self, hypothesis_id: HypothesisId) -> usize {
        let id_str = hypothesis_id.0.clone();
        if let Some(&idx) = self.node_index.get(&id_str) {
            return idx;
        }
        
        let node = HypothesisNode {
            hypothesis_id: hypothesis_id.clone(),
            metadata: NodeMetadata::default(),
        };
        
        let index = self.nodes.len();
        self.nodes.push(node);
        self.node_index.insert(id_str, index);
        index
    }

    /// Add an edge between two hypotheses
    pub fn add_edge(&mut self, from: HypothesisId, to: HypothesisId, relationship: HypothesisRelationship) -> Option<usize> {
        // Ensure both nodes exist
        self.add_node(from.clone());
        self.add_node(to.clone());
        
        // Check if edge already exists
        if self.has_edge(&from, &to, &relationship) {
            return None;
        }
        
        let edge = HypothesisEdge {
            id: EdgeId::new(),
            from,
            to,
            relationship,
            weight: 1.0,
        };
        
        let index = self.edges.len();
        self.edges.push(edge);
        Some(index)
    }

    /// Check if an edge exists
    pub fn has_edge(&self, from: &HypothesisId, to: &HypothesisId, relationship: &HypothesisRelationship) -> bool {
        self.edges.iter().any(|e| 
            e.from.0 == from.0 && 
            e.to.0 == to.0 && 
            e.relationship == *relationship
        )
    }

    /// Get all edges for a node
    pub fn get_edges(&self, hypothesis_id: &HypothesisId) -> Vec<&HypothesisEdge> {
        self.edges.iter().filter(|e| e.from.0 == hypothesis_id.0).collect()
    }

    /// Get all incoming edges for a node
    pub fn get_incoming_edges(&self, hypothesis_id: &HypothesisId) -> Vec<&HypothesisEdge> {
        self.edges.iter().filter(|e| e.to.0 == hypothesis_id.0).collect()
    }

    /// Find all connected hypotheses
    pub fn find_connected(&self, hypothesis_id: &HypothesisId) -> Vec<HypothesisId> {
        let mut visited = HashSet::new();
        let mut queue = VecDeque::new();
        let mut result = Vec::new();
        
        queue.push_back(hypothesis_id.0.clone());
        visited.insert(hypothesis_id.0.clone());
        
        while let Some(current) = queue.pop_front() {
            // Find all outgoing neighbors
            for edge in self.edges.iter().filter(|e| e.from.0 == current) {
                if visited.insert(edge.to.0.clone()) {
                    result.push(HypothesisId(edge.to.0.clone()));
                    queue.push_back(edge.to.0.clone());
                }
            }
            
            // Find all incoming neighbors
            for edge in self.edges.iter().filter(|e| e.to.0 == current) {
                if visited.insert(edge.from.0.clone()) {
                    result.push(HypothesisId(edge.from.0.clone()));
                    queue.push_back(edge.from.0.clone());
                }
            }
        }
        
        result
    }

    /// Find path between two hypotheses using BFS
    pub fn find_path(&self, from: &HypothesisId, to: &HypothesisId) -> Option<Vec<HypothesisId>> {
        if !self.node_index.contains_key(&from.0) || !self.node_index.contains_key(&to.0) {
            return None;
        }
        
        let mut visited = HashSet::new();
        let mut queue = VecDeque::new();
        let mut parent: HashMap<String, Option<String>> = HashMap::new();
        
        queue.push_back(from.0.clone());
        visited.insert(from.0.clone());
        parent.insert(from.0.clone(), None);
        
        while let Some(current) = queue.pop_front() {
            if current == to.0 {
                // Reconstruct path
                let mut path = Vec::new();
                let mut node = Some(current);
                while let Some(n) = node {
                    path.push(HypothesisId(n.clone()));
                    node = parent.get(&n).cloned().flatten();
                }
                path.reverse();
                return Some(path);
            }
            
            for edge in self.edges.iter().filter(|e| e.from.0 == current) {
                if visited.insert(edge.to.0.clone()) {
                    parent.insert(edge.to.0.clone(), Some(current.clone()));
                    queue.push_back(edge.to.0.clone());
                }
            }
        }
        
        None
    }

    /// Detect cycles in the graph
    pub fn detect_cycles(&self) -> Vec<Vec<HypothesisId>> {
        let mut visited = HashSet::new();
        let mut recursion_stack = HashSet::new();
        let mut cycles = Vec::new();
        let mut path = Vec::new();
        
        for node in &self.nodes {
            if !visited.contains(&node.hypothesis_id.0) {
                self.detect_cycles_dfs(
                    &node.hypothesis_id,
                    &mut visited,
                    &mut recursion_stack,
                    &mut path,
                    &mut cycles,
                );
            }
        }
        
        cycles
    }

    fn detect_cycles_dfs(
        &self,
        hypothesis_id: &HypothesisId,
        visited: &mut HashSet<String>,
        recursion_stack: &mut HashSet<String>,
        path: &mut Vec<String>,
        cycles: &mut Vec<Vec<HypothesisId>>,
    ) {
        let id = hypothesis_id.0.clone();
        visited.insert(id.clone());
        recursion_stack.insert(id.clone());
        path.push(id.clone());
        
        for edge in self.edges.iter().filter(|e| e.from.0 == id) {
            if !visited.contains(&edge.to.0) {
                self.detect_cycles_dfs(
                    &HypothesisId(edge.to.0.clone()),
                    visited,
                    recursion_stack,
                    path,
                    cycles,
                );
            } else if recursion_stack.contains(&edge.to.0) {
                // Found a cycle
                if let Some(start) = path.iter().position(|p| p == &edge.to.0) {
                    let cycle: Vec<HypothesisId> = path[start..]
                        .iter()
                        .chain(std::iter::once(&edge.to.0))
                        .map(|s| HypothesisId(s.clone()))
                        .collect();
                    cycles.push(cycle);
                }
            }
        }
        
        path.pop();
        recursion_stack.remove(&id);
    }

    /// Find all supporting edges for a hypothesis
    pub fn find_supporters(&self, hypothesis_id: &HypothesisId) -> Vec<&HypothesisEdge> {
        self.edges.iter()
            .filter(|e| e.to.0 == hypothesis_id.0 && e.relationship == HypothesisRelationship::Supports)
            .collect()
    }

    /// Find all contradicting edges for a hypothesis
    pub fn find_contradictions(&self, hypothesis_id: &HypothesisId) -> Vec<&HypothesisEdge> {
        self.edges.iter()
            .filter(|e| e.to.0 == hypothesis_id.0 && e.relationship == HypothesisRelationship::Contradicts)
            .collect()
    }

    /// Find all dependencies for a hypothesis
    pub fn find_dependencies(&self, hypothesis_id: &HypothesisId) -> Vec<&HypothesisEdge> {
        self.edges.iter()
            .filter(|e| e.from.0 == hypothesis_id.0 && e.relationship == HypothesisRelationship::DependsOn)
            .collect()
    }

    /// Get strongly connected components
    pub fn strongly_connected_components(&self) -> Vec<Vec<HypothesisId>> {
        // First DFS to get finish order
        let mut visited = HashSet::new();
        let mut finish_order = Vec::new();
        
        for node in &self.nodes {
            if !visited.contains(&node.hypothesis_id.0) {
                self.dfs_fill_order(&node.hypothesis_id, &mut visited, &mut finish_order);
            }
        }
        
        // Transpose the graph
        let transposed = self.transpose();
        
        // Second DFS in order of decreasing finish time
        visited.clear();
        let mut components = Vec::new();
        
        for id in finish_order.into_iter().rev() {
            if !visited.contains(&id) {
                let mut component = Vec::new();
                transposed.dfs_collect(&HypothesisId(id.clone()), &mut visited, &mut component);
                components.push(component);
            }
        }
        
        components
    }

    fn dfs_fill_order(&self, hypothesis_id: &HypothesisId, visited: &mut HashSet<String>, finish_order: &mut Vec<String>) {
        visited.insert(hypothesis_id.0.clone());
        
        for edge in self.edges.iter().filter(|e| e.from.0 == hypothesis_id.0) {
            if !visited.contains(&edge.to.0) {
                self.dfs_fill_order(&HypothesisId(edge.to.0.clone()), visited, finish_order);
            }
        }
        
        finish_order.push(hypothesis_id.0.clone());
    }

    fn dfs_collect(&self, hypothesis_id: &HypothesisId, visited: &mut HashSet<String>, result: &mut Vec<HypothesisId>) {
        visited.insert(hypothesis_id.0.clone());
        result.push(hypothesis_id.clone());
        
        for edge in self.edges.iter().filter(|e| e.from.0 == hypothesis_id.0) {
            if !visited.contains(&edge.to.0) {
                self.dfs_collect(&HypothesisId(edge.to.0.clone()), visited, result);
            }
        }
    }

    fn transpose(&self) -> Self {
        let mut transposed = Self::new();
        
        for node in &self.nodes {
            transposed.add_node(node.hypothesis_id.clone());
        }
        
        for edge in &self.edges {
            transposed.add_edge(
                edge.to.clone(),
                edge.from.clone(),
                edge.relationship,
            );
        }
        
        transposed
    }

    /// Get the topological order of hypotheses
    pub fn topological_sort(&self) -> Option<Vec<HypothesisId>> {
        let mut in_degree: HashMap<String, usize> = HashMap::new();
        let mut adjacency: HashMap<String, Vec<String>> = HashMap::new();
        
        // Initialize
        for node in &self.nodes {
            in_degree.insert(node.hypothesis_id.0.clone(), 0);
            adjacency.insert(node.hypothesis_id.0.clone(), Vec::new());
        }
        
        // Build adjacency list and in-degree count
        for edge in &self.edges {
            adjacency.entry(edge.from.0.clone())
                .or_default()
                .push(edge.to.0.clone());
            *in_degree.entry(edge.to.0.clone()).or_insert(0) += 1;
        }
        
        // Kahn's algorithm
        let mut queue: VecDeque<String> = in_degree.iter()
            .filter(|(_, &d)| d == 0)
            .map(|(id, _)| id.clone())
            .collect();
        
        let mut result = Vec::new();
        
        while let Some(node) = queue.pop_front() {
            result.push(HypothesisId(node.clone()));
            
            if let Some(neighbors) = adjacency.get(&node) {
                for neighbor in neighbors {
                    if let Some(degree) = in_degree.get_mut(neighbor) {
                        *degree -= 1;
                        if *degree == 0 {
                            queue.push_back(neighbor.clone());
                        }
                    }
                }
            }
        }
        
        if result.len() == self.nodes.len() {
            Some(result)
        } else {
            // Graph has cycles
            None
        }
    }

    /// Get graph statistics
    pub fn stats(&self) -> GraphStats {
        let support_edges = self.edges.iter()
            .filter(|e| e.relationship == HypothesisRelationship::Supports)
            .count();
        let contradict_edges = self.edges.iter()
            .filter(|e| e.relationship == HypothesisRelationship::Contradicts)
            .count();
        let depends_edges = self.edges.iter()
            .filter(|e| e.relationship == HypothesisRelationship::DependsOn)
            .count();
        let related_edges = self.edges.iter()
            .filter(|e| e.relationship == HypothesisRelationship::Related)
            .count();
        
        GraphStats {
            node_count: self.nodes.len(),
            edge_count: self.edges.len(),
            support_edges,
            contradict_edges,
            depends_edges,
            related_edges,
            cycles: self.detect_cycles().len(),
        }
    }

    /// Remove a node and all its edges
    pub fn remove_node(&mut self, hypothesis_id: &HypothesisId) -> bool {
        if let Some(&idx) = self.node_index.get(&hypothesis_id.0) {
            self.nodes.remove(idx);
            self.node_index.remove(&hypothesis_id.0);
            
            // Rebuild index
            self.node_index.clear();
            for (i, node) in self.nodes.iter().enumerate() {
                self.node_index.insert(node.hypothesis_id.0.clone(), i);
            }
            
            // Remove all edges involving this node
            self.edges.retain(|e| e.from.0 != hypothesis_id.0 && e.to.0 != hypothesis_id.0);
            
            true
        } else {
            false
        }
    }

    /// Clear all nodes and edges
    pub fn clear(&mut self) {
        self.nodes.clear();
        self.edges.clear();
        self.node_index.clear();
    }

    /// Get node count
    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    /// Get edge count
    pub fn edge_count(&self) -> usize {
        self.edges.len()
    }

    /// Check if node exists
    pub fn has_node(&self, hypothesis_id: &HypothesisId) -> bool {
        self.node_index.contains_key(&hypothesis_id.0)
    }

    /// Get node by ID
    pub fn get_node(&self, hypothesis_id: &HypothesisId) -> Option<&HypothesisNode> {
        self.node_index.get(&hypothesis_id.0)
            .and_then(|&idx| self.nodes.get(idx))
    }
}

/// ============================================================================
/// GRAPH STATISTICS
/// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphStats {
    pub node_count: usize,
    pub edge_count: usize,
    pub support_edges: usize,
    pub contradict_edges: usize,
    pub depends_edges: usize,
    pub related_edges: usize,
    pub cycles: usize,
}

/// ============================================================================
/// NODE
/// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HypothesisNode {
    pub hypothesis_id: HypothesisId,
    #[serde(default)]
    pub metadata: NodeMetadata,
}

impl HypothesisNode {
    /// Create a new node
    pub fn new(hypothesis_id: HypothesisId) -> Self {
        Self {
            hypothesis_id,
            metadata: NodeMetadata::default(),
        }
    }
}

/// Node metadata for additional information
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct NodeMetadata {
    /// Node position for visualization
    pub position: Option<(f32, f32)>,
    
    /// Custom labels
    pub labels: Vec<String>,
    
    /// Node weight for algorithms
    pub weight: f32,
}

impl NodeMetadata {
    pub fn with_position(mut self, x: f32, y: f32) -> Self {
        self.position = Some((x, y));
        self
    }
    
    pub fn with_label(mut self, label: impl Into<String>) -> Self {
        self.labels.push(label.into());
        self
    }
}

/// ============================================================================
/// EDGE
/// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HypothesisEdge {
    pub id: EdgeId,
    pub from: HypothesisId,
    pub to: HypothesisId,
    pub relationship: HypothesisRelationship,
    pub weight: f32,
}

impl HypothesisEdge {
    /// Create a new edge with supports relationship
    pub fn supports(from: HypothesisId, to: HypothesisId) -> Self {
        Self {
            id: EdgeId::new(),
            from,
            to,
            relationship: HypothesisRelationship::Supports,
            weight: 1.0,
        }
    }
    
    /// Create a new edge with contradicts relationship
    pub fn contradicts(from: HypothesisId, to: HypothesisId) -> Self {
        Self {
            id: EdgeId::new(),
            from,
            to,
            relationship: HypothesisRelationship::Contradicts,
            weight: 1.0,
        }
    }
    
    /// Create a new edge with depends_on relationship
    pub fn depends_on(from: HypothesisId, to: HypothesisId) -> Self {
        Self {
            id: EdgeId::new(),
            from,
            to,
            relationship: HypothesisRelationship::DependsOn,
            weight: 1.0,
        }
    }
    
    /// Create a new edge with related relationship
    pub fn related(from: HypothesisId, to: HypothesisId) -> Self {
        Self {
            id: EdgeId::new(),
            from,
            to,
            relationship: HypothesisRelationship::Related,
            weight: 1.0,
        }
    }
}

/// Unique edge identifier
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct EdgeId(pub String);

impl EdgeId {
    pub fn new() -> Self {
        Self(uuid::Uuid::new_v4().to_string())
    }
}

impl Default for EdgeId {
    fn default() -> Self {
        Self::new()
    }
}

/// ============================================================================
/// RELATIONSHIP
/// ============================================================================

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum HypothesisRelationship {
    /// Hypothesis A provides evidence supporting Hypothesis B
    Supports,
    
    /// Hypothesis A provides evidence contradicting Hypothesis B
    Contradicts,
    
    /// Hypothesis A depends on Hypothesis B being true
    DependsOn,
    
    /// Hypothesis A is related to Hypothesis B
    Related,
}

impl HypothesisRelationship {
    /// Check if this relationship is positive (supporting)
    pub fn is_supporting(&self) -> bool {
        matches!(self, HypothesisRelationship::Supports)
    }
    
    /// Check if this relationship is negative (contradicting)
    pub fn is_contradicting(&self) -> bool {
        matches!(self, HypothesisRelationship::Contradicts)
    }
    
    /// Get the inverse relationship
    pub fn inverse(&self) -> Self {
        match self {
            HypothesisRelationship::Supports => HypothesisRelationship::Contradicts,
            HypothesisRelationship::Contradicts => HypothesisRelationship::Supports,
            HypothesisRelationship::DependsOn => HypothesisRelationship::DependsOn,
            HypothesisRelationship::Related => HypothesisRelationship::Related,
        }
    }
}

/// ============================================================================
/// GRAPH BUILDER
/// ============================================================================

/// Builder for creating hypothesis graphs
#[derive(Debug, Clone, Default)]
pub struct GraphBuilder {
    graph: HypothesisGraph,
}

impl GraphBuilder {
    pub fn new() -> Self {
        Self {
            graph: HypothesisGraph::new(),
        }
    }
    
    /// Add a node
    pub fn add_node(mut self, hypothesis_id: HypothesisId) -> Self {
        self.graph.add_node(hypothesis_id);
        self
    }
    
    /// Add a support edge
    pub fn add_support(mut self, from: HypothesisId, to: HypothesisId) -> Self {
        self.graph.add_edge(from, to, HypothesisRelationship::Supports);
        self
    }
    
    /// Add a contradiction edge
    pub fn add_contradiction(mut self, from: HypothesisId, to: HypothesisId) -> Self {
        self.graph.add_edge(from, to, HypothesisRelationship::Contradicts);
        self
    }
    
    /// Add a dependency edge
    pub fn add_dependency(mut self, from: HypothesisId, to: HypothesisId) -> Self {
        self.graph.add_edge(from, to, HypothesisRelationship::DependsOn);
        self
    }
    
    /// Add a related edge
    pub fn add_related(mut self, from: HypothesisId, to: HypothesisId) -> Self {
        self.graph.add_edge(from, to, HypothesisRelationship::Related);
        self
    }
    
    /// Build the graph
    pub fn build(self) -> HypothesisGraph {
        self.graph
    }
}
