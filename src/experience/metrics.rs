// /src/experience/metrics.rs
// Metrics collection for performance and learning tracking
#![allow(dead_code)]

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// A single metric data point
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricPoint {
    /// Metric name
    pub name: String,

    /// Metric value
    pub value: f64,

    /// Timestamp
    pub timestamp: DateTime<Utc>,

    /// Optional labels/tags
    pub labels: HashMap<String, String>,
}

/// Aggregated metric over a time window
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AggregatedMetric {
    pub name: String,
    pub count: u64,
    pub sum: f64,
    pub min: f64,
    pub max: f64,
    pub avg: f64,
    pub std_dev: Option<f64>,
}

/// Metrics collector for tracking system performance
pub struct MetricsCollector {
    /// In-memory storage for current metrics
    metrics: Arc<RwLock<HashMap<String, Vec<MetricPoint>>>>,

    /// Counters for discrete events
    counters: Arc<RwLock<HashMap<String, u64>>>,

    /// Gauges for current values
    gauges: Arc<RwLock<HashMap<String, f64>>>,
}

impl MetricsCollector {
    /// Create a new metrics collector
    pub fn new() -> Self {
        Self {
            metrics: Arc::new(RwLock::new(HashMap::new())),
            counters: Arc::new(RwLock::new(HashMap::new())),
            gauges: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Record a metric value
    pub async fn record(&self, name: impl Into<String>, value: f64) {
        let name = name.into();
        let point = MetricPoint {
            name: name.clone(),
            value,
            timestamp: Utc::now(),
            labels: HashMap::new(),
        };

        let mut metrics = self.metrics.write().await;
        metrics.entry(name).or_insert_with(Vec::new).push(point);
    }

    /// Record a metric with labels
    pub async fn record_with_labels(
        &self,
        name: impl Into<String>,
        value: f64,
        labels: HashMap<String, String>,
    ) {
        let name = name.into();
        let point = MetricPoint {
            name: name.clone(),
            value,
            timestamp: Utc::now(),
            labels,
        };

        let mut metrics = self.metrics.write().await;
        metrics.entry(name).or_insert_with(Vec::new).push(point);
    }

    /// Increment a counter
    pub async fn increment(&self, name: impl Into<String>) {
        let name = name.into();
        let mut counters = self.counters.write().await;
        *counters.entry(name).or_insert(0) += 1;
    }

    /// Increment a counter by value
    pub async fn increment_by(&self, name: impl Into<String>, value: u64) {
        let name = name.into();
        let mut counters = self.counters.write().await;
        *counters.entry(name).or_insert(0) += value;
    }

    /// Get counter value
    pub async fn get_counter(&self, name: &str) -> u64 {
        let counters = self.counters.read().await;
        counters.get(name).copied().unwrap_or(0)
    }

    /// Set a gauge value
    pub async fn set_gauge(&self, name: impl Into<String>, value: f64) {
        let name = name.into();
        let mut gauges = self.gauges.write().await;
        gauges.insert(name, value);
    }

    /// Get gauge value
    pub async fn get_gauge(&self, name: &str) -> Option<f64> {
        let gauges = self.gauges.read().await;
        gauges.get(name).copied()
    }

    /// Get all values for a metric
    pub async fn get_metric(&self, name: &str) -> Vec<MetricPoint> {
        let metrics = self.metrics.read().await;
        metrics.get(name).cloned().unwrap_or_default()
    }

    /// Get aggregated metric
    pub async fn aggregate(&self, name: &str) -> Option<AggregatedMetric> {
        let metrics = self.metrics.read().await;
        let points = metrics.get(name)?;

        if points.is_empty() {
            return None;
        }

        let count = points.len() as u64;
        let sum: f64 = points.iter().map(|p| p.value).sum();
        let min = points.iter().map(|p| p.value).fold(f64::INFINITY, f64::min);
        let max = points
            .iter()
            .map(|p| p.value)
            .fold(f64::NEG_INFINITY, f64::max);
        let avg = sum / count as f64;

        let std_dev = if count > 1 {
            let variance: f64 =
                points.iter().map(|p| (p.value - avg).powi(2)).sum::<f64>() / (count - 1) as f64;
            Some(variance.sqrt())
        } else {
            None
        };

        Some(AggregatedMetric {
            name: name.to_string(),
            count,
            sum,
            min,
            max,
            avg,
            std_dev,
        })
    }

    /// Get all counters
    pub async fn get_all_counters(&self) -> HashMap<String, u64> {
        let counters = self.counters.read().await;
        counters.clone()
    }

    /// Get all gauges
    pub async fn get_all_gauges(&self) -> HashMap<String, f64> {
        let gauges = self.gauges.read().await;
        gauges.clone()
    }

    /// Clear old metrics (older than specified hours)
    pub async fn clear_old(&self, hours: i64) {
        let cutoff = Utc::now() - chrono::Duration::hours(hours);
        let mut metrics = self.metrics.write().await;

        for points in metrics.values_mut() {
            points.retain(|p| p.timestamp > cutoff);
        }
    }

    /// Reset all counters
    pub async fn reset_counters(&self) {
        let mut counters = self.counters.write().await;
        counters.clear();
    }

    /// Get summary of all metrics
    pub async fn summary(&self) -> MetricsSummary {
        let counters = self.get_all_counters().await;
        let gauges = self.get_all_gauges().await;

        let mut metric_summaries = HashMap::new();
        let metrics = self.metrics.read().await;

        for (name, points) in &*metrics {
            if !points.is_empty() {
                let values: Vec<f64> = points.iter().map(|p| p.value).collect();
                let sum: f64 = values.iter().sum();
                let count = values.len() as u64;
                metric_summaries.insert(
                    name.clone(),
                    AggregatedMetric {
                        name: name.clone(),
                        count,
                        sum,
                        min: values.iter().copied().fold(f64::INFINITY, f64::min),
                        max: values.iter().copied().fold(f64::NEG_INFINITY, f64::max),
                        avg: sum / count as f64,
                        std_dev: None,
                    },
                );
            }
        }

        MetricsSummary {
            counters,
            gauges,
            metrics: metric_summaries,
        }
    }
}

impl Default for MetricsCollector {
    fn default() -> Self {
        Self::new()
    }
}

/// Summary of all metrics
#[derive(Debug)]
pub struct MetricsSummary {
    pub counters: HashMap<String, u64>,
    pub gauges: HashMap<String, f64>,
    pub metrics: HashMap<String, AggregatedMetric>,
}

/// Predefined metric names for consistency
pub mod metric_names {
    // Experience metrics
    pub const EXPERIENCES_RECORDED: &str = "experiences.recorded";
    pub const EXPERIENCES_SUCCESS: &str = "experiences.success";
    pub const EXPERIENCES_FAILURE: &str = "experiences.failure";

    // Reflection metrics
    pub const REFLECTIONS_CREATED: &str = "reflections.created";
    pub const REFLECTIONS_VALIDATED: &str = "reflections.validated";
    pub const PATTERNS_DETECTED: &str = "patterns.detected";

    // Hypothesis metrics
    pub const HYPOTHESES_GENERATED: &str = "hypotheses.generated";
    pub const HYPOTHESES_CONFIRMED: &str = "hypotheses.confirmed";
    pub const HYPOTHESES_REJECTED: &str = "hypotheses.rejected";

    // Exploration metrics
    pub const EXPLORATIONS_STARTED: &str = "explorations.started";
    pub const EXPLORATIONS_COMPLETED: &str = "explorations.completed";
    pub const FINDINGS_DISCOVERED: &str = "explorations.findings";

    // Evolution metrics
    pub const BEHAVIORS_CREATED: &str = "behaviors.created";
    pub const BEHAVIORS_ACTIVATED: &str = "behaviors.activated";
    pub const BEHAVIORS_DEPRECATED: &str = "behaviors.deprecated";

    // Reputation metrics
    pub const REPUTATION_UPDATES: &str = "reputation.updates";

    // Performance metrics
    pub const PROCESSING_TIME_MS: &str = "processing.time_ms";
    pub const DATABASE_OPERATIONS: &str = "database.operations";
    pub const DATABASE_LATENCY_MS: &str = "database.latency_ms";

    // Learning metrics
    pub const INSIGHTS_GENERATED: &str = "insights.generated";
    pub const LEARNING_ITERATIONS: &str = "learning.iterations";
    pub const KNOWLEDGE_CONFIDENCE: &str = "knowledge.confidence";
}
