// src/experience/event_handler.rs
// Event handler that processes events from the bus

use crate::experience::bus::ExperienceBus;
use crate::experience::events::types::ExperienceEvent;
use std::sync::Arc;

/// Event handler that subscribes to the event bus and processes events.
pub struct EventHandler {
    bus: Arc<ExperienceBus>,
}

impl EventHandler {
    /// Create a new event handler
    pub fn new(bus: Arc<ExperienceBus>) -> Self {
        Self { bus }
    }

    /// Start the event handler - subscribes to events and logs them.
    /// This runs in the background processing events.
    pub fn start(&self) {
        let mut receiver = self.bus.subscribe();

        tokio::spawn(async move {
            tracing::info!("Event handler started, listening for events");
            while let Ok(event) = receiver.recv().await {
                Self::handle_event(&event);
            }
            tracing::warn!("Event handler stopped - bus closed");
        });
    }

    /// Handle a single event
    fn handle_event(event: &ExperienceEvent) {
        tracing::debug!(
            "Event: {} for experience {}",
            event.event_type.name(),
            event.experience_id
        );
    }

    /// Get subscriber count for monitoring
    #[allow(dead_code)]
    pub fn subscriber_count(&self) -> usize {
        self.bus.subscriber_count()
    }
}
