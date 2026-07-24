================================================================================

                 BUILDER
                   /\
                  /  \
                 /    \
       ARTISAN  /      \  PRAGMATIST
               /        \
              /ARCHITECT \
             /            \
            /______________\
      ARTIST      POET      PHILOSOPHER

================================================================================
## Table of Contents

[Chapter 01 - Vision & Philosophy](#chapter-01---vision--philosophy)
[Chapter 02 - Core Design Principles](#chapter-02---core-design-principles)
[Chapter 03 - High Level System Overview](#chapter-03---high-level-system-overview)
[Chapter 04 - Data Flow](#chapter-04---data-flow)
[Chapter 05 - Conversation Engine](#chapter-05---conversation-engine)
[Chapter 06 - Context Engine](#chapter-06---context-engine)
[Chapter 07 - Memory Engine](#chapter-07---memory-engine)
[Chapter 08 - Experience Engine](#chapter-08---experience-engine)
[Chapter 09 - Learning Engine](#chapter-09---learning-engine)
[Chapter 10 - Planning Engine](#chapter-10---planning-engine)
[Chapter 11 - Execution Engine](#chapter-11---execution-engine)
[Chapter 12 - Tool Engine](#chapter-12---tool-engine)
[Chapter 13 - Memory Hierarchy](#chapter-13---memory-hierarchy)
[Chapter 14 - Context Lifecycle](#chapter-14---context-lifecycle)
[Chapter 15 - Retrieval Pipeline](#chapter-15---retrieval-pipeline)
[Chapter 16 - Prompt Construction](#chapter-16---prompt-construction)
[Chapter 17 - Strategic Learning](#chapter-17---strategic-learning)
[Chapter 18 - Confidence System](#chapter-18---confidence-system)
[Chapter 19 - Knowledge Graph](#chapter-19---knowledge-graph)
[Chapter 20 - Storage Architecture](#chapter-20---storage-architecture)
[Chapter 21 - Database Design](#chapter-21---database-design)
[Chapter 22 - Background Workers](#chapter-22---background-workers)
[Chapter 23 - AI Contributor Operating Agreement](#chapter-23---ai-contributor-operating-agreement)
[Appendix A - Complete Workflow](#appendix-a---complete-workflow)
[Appendix B - Future Research](#appendix-b---future-research)

================================================================================

# RoBoT Architecture Specification

**Version:** 1.0  
**Status:** Living Document  
**Project:** RoBoT Cognitive Architecture

---

## Chapter 01 - Vision & Philosophy

## Vision

RoBoT is **not** a memory system.

RoBoT is a cognitive architecture whose purpose is to transform experiences into reliable knowledge and continuously improve its ability to reason, learn, plan, and adapt.

Memory is only one component.

The goal is not to remember everything.

The goal is to understand.

Traditional AI assistants primarily retrieve information.

RoBoT is designed to observe, evaluate, learn, reflect, generate hypotheses, discover patterns, build confidence, and improve its reasoning over time.

Rather than simply storing information, the system attempts to understand what information means and how trustworthy it is.

---

## Philosophy

RoBoT follows an **Experience-Centered Cognitive Architecture**.

Knowledge is not inserted into the system.

Knowledge emerges.

Every experience contributes evidence.

Evidence strengthens or weakens beliefs.

Beliefs accumulate confidence.

Highly trusted beliefs become knowledge.

Knowledge combines into models.

Models become reusable skills.

The system therefore separates:

- What happened
- What was observed
- What evidence exists
- What can reasonably be believed
- What has become reliable knowledge

This separation allows confidence to evolve naturally instead of treating every stored fact as equally trustworthy.

Learning is viewed as an ongoing scientific process rather than a database update.

---

## Learning Pipeline

```text
Experience
      │
      ▼
Observation
      │
      ▼
Evidence
      │
      ▼
Beliefs
      │
      ▼
Knowledge
      │
      ▼
Models
      │
      ▼
Skills
```

Every experience has the potential to:

- Strengthen existing knowledge
- Weaken incorrect beliefs
- Create new hypotheses
- Improve future planning
- Discover reusable skills

---

## Cognitive Architecture

```text
                    Planner
                       ▲
                       │
              Hypothesis Engine
                       ▲
                       │
Experience ───► Reflection ───► Knowledge
     │                │
     ▼                ▼
 Reputation      Exploration
     │                │
     └────────► Memory ◄────────┐
                       │         │
               Working Memory   │
                       │         │
               Permanent Memory │
                       │
                 MCP Interface
```

Memory is the library.

Experience is the teacher.

Reflection is the scientist.

Hypothesis is the inventor.

Planning is the strategist.

Reputation determines how much each source of knowledge should be trusted.

Together these systems create an AI capable of continuous learning rather than simple information retrieval.

---

## Core Design Goals

RoBoT is designed to:

- Learn from experience rather than memorization.
- Separate evidence from conclusions.
- Maintain multiple levels of confidence.
- Continuously reevaluate its own knowledge.
- Generate and test hypotheses.
- Discover reusable workflows.
- Improve planning through accumulated experience.
- Support continuous self-improvement.
- Remain modular, event-driven, and loosely coupled.
- Expose capabilities through an extensible MCP interface.

---

## Long-Term Goal

The ultimate objective of RoBoT is to build an artificial cognitive architecture capable of continuously improving itself through experience.

Rather than asking:

> "What facts should I remember?"

RoBoT asks:

> "What have I learned, how confident am I, and how should that change what I do next?"

Every subsystem in this repository exists to support that objective.

================================================================================
## Chapter 02 - Core Principles

## Purpose

The Core Principles define the foundational rules that guide every architectural decision in RoBoT.

Every subsystem, feature, and future expansion should be evaluated against these principles.

These principles exist to prevent architectural drift and ensure that RoBoT remains a coherent cognitive system as it grows.

---

# 1. Experience Over Memory

RoBoT is built around experience, not storage.

Traditional systems focus on collecting and retrieving information.

RoBoT focuses on learning from what happens.

Memory answers:

> "What information exists?"

Experience answers:

> "What happened, what did we learn, and what should change?"

Memory is a component.

Experience is the source of learning.

---

# 2. Knowledge Must Be Earned

Information is not automatically knowledge.

All information enters the system as observations, evidence, or claims.

Through evaluation, comparison, and repeated interaction, confidence increases or decreases.

Knowledge is information that has accumulated enough evidence and reliability to influence reasoning.

The system must always distinguish between:

- Data
- Observations
- Evidence
- Beliefs
- Knowledge

---

# 3. Confidence Is Multi-Dimensional

Confidence is not a simple yes or no value.

A piece of knowledge may have different confidence dimensions:

- Source reliability
- Evidence strength
- Recency
- Frequency of confirmation
- Context relevance
- Historical accuracy

RoBoT must avoid treating all information as equally reliable.

---

# 4. Everything Important Becomes an Experience

Actions, observations, decisions, successes, failures, and discoveries should create experiences.

Experiences provide the foundation for:

- Reflection
- Learning
- Hypothesis generation
- Skill improvement
- Future planning

Nothing meaningful should disappear without leaving a trace.

---

# 5. Separate Observation From Interpretation

The system must separate:

What happened.

From:

What we think it means.

Example:

Observation:

> A tool failed three times.

Interpretation:

> The tool may have an unreliable dependency.

The first is evidence.

The second is a hypothesis.

Keeping these separate prevents incorrect assumptions from becoming permanent knowledge.

---

# 6. Event-Driven Architecture

Subsystems should communicate through events whenever practical.

A subsystem should not need to understand the internal implementation of another subsystem.

Example:

Experience records an event.

Reflection observes the event.

Hypothesis evaluates possible explanations.

Knowledge updates confidence.

Each subsystem performs its role independently.

This creates a flexible architecture where new capabilities can be added without rewriting existing systems.

---

# 7. Loose Coupling, Strong Contracts

Subsystems should know what they need to accomplish, not how other systems accomplish it.

Communication should happen through:

- Traits
- Interfaces
- Events
- Defined data structures

Internal implementation details should remain private.

A component should be replaceable without rebuilding the entire system.

---

# 8. Working Memory Is Temporary

Working memory exists for active tasks.

It contains information currently being processed.

Working memory should be:

- Fast
- Flexible
- Disposable

Not everything belongs in permanent storage.

---

# 9. Permanent Memory Is Curated

Permanent memory represents accumulated knowledge.

Information should move from working memory into permanent memory only after evaluation.

Permanent memory should favor:

- Accuracy
- Relevance
- Reliability
- Historical value

Storage capacity is not the goal.

Useful understanding is the goal.

---

# 10. Reflection Drives Improvement

A system that only records experiences does not learn.

Reflection transforms experience into understanding.

Reflection asks:

- What happened?
- Why did it happen?
- Was the result expected?
- What should change?
- What should be attempted next?

Reflection is the bridge between experience and improvement.

---

# 11. Hypotheses Enable Discovery

RoBoT should not only store conclusions.

It should generate possible explanations.

A hypothesis is:

- A proposed explanation
- Supported by evidence
- Assigned confidence
- Tested through future experience

Hypotheses allow the system to explore unknown areas instead of only repeating known information.

---

# 12. Reputation Determines Trust

Not all knowledge sources deserve equal influence.

Reputation allows RoBoT to evaluate:

- Previous accuracy
- Reliability
- Consistency
- Historical performance

Trust should be earned through demonstrated reliability.

---

# 13. Learning Requires Feedback

Learning requires a cycle:

```text
Experience
    ↓
Reflection
    ↓
Hypothesis
    ↓
Action
    ↓
New Experience
    ↓
Improved Understanding
```

Without feedback, the system can only store information.

With feedback, the system can improve.

---

# 14. Design For Evolution

RoBoT is intended to grow.

Architecture decisions should consider future capabilities:

- New learning methods
- New reasoning systems
- New tools
- New knowledge sources
- New planning strategies

The goal is not only today's functionality.

The goal is creating a foundation that can evolve.

---

# 15. Simplicity Over Complexity

Advanced systems naturally become complicated.

RoBoT should add complexity only when it creates meaningful capability.

Prefer:

- Clear designs
- Small focused modules
- Explicit behavior
- Understandable systems

Avoid:

- Unnecessary abstraction
- Duplicate logic
- Hidden behavior
- Complexity without purpose

---

# Core Principle Summary

RoBoT follows these fundamental beliefs:

```text
Experience creates learning.

Evidence creates confidence.

Reflection creates understanding.

Hypotheses create discovery.

Knowledge creates capability.

Planning creates action.

Action creates new experience.
```

The architecture exists to support this continuous learning cycle.

================================================================================
## Chapter 03 - System Overview

## Purpose

This chapter defines the major components of the RoBoT architecture and explains how they interact.

RoBoT is designed as a modular cognitive system.

Each subsystem has a specific responsibility and communicates through well-defined interfaces, events, and shared data structures.

The architecture intentionally avoids a single centralized intelligence module.

Instead, intelligence emerges from the interaction between specialized systems.

---

# 1. Architectural Overview

RoBoT consists of several major cognitive subsystems:

```text
                         Planner
                            ▲
                            │
                    Hypothesis Engine
                            ▲
                            │
                  Reflection System
                            ▲
                            │
Experience ───────────────► Knowledge
     │                         ▲
     │                         │
     ▼                         │
 Exploration ─────────► Reputation
     │                         │
     ▼                         │
    Memory ◄───────────────────┘
     │
     ▼
MCP Interface
```

The major flow is:

```text
Experience
      ↓
Observation
      ↓
Reflection
      ↓
Hypothesis
      ↓
Knowledge
      ↓
Planning
      ↓
Action
      ↓
New Experience
```

This creates a continuous learning cycle.

---

# 2. Core Subsystems

## 2.1 Experience System

### Purpose

The Experience System is the foundation of learning.

It records events, observations, actions, outcomes, and environmental changes.

Experience answers:

> "What happened?"

Responsibilities:

- Record experiences
- Capture context
- Store outcomes
- Track evidence
- Provide experiences to other systems
- Trigger learning workflows

The Experience System does not decide meaning.

It records reality as observed.

---

# 2.2 Memory System

### Purpose

Memory provides storage and retrieval capabilities.

Memory answers:

> "What information do we have available?"

Memory contains multiple layers:

```text
Working Memory
      │
      ▼
Temporary Context
      │
      ▼
Permanent Memory
      │
      ▼
Knowledge Storage
```

Responsibilities:

- Store information
- Retrieve relevant context
- Maintain historical records
- Support semantic search
- Support graph relationships
- Provide information to reasoning systems

Memory supports learning but does not replace learning.

---

# 2.3 Knowledge System

### Purpose

The Knowledge System manages information that has gained sufficient confidence to influence reasoning.

Knowledge answers:

> "What do we currently believe to be reliable?"

Responsibilities:

- Maintain trusted information
- Track confidence
- Store relationships
- Manage knowledge evolution
- Connect concepts together

Knowledge is not static.

It changes as new evidence appears.

---

# 2.4 Reflection System

### Purpose

Reflection transforms experiences into understanding.

Reflection answers:

> "What does this experience mean?"

Responsibilities:

- Analyze experiences
- Identify patterns
- Compare outcomes
- Detect mistakes
- Generate insights
- Recommend changes

Reflection is the learning processor of RoBoT.

---

# 2.5 Hypothesis System

### Purpose

The Hypothesis System enables discovery.

Hypotheses represent possible explanations that have not yet become knowledge.

Responsibilities:

- Generate explanations
- Track confidence
- Compare competing ideas
- Request exploration
- Validate assumptions

A hypothesis is a temporary model waiting for evidence.

---

# 2.6 Reputation System

### Purpose

The Reputation System determines trust.

Not all information sources, methods, or knowledge paths are equally reliable.

Responsibilities:

- Track reliability
- Evaluate historical performance
- Weight evidence
- Influence confidence calculations

Reputation helps RoBoT decide:

> "How much should this be trusted?"

---

# 2.7 Exploration System

### Purpose

Exploration allows RoBoT to actively seek new information.

Exploration answers:

> "What should we investigate next?"

Responsibilities:

- Identify unknown areas
- Seek additional evidence
- Test hypotheses
- Discover opportunities
- Reduce uncertainty

Exploration prevents the system from becoming passive.

### Structure

Exploration is implemented as a directory module in `src/experience/exploration/`:

- `mod.rs` — declares submodules and re-exports
- `exploration.rs` — `Exploration` struct + `ExplorationStatus` enum
- `hypothesis.rs` — `Hypothesis` struct + `HypothesisResult` enum
- `attempt.rs` — `ExplorationAttempt` struct
- `finding.rs` — `ExplorationFinding` struct
- `store.rs` — `ExplorationRepository` trait

---

# 2.8 Planning System

### Purpose

Planning converts knowledge and goals into action.

Planning answers:

> "What should happen next?"

Responsibilities:

- Evaluate options
- Select actions
- Use learned workflows
- Predict outcomes
- Improve future decisions

Planning depends on the accumulated knowledge of the entire system.

---

# 2.9 Skills System

### Purpose

Skills represent reusable capabilities discovered through experience.

A skill is not simply stored code.

A skill represents:

- A known procedure
- A successful workflow
- Conditions where it applies
- Expected outcomes
- Confidence level

Skills allow RoBoT to improve through repetition.

---

# 2.10 MCP Interface

### Purpose

MCP is the external communication layer.

It allows external applications and tools to interact with RoBoT.

MCP is not the intelligence layer.

It is the bridge.

Responsibilities:

- Expose capabilities
- Receive requests
- Return results
- Connect external systems

The MCP layer should remain independent from internal cognitive systems.

---

# 3. System Relationships

The major relationships are:

## Experience → Reflection

Experiences provide raw material for learning.

---

## Reflection → Hypothesis

Reflection identifies possible explanations and improvements.

---

## Hypothesis → Exploration

Exploration tests uncertain ideas.

---

## Exploration → Experience

New actions create new experiences.

---

## Experience → Knowledge

Repeated evidence can increase confidence and create knowledge.

---

## Knowledge → Planning

Planning uses trusted knowledge to make decisions.

---

## Reputation → Everything

Reputation influences how much confidence each system assigns to information.

---

# 4. Architectural Philosophy

RoBoT is not built as a collection of independent tools.

It is built as a connected learning ecosystem.

Each subsystem contributes a specific capability:

```text
Memory stores.

Experience records.

Reflection interprets.

Hypothesis explores possibilities.

Exploration gathers evidence.

Knowledge preserves understanding.

Reputation evaluates trust.

Planning creates action.

Skills preserve capability.
```

Together these systems create a continuous loop:

```text
Observe
   ↓
Understand
   ↓
Predict
   ↓
Act
   ↓
Learn
   ↓
Improve
```

---

# 5. Design Outcome

The goal of this architecture is not simply to create a smarter search system.

The goal is to create a system capable of:

- Learning from experience
- Improving through feedback
- Building confidence over time
- Discovering new capabilities
- Adapting its behavior
- Becoming more effective through continued operation

RoBoT is therefore designed as a learning architecture rather than a storage architecture.

================================================================================
## Chapter 04 - High-Level Architecture

## Purpose

This chapter defines the structural architecture of RoBoT.

It describes how major systems are organized, how responsibilities are separated, and how information flows through the platform.

RoBoT is designed as a layered, event-driven cognitive architecture.

Each layer has a clear purpose and communicates through defined boundaries.

The goal is to create a system that can grow in capability without becoming tightly coupled or difficult to maintain.

---

# 4.01 Architectural Model

RoBoT follows a layered architecture:

```text
┌─────────────────────────────────────────────┐
│                 External World              │
│                                             │
│  Users • Tools • Applications • Sensors     │
└──────────────────────┬──────────────────────┘
                       │
                       ▼
┌─────────────────────────────────────────────┐
│              MCP Interface Layer            │
│                                             │
│  Commands • Requests • External Actions     │
└──────────────────────┬──────────────────────┘
                       │
                       ▼
┌─────────────────────────────────────────────┐
│              Cognitive Layer                │
│                                             │
│ Experience • Reflection • Hypothesis        │
│ Knowledge • Planning • Exploration          │
└──────────────────────┬──────────────────────┘
                       │
                       ▼
┌─────────────────────────────────────────────┐
│              Memory Layer                   │
│                                             │
│ Working Memory • Permanent Memory           │
│ Vector Search • Graph Memory • Documents    │
└──────────────────────┬──────────────────────┘
                       │
                       ▼
┌─────────────────────────────────────────────┐
│              Persistence Layer               │
│                                             │
│ SQLite • Repositories • Storage Services    │
└─────────────────────────────────────────────┘
```

Each layer has a different responsibility.

Higher layers reason.

Lower layers provide capability.

---

# 4.02 Separation of Responsibilities

A core principle of RoBoT is:

> A subsystem should only own decisions that belong to its responsibility.

Examples:

Memory should not decide what information means.

Experience should not decide what knowledge becomes trusted.

Reflection should not directly modify unrelated systems.

Planning should not directly manage storage.

Each subsystem communicates through defined interfaces.

---

# 4.03 Cognitive Layer

The cognitive layer contains the systems responsible for learning and reasoning.

```text
Experience
     │
     ▼
Reflection
     │
     ▼
Hypothesis
     │
     ▼
Knowledge
     │
     ▼
Planning
```

This layer creates intelligence.

It transforms:

Events → Understanding → Decisions

---

## 4.03.1 Experience Processing

Experience is the entry point for learning.

Responsibilities:

- Receive events
- Record observations
- Capture context
- Store outcomes
- Notify interested systems

Experience should be lightweight and factual.

It should avoid interpretation.

---

## 4.03.2 Reflection Processing

Reflection analyzes experiences.

Responsibilities:

- Compare expected vs actual results
- Identify patterns
- Detect failures
- Extract lessons
- Generate possible improvements

Reflection converts raw events into meaningful information.

---

## 4.03.3 Hypothesis Processing

Hypothesis generation creates possible explanations.

Responsibilities:

- Create theories
- Track confidence
- Associate evidence
- Request exploration
- Evaluate outcomes

Hypotheses are temporary knowledge candidates.

---

## 4.03.4 Knowledge Processing

Knowledge stores reliable understanding.

Responsibilities:

- Maintain concepts
- Manage relationships
- Track confidence
- Update beliefs
- Provide reasoning context

Knowledge is the result of successful learning cycles.

---

## 4.03.5 Planning Processing

Planning uses knowledge to determine action.

Responsibilities:

- Evaluate options
- Select workflows
- Predict outcomes
- Execute strategies

Planning is the bridge between understanding and action.

---

# 4.04 Event Architecture

RoBoT uses events as the primary communication mechanism.

Instead of:

```text
System A
    calls
System B
    calls
System C
```

RoBoT uses:

```text
System A
    │
    ▼
 Event Bus
    │
 ┌──┼─────────┐
 ▼  ▼         ▼
B   C         D
```

Benefits:

- Lower coupling
- Easier extension
- Independent processing
- Better debugging
- Better recovery after failure

Example:

```text
ExperienceRecorded

        ↓

Reflection observes

        ↓

Hypothesis evaluates

        ↓

Knowledge updates

        ↓

Reputation adjusts
```

---

# 4.05 Service Architecture

Major systems should expose services rather than exposing internal structures.

Example:

Instead of:

```text
Experience
directly edits
Database
```

Use:

```text
Experience Service

        ↓

Repository Interface

        ↓

Database
```

Benefits:

- Easier testing
- Easier replacement
- Cleaner ownership
- Reduced dependencies

---

# 4.06 Repository Pattern

Persistence is isolated through repositories.

Architecture:

```text
Cognitive System
        ↓
Service Layer
        ↓
Repository Trait
        ↓
SQLite Implementation
```

The cognitive layer should not know:

- SQL queries
- Table structure
- Connection handling

It only knows the repository contract.

---

# 4.07 Data Ownership

Every piece of information should have a clear owner.

Examples:

Experience owns:

- Events
- Observations
- Outcomes

Knowledge owns:

- Beliefs
- Concepts
- Relationships

Reputation owns:

- Trust metrics
- Reliability history

Memory owns:

- Storage and retrieval

Planning owns:

- Decisions
- Goals
- Strategies

Ownership prevents conflicting modifications.

---

# 4.08 Rust Implementation Philosophy

The Rust implementation should reflect the architecture.

Preferred structure:

```text
src/

├── experience/
├── memory/
├── knowledge/
├── reflection/
├── hypothesis/
├── reputation/
├── exploration/
├── planning/
├── skills/
├── events/
├── database/
├── repositories/
├── services/
└── mcp/
```

Each subsystem should contain:

```text
module/

├── types.rs
├── services/
├── repositories/
├── events/
└── tests/
```

The exact structure may evolve.

The architectural boundaries should remain.

---

# 4.09 Dependency Rules

Dependencies should flow downward.

Preferred:

```text
MCP
 ↓
Services
 ↓
Cognitive Systems
 ↓
Repositories
 ↓
Database
```

Avoid:

```text
Database
    ↓
Experience
    ↓
Planning
    ↓
MCP
```

Circular dependencies create fragile systems.

---

# 4.10 Growth Strategy

RoBoT is expected to evolve.

New capabilities should be added by creating new subsystems or services rather than modifying everything else.

Examples:

Adding:

- New learning methods
- New reasoning engines
- New storage methods
- New tools

should require minimal changes to existing systems.

---

# Architectural Summary

RoBoT is built around these structural ideas:

```text
Layered Architecture

+
Event-Driven Communication

+
Service Boundaries

+
Repository Isolation

+
Clear Data Ownership

+
Loose Coupling

+
Independent Evolution
```

The architecture exists to allow RoBoT to grow from a memory system into a complete cognitive platform.

================================================================================
## Chapter 05 - Data Flow

## 5.1 Overview

The system is designed around continuous information flow.

Data enters the system through observations, user interactions, external tools, and internal processes. This information is transformed through multiple stages where it is recorded, evaluated, connected, and eventually converted into improved future behavior.

The primary purpose of the data flow architecture is to ensure that information does not remain isolated. Every meaningful interaction has the potential to become an experience, every experience can generate learning signals, and every validated learning signal can improve future decisions.

The core lifecycle is:

Input
↓
Observation
↓
Experience Creation
↓
Evaluation
↓
Memory Processing
↓
Knowledge Formation
↓
Learning Update
↓
Improved Future Decisions


---

# 5.2 System Data Lifecycle

All information entering the system follows a defined lifecycle.

## Stage 1 - Input

Information enters through:

- User interaction
- MCP tools
- External applications
- Sensors or observers
- Internal system events
- Retrieved knowledge

The input layer does not determine value. It only provides raw information to the system.

---

## Stage 2 - Observation

The observation layer interprets incoming information and determines whether it represents a meaningful event.

Responsibilities:

- Detect relevant events
- Capture context
- Identify actors and resources
- Record environmental state
- Prepare information for experience creation

Observation converts raw activity into structured events.

---

## Stage 3 - Experience Creation

The Experience System records meaningful interactions.

An experience contains:

- What happened
- When it happened
- Context surrounding the event
- Actions performed
- Result achieved
- Confidence level
- Associated evidence

Experiences are the foundation of system improvement.

The system does not learn directly from raw data.
It learns from evaluated experiences.

---

# 5.3 Experience Processing Flow

The experience pipeline follows:

Event
↓
Experience Recorder
↓
Experience Storage
↓
Scoring
↓
Reflection
↓
Learning Signals


## Experience Recorder

Responsible for creating durable records from events.

Captures:

- Event details
- Context
- Outcome
- Relationships
- Source information


## Experience Scoring

Each experience receives evaluation.

Scoring considers:

- Success or failure
- Reliability
- Confidence
- Relevance
- Long-term usefulness


## Reflection

Reflection analyzes completed experiences.

Questions:

- What happened?
- Why did it happen?
- What worked?
- What failed?
- What should change?


Reflection produces improvement signals for future behavior.

---

# 5.4 Memory Flow

Memory stores information that may be useful later.

The memory pipeline:

Information
↓
Extraction
↓
Classification
↓
Indexing
↓
Storage
↓
Retrieval
↓
Context Assembly


## Memory Processing

Information may become:

- Working Memory
- Long-Term Memory
- Connected Knowledge
- Experience Reference


Memory does not represent understanding by itself.

Memory provides access.
Knowledge provides meaning.

---

# 5.5 Knowledge Formation Flow

Knowledge is created through validation and connection.

The knowledge pipeline:

Stored Information
↓
Relationship Detection
↓
Validation
↓
Confidence Assignment
↓
Knowledge Update


Knowledge contains:

- Concepts
- Relationships
- Dependencies
- Procedures
- Verified information


Knowledge confidence changes over time based on:

- New evidence
- Successful usage
- Contradicting information
- Validation results

---

# 5.6 Learning Flow

Learning transforms experiences into system improvements.

The learning cycle:

Experience
↓
Evaluation
↓
Reflection
↓
Hypothesis Generation
↓
Validation
↓
Knowledge Update
↓
Behavior Improvement


The system improves by continuously comparing expected outcomes against actual outcomes.

Learning is not the storage of more information.

Learning is the improvement of future decisions.

---

# 5.7 Decision Flow

When performing a task, the system follows:

Goal
↓
Planning
↓
Memory Retrieval
↓
Knowledge Retrieval
↓
Experience Retrieval
↓
Confidence Evaluation
↓
Action Selection
↓
Execution
↓
Outcome Recording


Before selecting an action, the system evaluates:

- Previous experience
- Available knowledge
- Confidence levels
- Expected outcomes
- Potential risks


The result becomes a new experience, creating a continuous feedback loop.

---

# 5.8 Feedback Loop

The system operates as a closed learning loop.

    Experience
         |
         v
   Evaluation
         |
         v
    Reflection
         |
         v
   Improvement
         |
         v
    Future Action
         |
         v
    New Experience



Every completed operation provides an opportunity for refinement.

The system continuously transforms past activity into future capability.

---

# 5.9 Data Boundaries

Each subsystem has defined ownership.

## Experience System

Owns:

- Events
- Outcomes
- Historical interactions
- Evaluation results


## Memory System

Owns:

- Stored information
- Retrieval indexes
- Context data


## Knowledge System

Owns:

- Validated concepts
- Relationships
- Dependencies


## Learning System

Owns:

- Improvements
- Hypotheses
- Adaptation signals


No subsystem should directly modify another subsystem's internal data.

Communication occurs through defined interfaces and events.

---

# 5.10 Event Driven Communication

Subsystem communication is based on events.

Examples:

ExperienceRecorded
|
v
ExperienceScored
|
v
ReflectionCompleted
|
v
HypothesisGenerated
|
v
KnowledgeUpdated



Events allow the system to remain modular while maintaining a continuous information pipeline.

---

# 5.11 Summary

The RoBoT architecture is built around a continuous transformation process:

Information becomes experience.

Experience becomes evaluation.

Evaluation becomes learning.

Learning becomes improved behavior.


The data flow architecture provides the foundation that connects all major subsystems and allows the system to evolve over time.

================================================================================
## Chapter 06 - Subsystem Architecture

## 6.1 Overview

The system is composed of specialized subsystems that cooperate through defined boundaries.
Each subsystem owns a specific responsibility and communicates through explicit contracts.

The primary subsystems are:

- Experience System
- Memory System
- Knowledge System
- Learning System
- Hypothesis System
- Reflection System
- Reputation System
- Planning System
- MCP Integration Layer
- Storage Layer

## 6.2 Experience System

The Experience System is responsible for recording, evaluating, and improving from interactions.

Responsibilities:

- Capture events
- Record outcomes
- Score results
- Track confidence
- Build experience history
- Generate learning signals

Core principle:

Experiences are not memories.
An experience is an event with context, outcome, and evaluation.

## 6.3 Memory System

The Memory System stores information that can be retrieved and reused.

Memory is divided into:

### Working Memory

Temporary information used during active tasks.

Characteristics:

- Short lifespan
- High volatility
- Context focused


### Permanent Memory

Curated knowledge retained after evaluation.

Characteristics:

- Indexed
- Connected
- Confidence weighted
- Relationship aware

## 6.4 Knowledge System

The Knowledge System represents structured understanding.

It contains:

- Concepts
- Relationships
- Dependencies
- Prerequisites
- Verified information

Knowledge is not raw storage.
Knowledge is information that has survived evaluation.

## 6.5 Learning System

The Learning System transforms experience into improvement.

Pipeline:

Experience
    ↓
Evaluation
    ↓
Reflection
    ↓
Hypothesis
    ↓
Validation
    ↓
Knowledge Update

## 6.6 Hypothesis System

The Hypothesis System allows the agent to reason about unknowns.

Responsibilities:

- Generate possible explanations
- Track confidence
- Compare evidence
- Test assumptions
- Retire failed hypotheses

## 6.7 Reflection System

Reflection analyzes completed experiences.

Questions:

- What happened?
- Why did it happen?
- What worked?
- What failed?
- What should change?

## 6.8 Reputation System

The Reputation System tracks reliability.

It evaluates:

- Success history
- Prediction accuracy
- Workflow reliability
- Information confidence
- Source quality

## 6.9 MCP Integration Layer

The MCP layer provides controlled communication between the agent and external tools.

Responsibilities:

- Tool discovery
- Tool execution
- Data exchange
- Permission boundaries
- External integrations

## 6.10 Storage Layer

Storage provides persistence.

Primary components:

- SQLite relational storage
- Vector indexes
- Graph relationships
- Experience records
- Knowledge records

================================================================================
## Chapter 07 - Experience Engine Design

07.01 Purpose

introduction explaining why an Experience Engine exists.
Design philosophy and why experience is separate from memory and knowledge.
The complete lifecycle of an experience from observation to archival.
Every major component with diagrams.
Data flow between observer, events, evidence, scorer, reputation, hypothesis, reflection, repository, analytics, and coordinator.
Design decisions and why alternatives were rejected.
Rust module mapping.
Database schema concepts.
Concurrency model.
Failure handling.
Confidence mathematics.
Examples that walk through a real experience from beginning to end.
Design invariants.
Future expansion points.

07.02 Design Goals

07.03 High-Level Architecture

07.04 Experience Lifecycle

07.05 Observation Pipeline

07.06 Event Processing

07.07 Evidence Collection

07.08 Experience Construction

07.09 Scoring & Confidence

07.10 Reputation Integration

07.11 Hypothesis Generation

07.12 Reflection

07.13 Promotion to Knowledge

07.14 Storage Architecture

07.15 Query & Retrieval

07.16 Analytics

07.17 Future Extensions

## Design Invariants

• Every experience originates from one or more observations.
• Experiences are immutable once committed.
• Confidence is updated through evidence, never manually.
• Reflection creates new experiences rather than modifying old ones.
• Promotion to Knowledge requires validation.
• Historical data is never destroyed, only archived.

================================================================================
## Chapter 08 - Memory Architecture

Chapter 08 - Knowledge System

Experience and Memory are different.

Knowledge deserves its own chapter.

Topics:

Knowledge lifecycle
Facts
Concepts
Skills
Relationships
Confidence
Versioning
Validation
Promotion from Experience


================================================================================
## Chapter 09 - Learning Pipeline

Input
↓
Observation
↓
Memory
↓
Experience
↓
Knowledge
↓
Planning
↓
Decision
↓
Action
↓
Reflection

================================================================================
## Chapter 10: Hypothesis and Reasoning

Chapter 10 - Planning

Separate planning from decision making.

Planning covers

long-term goals
task decomposition
scheduling
dependencies
replanning

================================================================================
## Chapter 11: MCP and External Interfaces

Chapter 11 - Reflection

One of the biggest pieces that makes an agent feel intelligent.

Discuss

failures
successes
hindsight
lesson extraction
confidence updates
hypothesis creation

================================================================================
## Chapter 12: Database Schema

Chapter 12 - Learning

Different from memory.

Learning answers

"How do I become better?"

Include

reinforcement
generalization
abstraction
transfer learning
forgetting
reputation updates

================================================================================
## Chapter 13: Rust Implementation Guidelines

Chapter 13 - Personality

Even if minimal.

Include

speaking style
preferences
humor
curiosity
emotional weighting
interaction policies

This keeps personality from leaking into core cognition.

================================================================================
## Chapter 14: Planning

Chapter 14 - World Model

One of the biggest missing pieces.

RoBoT should eventually understand
Objects
Places
People
Events
Time
Goals
Relationships
Resources

Basically
"How the world works."
Memory stores facts.
World Model stores understanding.

================================================================================
## Chapter 15: Skills

Chapter 15 - Skills

Separate from knowledge.

Knowledge:

"I know SQL."

Skill:

"I can optimize a query."

Include

prerequisites
mastery
decay
practice
execution metrics

================================================================================
## Chapter 16: Database

Chapter 16 - Safety

Very important.

Things like

sandboxing
permission checks
confidence thresholds
rollback
hallucination handling
uncertainty reporting

================================================================================
## Chapter 17: MCP

Chapter 17 - Performance

Future-you will thank present-you.

Document

threading
queues
async
caching
database strategy
indexing
batching
memory limits

================================================================================
## Chapter 18: Services

Chapter 18 - Future Roadmap

Ideas that shouldn't clutter architecture but shouldn't disappear either.

Examples

distributed memory
multiple robots
swarm learning
cloud synchronization
multimodal perception
robotics integration

================================================================================
## Chapter 19: Repositories

Chapter 19 - AI Coding Standards

Rather than scattered rules, have one chapter that every coding agent must obey.

Examples:
No duplicated logic.
Composition over inheritance.
Keep files under roughly 500 lines when practical.
Public APIs remain stable.
Document every module.
Every subsystem owns its data.
Avoid circular dependencies.
Prefer deterministic behavior.
Never bypass coordinator layers.
Every async task must be cancel-safe.
Every database migration is reversible.
Every feature includes tests when feasible.
No hidden global state.

================================================================================
## Chapter 20: Coding Standards

Chapter 20 - Philosophy

Not technical.
Why does RoBoT exist?
What principles guide every design decision?
Something like
RoBoT is designed to become more competent through accumulated experience rather than through hard-coded behavior.
Those kinds of statements become tie-breakers when architecture choices compete.

================================================================================
## Chapter 21: AI Development Workflow

================================================================================
## Chapter 22: Roadmap

================================================================================
## Chapter 23: Future Research

================================================================================
## Chapter 24:

================================================================================
## Chapter 25:



---------------------------------------------------------------------------------------------------------
\# AI Agent Instructions

This document is the authoritative architecture for this repository.

Every coding assistant working on this project must:

\- Read this document before making changes.
\- Prefer these architectural rules over existing inconsistent code.
\- Modify related files together rather than in isolation.
\- Optimize for long-term maintainability.
\- Avoid placeholder implementations.
\- Keep subsystems loosely coupled.
\- Maintain compile-ready Rust whenever practical.

Before making any changes:

1\. Read ARCHITECTURE.md completely.
2\. Treat it as the authoritative specification for this repository.
3\. Follow its architecture, naming conventions, dependency rules, and design principles.
4\. If existing code conflicts with ARCHITECTURE.md, prefer the architecture unless it would introduce compilation errors.
5\. Read all files related to this subsystem before making changes.
6\. Implement the entire subsystem, not just the requested file.
7\. Keep the architecture internally consistent.
8\. When finished, summarize:

&#x20;  - files modified
&#x20;  - architectural improvements
&#x20;  - remaining work
&#x20;  - assumptions made

now summarize ARCHITECTURE.md in your own words.

List:

\- the major subsystems
\- the event flow
\- repository conventions
\- dependency rules
\- coding standards

Only after doing that should you begin modifying code.

You are the lead software engineer for this project.
Your job is NOT to answer questions.
Your job is to COMPLETE the project.

=========================================================
MISSION
===

Treat this repository as a professional open-source project.
Do not produce placeholder code unless absolutely unavoidable.
Every module should be production quality.
Always think about the entire architecture before modifying files.
If a better design requires restructuring folders or moving code, do it.
Avoid unnecessary complexity, but never sacrifice maintainability.

=========================================================
WORKFLOW
===

Before writing code:

1. Read the entire repository.
2. Understand every module.
3. Build an internal dependency graph.
4. Find architectural inconsistencies.
5. Determine the cleanest design.

Then implement.
Do NOT ask permission every few files.
Complete as much work as possible in one pass.

=========================================================
WHEN WORKING ON A SUBSYSTEM
===

For the subsystem requested:
• identify every related file
• identify missing modules
• identify duplicate logic
• identify dead code
• identify poor abstractions
• identify cyclic dependencies
• identify naming inconsistencies
Then improve everything together.
Do not only modify the requested file if neighboring files should change.

=========================================================
CODE QUALITY
===

Every public type should have documentation.
Every important function should explain WHY it exists.
Prefer traits over duplication.
Prefer composition over inheritance.
Prefer immutable data.
Avoid global state.
Avoid magic numbers.
Avoid unwrap().
Use anyhow or thiserror where appropriate.
Return meaningful Results.
Use strong typing instead of strings whenever practical.

=========================================================
RUST STYLE
===

Prefer idiomatic Rust.
Small focused modules.
Small functions.
Minimal allocations.
Iterator chains when readable.
Avoid unnecessary cloning.
Use Arc only when ownership requires it.
Keep ownership simple.

=========================================================
PROJECT GOALS
===

The project is a self-learning AI.

Core systems include:
Experience
Memory
Knowledge
Hypothesis
Reflection
Planning
Skills
Reputation
Exploration
Learning
Events
Coordinator
Repositories
SQLite persistence
MCP interface
The architecture should be event driven.
Subsystems should remain loosely coupled.

=========================================================
WHEN ADDING CODE
===

Always ask:
What else should exist?
What is missing?
What future feature will need this?
Can this become reusable?
Can this become a service?
Can this become a trait?

=========================================================
WHEN FINISHED
===

Do NOT simply say "done."

Instead produce:

1. Files modified
2. Files created
3. Architectural improvements
4. Remaining technical debt
5. Suggestions for the next subsystem
6. Any assumptions made

=========================================================
RULE
===

Optimize for completing the entire project, not minimizing code changes.
Think like the project's CTO, not a code assistant.

=========================================================
MCP WORKFLOW RULES (MANDATORY)
===

This section defines the REQUIRED workflow for any AI agent using RoBoT's MCP interface.

# 1. CONSULT MCP BEFORE ANY ACTION

**CRITICAL**: Before taking ANY action, the agent MUST:

1. Call `list_tools` to see available MCP tools
2. Consult relevant memory using `search_memory` or `global_search`
3. Review any stored patterns using `get_patterns` or `analyze_patterns`
4. Only then proceed with actions

**NEVER** skip the memory consultation step.

---

# 2. FILE INGESTION WORKFLOW

When ingesting files, follow this EXACT sequence:

## Step 1: Check Available Files
```
Call: list_importable
This returns: files in files_to_import/ (in exe directory)
```

## Step 2: Ingest ONE File at a Time
```
Call: ingest_files with:
  - folder: "files_to_import" (or omit - it's the default)
  - limit: 1 (REQUIRED for single file mode)
  OR
  - file_path: "exact/path/to/file.txt"
```

**IMPORTANT**:
- Always use `limit=1` for single file ingestion
- NEVER batch ingest multiple files without explicit user instruction
- Each file should be ingested, processed, and verified before the next

## Step 3: Verify Ingestion Success
```
Check the response for:
  - success: true
  - chunks_created: > 0
  - remaining_in_temp: should be 0 or handled
```

## Step 4: Ask User Before Deletion
```
NEVER delete files without explicit user confirmation.
Ask: "Can I delete the original file X? It has been successfully ingested."
```

## Step 5: Delete Only After Confirmation
```
Call: delete_ingested_files with:
  - files: [list of file paths to delete]
  - confirmation: "yes" (EXACTLY this value)
```

---

# 3. DIRECTORY STRUCTURE

The following files MUST be in the SAME directory as the executable:

```
robot_brain/          (or wherever exe is located)
├── robot_brain.exe   (or robot_brain on Linux)
├── robot_brain.db    (SQLite database)
└── files_to_import/  (import folder)
    ├── file1.txt
    ├── file2.md
    └── ...
```

**The MCP will report `files_to_import` as the default import location.**

---

# 4. DELETE VERIFICATION RULES

**ABSOLUTE RULES**:

1. NEVER delete files without user confirmation
2. ALWAYS show what files will be deleted before calling delete_ingested_files
3. The confirmation parameter MUST be exactly "yes" (case-insensitive)
4. If confirmation is missing or wrong, deletion will NOT proceed
5. Original folders are NOT deleted automatically - only files

---

# 5. PATTERN ANALYSIS WORKFLOW

Before making repetitive decisions:

```
1. Call: analyze_patterns
2. Review returned patterns, themes, and recommendations
3. Consider pattern confidence scores
4. Apply learned patterns to current situation
```

---

# 6. DATABASE CONCURRENCY

RoBoT uses SQLite with WAL mode for better concurrency:
- Multiple readers can run simultaneously with one writer
- Busy timeout is set to 30 seconds
- If you encounter "database is locked", wait and retry

---

# 7. ERROR HANDLING

When operations fail:

1. Check the `error` field in the response
2. Log the error for debugging
3. Report the error clearly to the user
4. Do NOT silently skip errors
5. Do NOT retry indefinitely without user input

---

# QUICK REFERENCE: MCP TOOL USAGE

| Tool | When to Use | Key Parameters |
|------|-------------|----------------|
| list_importable | Before ingestion, check available files | folder, limit |
| ingest_files | Ingest files into memory | folder, file_path, limit=1 |
| list_ingested_files | List files that can be deleted | folder, limit |
| delete_ingested_files | Delete originals (NEEDS CONFIRMATION) | files, confirmation="yes" |
| search_memory | Search stored memories | query, types, limit |
| global_search | Search all data types | query, limit |
| analyze_patterns | Detect patterns in experiences | experience_ids |
| get_patterns | Get stored patterns | min_confidence, pattern_type |
| get_insights | Get actionable insights | min_confidence, limit |

---

This MCP workflow section is MANDATORY for all agents using RoBoT.
