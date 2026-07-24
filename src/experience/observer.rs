// robot_mcp/src/experience/observer.rs
#![allow(dead_code)]

use anyhow::Result;

use crate::experience::events::ExperienceEvent;

/// Every learning subsystem implements this trait.
///
/// The ExperienceCoordinator publishes events, and each observer
/// decides whether the event is relevant and how to process it.
pub trait ExperienceObserver: Send + Sync {
    /// A unique, human-readable name.
    ///
    /// Used for logging, diagnostics, work queues, and recovery.
    fn name(&self) -> &'static str;

    /// Called once when the observer is started.
    ///
    /// Override if initialization is required.
    fn start(&self) -> Result<()> {
        Ok(())
    }

    /// Called once before the observer shuts down.
    ///
    /// Override to flush queues or release resources.
    fn shutdown(&self) -> Result<()> {
        Ok(())
    }

    /// Determines whether this observer is interested in an event.
    ///
    /// By default every observer accepts every event.
    fn accepts(&self, event: &ExperienceEvent) -> bool {
        let _ = event;
        true
    }

    /// Observer execution priority.
    ///
    /// Lower values run first.
    fn priority(&self) -> u8 {
        100
    }

    /// Process an event.
    fn observe(&self, event: &ExperienceEvent) -> Result<()>;
}
