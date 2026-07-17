# RoBoT MCP

A Rust MCP (Model Context Protocol) server for Zed Editor — an AI agent with persistent memory, experience-based learning, and structured knowledge storage.

> **Status:** v0.1 scaffolding — database layer is solid and migrations are wired up, experience system backbone exists but references modules that don't exist yet. Zero MCP tools exposed. This is the foundation, not the product.

---

## Objective

**Problem:** Previous Python MCP memory suffered from storage bloat and slow ingestion due to embedding everything and eager graph extraction.

**Goal:** Redesign with selective storage, deferred processing, strict module boundaries, and a phased build that prioritizes reliability over premature intelligence.

### Core Principles

| Principle                   | Implementation                                                                                                      |
|-----------------------------|---------------------------------------------------------------------------------------------------------------------|
| Selective Embedding         | Score content on ingestion. Only embed high-value architectural decisions, not logs/temp data/repeated discussions  |
| Confidence + Exploration    | Track confidence ± range and exploration_value. Balance proven reliability vs. controlled experimentation           |
| Council Architecture        | No system overrides another. Modules advise via strict interfaces. Disagreements logged for policy tuning           |
| Event-Based Decisions       | Only record decisions for meaningful events (new workflows, failures, explorations). Avoid noise from trivial calls |
| Reflexes Before Imagination | Build execution/recording loop first. Defer LLM, graph, and learning modules until core is stable                   |

---

## Architecture

```
                    +----------------+                     +----------------+
                    |   RoBoT Brain  >-------------------> |   Zed Editor   |
                    +--------+-------+                     +--------v-------+
                             |          +--------+-------+          |
                             |<---------<   MCP Server   <----------+
                             |          +--------+-------+
              +--------------+--------------+
              |                             |
      +-------v--------+          +---------v---------+
      | Memory Core    |          | Experience System |
      +-------+--------+          +---------+---------+
              |                             |
    +---------+----------+          +-------+-------+
    |                    |          |                 |
    |  Memories          |          |  Recorder       |
    |  (content, types)  |          |  Coordinator    |
    |                    |          |  Pipeline       |
    |  Decisions         |          |                 |
    |  (workflow choices)|          |  Observers:     |
    |                    |          |  - Hypothesis   |
    |  Memory Sources    |          |  - Exploration  |
    |  (origin tracking) |          |  - Reflection   |
    |                    |          |  - Evolution    |
    |  Relationships     |          |                 |
    |  (graph links)     |          |  Scorer         |
    |                    |          |  Reputation     |
    |  Events            |          |                 |
    |  (timeline)        |          +-----------------+
    |  Reputations       |          |
    +--------------------+          +--------+----------+
              |                             |
              +-------------+---------------+
                            |
                    +-------v--------+
                    |    SQLite      |
                    |  Single Source |
                    |     of Truth   |
                    +----------------+
```


### Memory Layers

| Layer | Purpose | Size | Status |
|-------|---------|------|--------|
| **Index Card** (Working Memory) | Lightweight metadata: ID, Title, Summary, Keywords, Pointer | ~200-500 bytes/card | ⏳ Deferred |
| **Flat Memory** (Raw Chunks) | Original document chunks in SQLite. Only high-scoring chunks receive embeddings | Variable | ⏳ Deferred |
| **Graph Memory** | Stores relationships/facts only, never prose. Extracted async in background | Variable | ✅ Implemented (schema + tables) |

### Data Flow

1. **MCP Tools** receive requests from Zed Editor
2. **Experience System** records every action through the learning pipeline
3. **Memory Core** persists structured knowledge in SQLite
4. **Migration System** manages schema evolution automatically

---

## Database Schema

The database (`robot_brain.db`) is created automatically on first run via `SqliteStore::open()` using OS data directory resolution (`dirs` crate).

### Implemented Tables

| Table | Purpose | Created By |
|-------|---------|------------|
| `memories` | Core memory storage (content, type, confidence, importance) | Migration 0→1 |
| `decisions` | Records why workflows were chosen, alternatives considered, outcomes | Migration 1→2 |
| `memory_sources` | Tracks where each memory came from (chat, file import, user input, etc.) | Migration 2→3 |
| `relationships` | Graph connections between memories (source, target, type, strength) | `sqlite::initialize()` directly |
| `events` | Event timeline (what happened, when, what it relates to) | Migration 3→4 |
| `reputations` | Long-term reputation tracking per target | Migration 4→5 |

> **Note:** The `relationships` table is created directly in `sqlite::initialize()` via raw SQL and has no corresponding migration. If the DB is re-created from scratch it works, but on upgrade from an old database that skipped init, it won't exist until a migration path handles it.

| Model | Maps To |
|-------|---------|
| `MemoryCard` | `memories` table |
| `MemorySource` | `memory_sources` table |
| `Relationship` | `relationships` table |
| `DecisionRecord` | `decisions` table |
| `MemoryEvent` | `events` table |
| `ReputationRecord` | `reputations` table |

### Query Functions (src/database/queries.rs)

| Function | Operation |
|----------|-----------|
| `insert_memory()` | INSERT OR REPLACE into memories |
| `get_memory()` | SELECT by ID, returns Option<MemoryCard> |
| `search_memory()` | LIKE search across content, limit 100 |
| `insert_decision()` | INSERT into decisions (alternatives serialized as JSON) |
| `insert_source()` | INSERT into memory_sources |
| `insert_relationship()` | INSERT into relationships |
| `record_event()` | INSERT into events |

### Migration History

| Version | Changes |
|---------|---------|
| 0 → 1 | Core memory (`memories` table) |
| 1 → 2 | Decision memory (`decisions` table) |
| 2 → 3 | Source tracking (`memory_sources` table) |
| 3 → 4 | Event history (`events` table) |
| 4 → 5 | Reputation tracking (`reputations` table) |

### Policy Engine Config (planned)

Behavior tuning is intended to be externalized via TOML config — no implementation yet:

```toml
[policy]
experience_first = true
minimum_confidence = 30
exploration_rate = 25
avoid_high_cost_failures = true
```

### Memory Types

| Type | Description |
|------|-------------|
| `note` | General notes and observations |
| `fact` | Discrete facts (user preferences, settings) |
| `task` | Task records and their outcomes |
| `file` | File-related memories (imports, changes) |
| `conversation` | Dialogue snippets |
| `code` | Code snippets and patterns |
| `decision` | Decision records |
| `event` | System events |

---

## Experience System

The experience system tracks every action the agent takes, enabling learning over time. Modules communicate via typed structs passed through method calls (not yet event-driven — that's planned).

### Current Components

| File | Component | Status |
|------|-----------|--------|
| `experience/types.rs` | `Experience`, `ExperienceType`, `ExperienceScore`, `ReputationRecord`, `OutcomeKind`, etc. | ✅ Implemented |
| `experience/events.rs` | `ExperienceEvent` enum + `EventPayload` enum | ✅ Implemented |
| `experience/observer.rs` | `ExperienceObserver` trait (name, accepts, observe, priority) | ✅ Implemented |
| `experience/recorder.rs` | `ExperienceRecorder::record()` — inserts into DB via `ExperienceQueries` | ⚠️ Partial (see below) |
| `experience/bus.rs` | Publish/subscribe routing for events | ❌ Stub (`bus.publish(experience_id)` only) |
| `experience/queue.rs` | In-memory job queue with HashMap-backed push/pop/complete/fail | ✅ Implemented |
| `experience/worker.rs` | Spawns async worker per observer, processes jobs from channel receiver | ✅ Implemented |
| `experience/coordinator.rs` | Orchestrates full pipeline: recorder → scorer → reputation → hypothesis/exploration/reflection/evolution | ⚠️ Partial (imports resolved, but reflection/evolution stubbed) |

### Pipeline Design

```
Experience Recorded
        |
        v
    Recorder (insert_experience)
        |
        v
    Bus → Job Queue
        |
        v
    Notify Observers:
    ├── Hypothesis Engine  ✅
    ├── Exploration Engine  ✅
    ├── Reflection Engine   ⚠️ Stubbed
    └── Evolution Engine    ⚠️ Stubbed
```

### Key Types

| Component | Location | Description |
|-----------|----------|-------------|
| `Experience` | types.rs | A recorded action with context, outcome, and score |
| `ExperienceType` | types.rs | ToolExecution, MemoryLookup, Workflow, Planning, Exploration, etc. (15 variants) |
| `ExperienceScore` | types.rs | Multi-dimensional: importance, confidence, novelty, reliability |
| `ReputationRecord` | types.rs | Long-term reliability tracking per target (score, successes, failures) |
| `ExperienceObserver` | observer.rs | Trait for learning subsystems to react to events |
| `EventPayload` | events.rs | Recorded, ScoreCalculated, ReputationUpdated, ReflectionCompleted, HypothesisGenerated, ExplorationCompleted |

### Implemented Sub-Modules

All previously-planned sub-modules now exist as files:

| Module | Location | Purpose |
|--------|----------|---------|
| `scorer` | `experience/scorer.rs` | Score experiences on importance/confidence/novelty/reliability |
| `reputation` | `experience/reputation/` | Update long-term reputation for tools/workflows/models |
| `hypothesis` | `experience/hypothesis/` | Generate and track hypotheses from observations |
| `exploration` | `experience/exploration/` | Test new candidates via controlled experimentation |
| `reflection` | ⚠️ Stubbed | Analyze past experiences for patterns and improvements |
| `evolution` | ⚠️ Stubbed | Adapt behavior based on accumulated experience |

### Key Interfaces

#### Experience Observer Trait

```rust
pub trait ExperienceObserver: Send + Sync {
    fn name(&self) -> &'static str;       // Human-readable identifier
    fn start(&self) -> Result<()>;         // Initialization hook
    fn shutdown(&self) -> Result<()>;      // Cleanup hook
    fn accepts(&self, event: &ExperienceEvent) -> bool;  // Default: accept all
    fn priority(&self) -> u8;              // Lower = runs first (default: 100)
    fn observe(&self, event: &ExperienceEvent) -> Result<()>;  // Core logic
}
```

---

## Project Structure

```
robot/
src/
├── main.rs                     ✅
├── database\                   ✅
│   ├── sqlite.rs               ✅← connection + initialization
│   ├── models.rs               ✅← database structs
│   ├── migrations.rs           ✅← schema creation
│   └── queries.rs              ✅← CRUD operations
├── experience\                 ⚠️
│   ├── mod.rs                  ✅←                                    ├─ xp backbone
│   ├── types.rs                ✅← → experience data structures       ├─ xp backbone
│   ├── observer.rs             ✅← → observer contract                ├─ xp backbone
│   ├── events.rs               ✅← → ExperienceEvent + EventPayload   ├─ xp backbone
│───├── events\                 ✅← →                                  ├─ xp backbone
│   │   ├── mod.rs              ✅← →                                  ├─ xp backbone
│   │   ├── event.rs            ✅← → ExperienceEvent                  ├─ xp backbone
│   │   └── payload.rs          ✅← →EventPayload enum                 ├─ xp backbone
│   ├── bus.rs                  ✅← → publish/subscribe routing        ├─ xp backbone
│   ├── queue.rs                ✅← → queued work + retry/recovery     ├─ xp backbone
│   ├── worker.rs               ✅← → executes queued observer work    ├─ xp backbone
│   ├── coordinator.rs          ✅← → owns the whole lifecycle         ├─ xp backbone
│   ├── recorder.rs             ✅← entry point for writes experiences
│   ├── scorer.rs               ✅←
│   ├── reputation.rs           ✅←
│───├── reputation/             ✅←
│   │   ├── mod.rs	            ✅← Exposes the reputation subsystem
│   │   ├── reputation.rs       ✅← Core reputation state and updates
│   │   ├── factors.rs	        ✅← Different trust dimensions
│   │   ├── decay.rs	          ✅← Time-based reputation aging
│   │   ├── analytics.rs        ✅← Reports, trends, statistics
│   │   └── repository.rs       ✅← Save/load reputation data
│   ├── exploration.rs          ✅
│   ├── exploration/            ✅
│   │   ├── mod.rs              ✅
│   │   ├── exploration.rs      ✅
│   │   └── store.rs            ✅   <-- trait
│   │   ├── sqlite_store.rs     ✅   <-- actual database implementation
│   │   ├── vector_index.rs     ✅  <-- optional semantic search
│   │   └── analytics.rs        ✅ <-- patterns over explorations
│   ├── hypothesis.rs           ✅ Yes Core data structures
│   └── hypothesis/             ✅
│        ├── core/              ✅
│        │   ├── mod.rs         ✅define what hypothesis is
│        │   ├── hypothesis.rs  ✅ Public API and coordinator
│        │   ├── evidence.rs    ✅ Evidence models
│        │   ├── evaluator.rs   ✅ Confidence updates and evaluation logic
│        │   └── lifecycle.rs   ✅ State transitions
│        ├── services/          ✅
│        │   ├── mod.rs         ✅
│        │   ├── repository.rs  ✅ Storage interface similar to Experience/Reputation
│        │   ├── analytics.rs   🟡 Statistics and trend reporting are straightforward
│        │   ├── generator.rs   🟡 Basic pattern detection; smarter generation comes later
│        │   ├── matcher.rs     🟡 bridge between incoming experiences and existing beliefs
│        │   └── validator.rs   🟡 Contradiction checks can start simple
│        └── support/           ⚠️
│             ├── mod.rs        ⚠️
│             ├── statistics.rs 🟡 Mostly counters and summaries
│             ├── graph.rs      ⚠️ ⬅ placeholder Depends on a broader knowledge graph design
│             ├── simulation.rs ⚠️ ⬅ placeholder Requires planning/reasoning capabilities
│             └── planner.rs    ⚠️ ⬅ placeholder Depends on goals and decision-making systems
│   ├── reflection.rs           ❌
│   ├── evolution.rs            ❌
│   ├── metrics.rs              ❌
│   └── scheduler.rs            ❌
│   │   ├── evolution.rs        ❌
│   │   ├── evidence.rs         ❌        
│   │   └── metrics.rs          ❌
├── planner/                    ❌
│   ├── planner.rs              ❌
│   └── policy.rs               ❌
├── skills/                     ❌
│   └── registry.rs             ❌
├── workflows/                  ❌
│   └── engine.rs               ❌
└── learning/                   ❌
    ├── working_memory.rs       ❌
    ├── hypothesis.rs           ❌
    └── candidates.rs           ❌
```

**Legend:** ✅ Implemented | ⚠️ Stubbed/partial | ❌ Placeholder code only | 🟡 Partially done | 📋 Planned but not started

---

## Technology Stack

| Layer | Technology | Purpose |
|-------|-----------|---------|
| MCP Protocol | `rmcp` v2 | Model Context Protocol server (stdio transport) |
| Runtime | `tokio` v1 | Async runtime (`#[tokio::main]`) |
| Database | `rusqlite` v0.32 | Embedded SQLite with `load_extension` feature |
| Serialization | `serde` + `serde_json` | Data serialization (Experience, EventPayload, etc.) |
| Identity | `uuid` v1 | Unique IDs (v4) for memories and experiences |
| Time | `chrono` v0.4 | Timestamps (RFC3339) |
| File walking | `walkdir` v2 | Directory traversal for file ingestion |
| Compression | `zip` v2 | Zip archive handling |
| Hashing | `sha2` v0.10 | File content hashing |
| Paths | `dirs` v5 | OS data directory resolution |
| Error handling | `anyhow` v1 | Result propagation throughout |

---

## Getting Started

### Prerequisites

- Rust 2024 edition (per `Cargo.toml`)
- SQLite development libraries (for `rusqlite` with `load_extension`)

### Build

```bash
cargo build
```

> **Note:** The project does not currently compile. See Known Issues below.

---

## Current Status & Gaps

| Area | Status | Details |
|------|--------|---------|
| Database layer | ✅ Functional | Schema + 5 migrations (v0→v5 via `migrations.rs`), CRUD queries all implemented |
| Experience types/events | ✅ Complete | Full type system for experiences, scores, reputation, event payloads |
| Observer pattern | ✅ Implemented | Trait defined with priority and filter hooks |
| Job queue + worker | ✅ Implemented | In-memory queue with async worker (mpsc channel) |
| Event bus | ❌ Stub | Only a commented-out publish line in `bus.rs` |
| Experience coordinator | ⚠️ Partial | Pipeline logic written; all sub-modules exist but reflection/evolution are stubbed |
| Experience recorder | ⚠️ Partial | Record/success/failure methods work but reference missing `ExperienceQueries` type |
| MCP server (`app.rs`) | ❌ Stubbed | App struct defined but McpServer type doesn't exist yet |
| Main entry point | ❌ Stubbed | References `init_logging()` which isn't implemented; uses undeclared `App` type |

---

## Immediate Next Steps

1. **Create `ExperienceQueries`** — `recorder.rs` imports this from `database::queries` but it doesn't exist yet. Add the experience-specific query functions there.
2. **Implement MCP server** — Define the `McpServer` type in `app.rs` using the `rmcp` crate, register tools, handle stdio transport
3. **Wire event bus** — Replace the stub in `bus.rs` with a working publisher that connects the recorder to the observer queue via channels
4. **Implement reflection & evolution engines** — Stubbed observers need actual logic
5. **Fix main.rs** — Add `init_logging()`, import/define `App`, fix `Result<()>` return type

---

## Known Issues

- **3 compilation errors in `main.rs`**: `init_logging()` not defined, `App` type undeclared, `Result<()>` needs 2 generic params
- **No MCP tools exposed** — The server runs but has zero tools for Zed to call
- **`bus.rs` is empty** — Only a commented-out publish line. The event bus pattern is designed but not implemented
- **`queue.rs` is in-memory only** — No SQLite persistence for jobs yet
- **No `observer.rs` export** — `experience/mod.rs` only exports `coordinator`, so the observer module can't be used externally

## ⚖️ License & Fair-Pay Rule

This project is open-source, but it is also built on fairness. We believe that if the community helps improve this software, the community should share in its financial success.

### 1. For Open-Source Use (AGPL-3.0)
This project is licensed under the **GNU Affero General Public License v3.0 (AGPL-3.0)**. 
* You are completely free to use, modify, and share this code for personal or open-source projects.
* If you modify this code and run it as a cloud service or distribute it, **you must open-source your modifications** under the same AGPL-3.0 license.

### 2. For Commercial Use (Paid License)
Because many companies cannot or will not open-source their proprietary software, we offer a **Commercial License**. If a company wants to use this MCP server internally or in a closed-source product, they must purchase a commercial license from us.

### 3. The Fair-Pay Rule for Contributors
If you contribute code improvements to this project, you are an essential part of it. We do not believe in taking your work to enrich ourselves.
* **Revenue Sharing**: 100% of the net revenue generated from commercial licensing fees will be pooled and split among contributors.
* **How Payouts Work**: Payouts are distributed based on accepted code contributions (Pull Requests) and resolved GitHub issue bounties. 
* **Copyright**: By submitting a Pull Request, you maintain copyright over your code but grant us the right to include it in both the open-source AGPL-3.0 version and the paid commercial version, so we can legally sell it and pay you your share.
