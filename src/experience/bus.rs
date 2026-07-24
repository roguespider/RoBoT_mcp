// /src/experience/bus.rs
// Event bus for pub/sub communication between subsystems
#![allow(dead_code)]

use crate::experience::events::ExperienceEvent;
use anyhow::Result;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use tokio::sync::broadcast;

/// Event bus for publishing and subscribing to experience events
pub struct ExperienceBus {
    sender: broadcast::Sender<ExperienceEvent>,
    subscriber_count: Arc<AtomicUsize>,
}

impl ExperienceBus {
    /// Create a new event bus
    pub fn new() -> Self {
        let (sender, _) = broadcast::channel(100);
        Self {
            sender,
            subscriber_count: Arc::new(AtomicUsize::new(0)),
        }
    }

    /// Publish an event to all subscribers
    pub fn publish(&self, event: ExperienceEvent) -> Result<()> {
        tracing::debug!("Publishing event: {:?}", event);
        self.sender
            .send(event)
            .map_err(|e| anyhow::anyhow!("Failed to publish event: {}", e))?;
        Ok(())
    }

    /// Subscribe to events, returns a receiver
    pub fn subscribe(&self) -> broadcast::Receiver<ExperienceEvent> {
        // Create receiver BEFORE incrementing counter to avoid race
        let receiver = self.sender.subscribe();
        self.subscriber_count.fetch_add(1, Ordering::SeqCst);
        tracing::info!("New subscriber registered, total: {}", self.subscriber_count.load(Ordering::SeqCst));
        receiver
    }

    /// Unsubscribe (caller should drop their receiver)
    pub fn unsubscribe(&self) {
        let prev = self.subscriber_count.fetch_sub(1, Ordering::SeqCst);
        tracing::info!("Subscriber unsubscribed, remaining: {}", prev.saturating_sub(1));
    }

    /// Get the number of active subscribers
    pub fn subscriber_count(&self) -> usize {
        self.subscriber_count.load(Ordering::SeqCst)
    }

    /// Create a broadcast sender for external use
    pub fn sender(&self) -> broadcast::Sender<ExperienceEvent> {
        self.sender.clone()
    }
}

impl Default for ExperienceBus {
    fn default() -> Self {
        Self::new()
    }
}

/// Helper trait for publishing events
pub trait EventPublisher: Send + Sync {
    fn publish(&self, event: ExperienceEvent) -> Result<()>;
}

impl EventPublisher for ExperienceBus {
    fn publish(&self, event: ExperienceEvent) -> Result<()> {
        ExperienceBus::publish(self, event)
    }
}
