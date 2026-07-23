// src/memory/mod.rs
//! Memory System - Per Architecture §4.08, §6.3
//!
//! Memory provides storage and retrieval capabilities.
//!
//! Memory contains multiple layers:
//! - Working Memory: Temporary information used during active tasks
//! - Permanent Memory: Curated knowledge retained after evaluation
//!
//! Per Architecture §6.3:
//! - Working Memory: Short lifespan, high volatility, context focused
//! - Permanent Memory: Indexed, connected, confidence weighted, relationship aware

pub mod types;
pub mod working;
pub mod permanent;
pub mod retrieval;

pub use types::{MemoryItem, MemoryType, MemoryLayer, MemoryStatus};
pub use working::WorkingMemory;
pub use permanent::PermanentMemory;
pub use retrieval::MemoryRetrieval;
