# RoBoT Project — Context Summary

> Generated from files in `readme/` directory.
> This file exists so future sessions can quickly understand where we left off.

---

## 1. Project Overview

**Language:** Rust
**Framework:** MCP (Model Context Protocol) server
**Database:** SQLite via `rusqlite` with `bundled` feature
**Key dependency:** `rusqlite = { version = "0.31.0", features = ["bundled"] }`

---

## 2. Architecture: Experience System

The project is building an event-driven "experience system" — essentially a nervous system for an AI agent (RoBoT). The core philosophy is modular, decoupled components that communicate through events rather than direct calls.

### The Data Model Hierarchy

```
Encounter  = What happened (raw observation)
    ↓
Reflection = Think about it (analyze encounters)
    ↓
Experience = What was learned (derived from reflection)
    ↓
Insight    = Reusable knowledge (matured experience)
    ↓
Hypothesis = Possible explanation (testable belief)
    ↓
Exploration = Test it (intentional investigation)
    ↓
Evolution  = Improve behavior (long-term adaptation)
```

### Key Design Decisions

- **No subsystem overrides another** — each module observes and acts independently
- **Recorder-centric design** — all subsystems funnel through `ExperienceRecorder` → `ExperienceCoordinator` → Bus → Queues → Workers → Observers
- **Event-driven, not blocking** — everything is asynchronous via channels and queues
- **Crash recovery** — work is persisted in SQLite; workers resume from `Pending` state on restart

---

## 3. File Structure (Planned / In Progress)

```
experience/
├── mod.rs                    # Module declarations (needs all submodules)
├── types.rs                  # Core types: Encounter, Experience, ExperienceScore, ReputationRecord, etc.
├── events.rs                 # ExperienceEvent, EventPayload enum
├── observer.rs               # ExperienceObserver trait (name, start, shutdown, accepts, priority, observe)
├── coordinator.rs            # ExperienceCoordinator — orchestrates the flow
├── bus.rs                    # Publish/subscribe — broadcasts events
├── queue.rs                  # Work queue + Job/JobStatus enums
├── worker.rs                 # ExperienceWorker — async loop processing queued jobs
├── encounter_recorder.rs     # Records Encounters (renamed from recorder.rs)
├── scorer.rs                 # ExperienceScorer — calculates importance, confidence, novelty, reliability
├── reputation.rs             # Reputation tracking per target (tool, workflow, model, etc.)
│
├── exploration/
│   ├── mod.rs
│   ├── exploration.rs        # Exploration, Hypothesis, ExplorationAttempt, ExplorationFinding structs
│   └── store.rs              # ExplorationRepository trait
│
├── reflection/               # Not yet built
├── hypothesis/               # Not yet built
├── evolution/                # Not yet built
└── metrics/                  # Not yet built

database/
├── sqlite.rs                 # SqliteStore — opens/creates memories.db, initializes tables
└── migrations.rs             # Schema migrations

mcp_bridge/
├── server.rs                 # MCP server
└── infrastructure/           # IndexCard, MemoryType

main.rs / app.rs              # Entry point: init logging → open DB → migrate → run server
```

---

## 4. Key Types (from types.rs)

### Experience
```rust
pub struct Experience {
    pub id: String,
    pub timestamp: DateTime<Utc>,
    pub experience_type: ExperienceType,
    pub title: String,
    pub description: String,
    pub context: ExperienceContext,
    pub outcome: ExperienceOutcome,
    pub score: Option<ExperienceScore>,
    pub tags: Vec<String>,
    pub metadata: HashMap<String, String>,
}
```

### Encounter (new — replaces old "Experience" as the raw event type)
```rust
pub struct Encounter {
    pub id: String,
    pub timestamp: DateTime<Utc>,
    pub experience_id: Option<String>,
    pub context: ExperienceContext,
    pub input: String,
    pub action: String,
    pub result: EncounterResult,
    pub metadata: HashMap<String, String>,
}
```

### ExperienceScore
```rust
pub struct ExperienceScore {
    pub importance: f32,
    pub confidence: f32,
    pub novelty: f32,
    pub reliability: f32,
}
```

### KnowledgeMaturity
```rust
pub enum KnowledgeMaturity {
    Emerging, Developing, Established, Trusted,
    Questioned, Deprecated, Rejected,
}
```

### ExperienceType
```rust
pub enum ExperienceType {
    ToolExecution, MemoryLookup, MemoryStore, Workflow,
    Planning, Exploration, Hypothesis, Reflection,
    Learning, Conversation, UserFeedback, ModelInference,
    Error, System, Custom(String),
}
```

### ReputationRecord
```rust
pub struct ReputationRecord {
    pub target: ReputationTarget,
    pub score: f32,
    pub successes: u64,
    pub failures: u64,
    pub last_updated: DateTime<Utc>,
}
```

---

## 5. Event System

### ExperienceEvent
```rust
pub struct ExperienceEvent {
    pub id: Uuid,
    pub experience_id: String,
    pub kind: EventKind,
    pub timestamp: DateTime<Utc>,
}

pub enum EventKind {
    ExperienceRecorded,
    ScoreCalculated,
    ReputationUpdated,
    ReflectionCompleted,
    HypothesisCreated,
    ExplorationCompleted,
    Archived,
    Deleted,
}
```

### EventPayload (alternative design with payloads)
```rust
pub enum EventPayload {
    ExperienceRecorded,
    ExperienceUpdated,
    ExperienceArchived,
    ExperienceDeleted,
    ScoreCalculated { score: f32 },
    ReputationUpdated { previous: f32, current: f32 },
    ReflectionCompleted { reflection_id: Uuid },
    HypothesisGenerated { hypothesis_id: Uuid },
    ExplorationCompleted { exploration_id: Uuid },
    ObserverStarted { observer: String },
    ObserverStopped { observer: String },
    ObserverFailed { observer: String, error: String },
    ProcessingFailed { stage: String, error: String },
}
```

---

## 6. Observer Pattern

```rust
pub trait ExperienceObserver: Send + Sync {
    fn name(&self) -> &'static str;
    fn start(&self) -> Result<()> { Ok(()) }
    fn shutdown(&self) -> Result<()> { Ok(()) }
    fn accepts(&self, event: &ExperienceEvent) -> bool { true }
    fn priority(&self) -> u8 { 100 }
    fn observe(&self, event: &ExperienceEvent) -> Result<()>;
}
```

---

## 7. Worker / Queue System

```rust
pub struct ObserverJob {
    pub id: String,
    pub event: ExperienceEvent,
    pub attempts: u32,
}

pub enum JobStatus {
    Pending,
    Running,
    Complete,
    Failed,
}
```

SQLite table `experience_jobs`:
- id, experience_id, observer, status, priority, attempts, last_error, created_at, started_at, completed_at

---

## 8. Current State / Where We Left Off

### Completed (designed):
- `types.rs` — all core data structures
- `events.rs` — event types and payloads
- `observer.rs` — ExperienceObserver trait
- `explanation/exploration.rs` — Exploration data model
- `exploration/store.rs` — repository trait
- Database schema (SQLite with nodes/edges tables)

### Needs Attention:
- **`mod.rs`** — only exports `coordinator`, needs `pub mod` for all submodules
- **Pseudo-code in source files** — several files have design notes mixed in as code (bus.rs, encounter_recorder.rs, scorer.rs, events.rs, observer.rs, worker.rs, repository.rs) — must be converted to comments or proper Rust
- **Missing types referenced in queue.rs** — references `Job` and `JobStatus` from types.rs but they don't exist there
- **main.rs** — references `App`, `init_logging()` which don't exist yet

### Known Compilation Issues:
1. Missing module declarations in `mod.rs`
2. Pseudo-code/non-Rust text in multiple files
3. Missing type definitions (`Job`, `JobStatus`, `App`, `init_logging`)
4. `bus.rs` contains bare non-Rust prose

---

## 9. Architecture Philosophy

- **Encounter Recorder** captures raw events (what happened)
- **Experience Coordinator** orchestrates the pipeline
- **Bus** broadcasts events to interested observers
- **Queues** decouple producers from consumers
- **Workers** process queued jobs asynchronously
- **Observers** react to events independently (Scorer, Reflection, Reputation, Hypothesis, Exploration)
- **No direct inter-module coupling** — everything flows through the experience pipeline

---

## 10. Next Steps (When Returning)

1. Fix `mod.rs` — add all missing `pub mod` declarations
2. Clean pseudo-code from source files — convert design notes to comments
3. Add missing types to `types.rs` — `Job`, `JobStatus`
4. Fix `queue.rs` — correct type references
5. Clean `bus.rs` — replace prose with proper Rust stub
6. Run `cargo check` to see real vs. noise errors
7. Build out `reflection/` and `hypothesis/` submodules
8. Implement `App` pattern (or keep simple main.rs for now)
