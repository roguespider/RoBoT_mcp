// src/database/queries.rs

use anyhow::Result;
use chrono::{DateTime, Utc};
use rusqlite::{Connection, params};
use uuid::Uuid;

use crate::database::models::{
    DecisionRecord, MemoryCard, MemoryEvent, MemorySource, MemoryType, Relationship,
};

// ==========================================================
// MEMORY OPERATIONS
// ==========================================================

pub fn insert_memory(conn: &Connection, memory: &MemoryCard) -> Result<()> {
    conn.execute(
        "
        INSERT OR REPLACE INTO memories
        (
            id,
            content,
            memory_type,
            confidence,
            importance,
            created_at,
            updated_at
        )
        VALUES (?1,?2,?3,?4,?5,?6,?7)
        ",
        params![
            memory.id.to_string(),
            memory.content,
            memory.memory_type.to_string(),
            memory.confidence,
            memory.importance,
            memory.created_at.to_rfc3339(),
            memory.updated_at.to_rfc3339()
        ],
    )?;

    Ok(())
}

pub fn get_memory(conn: &Connection, id: Uuid) -> Result<Option<MemoryCard>> {
    let mut stmt = conn.prepare(
        "
        SELECT
            id,
            content,
            memory_type,
            confidence,
            importance,
            created_at,
            updated_at

        FROM memories

        WHERE id=?1
        ",
    )?;

    let result = stmt.query_row([id.to_string()], |row| {
        let uuid_str: String = row.get(0)?;
        Ok(MemoryCard {
            id: Uuid::parse_str(&uuid_str).map_err(|e| rusqlite::Error::InvalidParameterName(e.to_string()))?,

            content: row.get(1)?,

            memory_type: parse_memory_type(&row.get::<_, String>(2)?),

            confidence: row.get(3)?,

            importance: row.get(4)?,

            created_at: parse_time(&row.get::<_, String>(5)?),

            updated_at: parse_time(&row.get::<_, String>(6)?),
        })
    });

    match result {
        Ok(memory) => Ok(Some(memory)),

        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),

        Err(e) => Err(e.into()),
    }
}

pub fn search_memory(conn: &Connection, text: &str, limit: usize) -> Result<Vec<MemoryCard>> {
    let pattern = format!("%{}%", text);

    let mut stmt = conn.prepare(
        "
        SELECT
            id,
            content,
            memory_type,
            confidence,
            importance,
            created_at,
            updated_at

        FROM memories

        WHERE content LIKE ?1

        ORDER BY updated_at DESC

        LIMIT ?2
        ",
    )?;

    let rows = stmt.query_map(params![pattern, limit as i64], |row| {
        let uuid_str: String = row.get(0)?;
        Ok(MemoryCard {
            id: Uuid::parse_str(&uuid_str).map_err(|e| rusqlite::Error::InvalidParameterName(e.to_string()))?,

            content: row.get(1)?,

            memory_type: parse_memory_type(&row.get::<_, String>(2)?),

            confidence: row.get(3)?,

            importance: row.get(4)?,

            created_at: parse_time(&row.get::<_, String>(5)?),

            updated_at: parse_time(&row.get::<_, String>(6)?),
        })
    })?;

    Ok(rows.collect::<Result<Vec<_>, _>>()?)
}

pub fn list_memories(conn: &Connection, memory_type: Option<&str>, limit: usize) -> Result<Vec<MemoryCard>> {
    let rows = if let Some(mem_type) = memory_type {
        let mut stmt = conn.prepare(
            "SELECT id, content, memory_type, confidence, importance, created_at, updated_at
             FROM memories WHERE memory_type = ?1 ORDER BY updated_at DESC LIMIT ?2"
        )?;
        stmt.query_map(params![mem_type, limit as i64], |row| {
            let uuid_str: String = row.get(0)?;
            Ok(MemoryCard {
                id: Uuid::parse_str(&uuid_str).map_err(|e| rusqlite::Error::InvalidParameterName(e.to_string()))?,
                content: row.get(1)?,
                memory_type: parse_memory_type(&row.get::<_, String>(2)?),
                confidence: row.get(3)?,
                importance: row.get(4)?,
                created_at: parse_time(&row.get::<_, String>(5)?),
                updated_at: parse_time(&row.get::<_, String>(6)?),
            })
        })?.collect::<Result<Vec<_>, _>>()?
    } else {
        let mut stmt = conn.prepare(
            "SELECT id, content, memory_type, confidence, importance, created_at, updated_at
             FROM memories ORDER BY updated_at DESC LIMIT ?1"
        )?;
        stmt.query_map([limit as i64], |row| {
            let uuid_str: String = row.get(0)?;
            Ok(MemoryCard {
                id: Uuid::parse_str(&uuid_str).map_err(|e| rusqlite::Error::InvalidParameterName(e.to_string()))?,
                content: row.get(1)?,
                memory_type: parse_memory_type(&row.get::<_, String>(2)?),
                confidence: row.get(3)?,
                importance: row.get(4)?,
                created_at: parse_time(&row.get::<_, String>(5)?),
                updated_at: parse_time(&row.get::<_, String>(6)?),
            })
        })?.collect::<Result<Vec<_>, _>>()?
    };

    Ok(rows)
}

// ==========================================================
// DECISION MEMORY
// ==========================================================

pub fn insert_decision(conn: &Connection, decision: &DecisionRecord) -> Result<()> {
    conn.execute(
        "
        INSERT INTO decisions
        (
            id,
            task,
            chosen_workflow,
            alternatives,
            reasoning,
            result,
            success,
            confidence,
            created_at
        )

        VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9)
        ",
        params![
            decision.id.to_string(),
            decision.task,
            decision.chosen_workflow,
            serde_json::to_string(&decision.alternatives)?,
            decision.reasoning,
            decision.result,
            decision.success,
            decision.confidence,
            decision.created_at.to_rfc3339()
        ],
    )?;

    Ok(())
}

// ==========================================================
// MEMORY SOURCES
// ==========================================================

pub fn insert_source(conn: &Connection, source: &MemorySource) -> Result<()> {
    conn.execute(
        "
        INSERT INTO memory_sources
        (
            id,
            memory_id,
            source_type,
            source_name,
            source_location,
            created_at
        )

        VALUES (?1,?2,?3,?4,?5,?6)
        ",
        params![
            source.id.to_string(),
            source.memory_id.to_string(),
            source.source_type,
            source.source_name,
            source.source_location,
            source.created_at.to_rfc3339()
        ],
    )?;

    Ok(())
}

// ==========================================================
// RELATIONSHIPS
// ==========================================================

pub fn insert_relationship(conn: &Connection, relation: &Relationship) -> Result<()> {
    conn.execute(
        "
        INSERT INTO relationships
        (
            id,
            source_id,
            target_id,
            relationship,
            strength,
            created_at
        )

        VALUES (?1,?2,?3,?4,?5,?6)
        ",
        params![
            relation.id.to_string(),
            relation.source_id.to_string(),
            relation.target_id.to_string(),
            relation.relationship,
            relation.strength,
            relation.created_at.to_rfc3339()
        ],
    )?;

    Ok(())
}

// ==========================================================
// EVENT LOG
// ==========================================================

pub fn record_event(conn: &Connection, event: &MemoryEvent) -> Result<()> {
    conn.execute(
        "
        INSERT INTO events
        (
            id,
            event_type,
            description,
            related_id,
            created_at
        )

        VALUES (?1,?2,?3,?4,?5)
        ",
        params![
            event.id.to_string(),
            event.event_type,
            event.description,
            event.related_id.map(|id| id.to_string()),
            event.created_at.to_rfc3339()
        ],
    )?;

    Ok(())
}

// ==========================================================
// HELPERS
// ==========================================================

fn parse_memory_type(value: &str) -> MemoryType {
    match value {
        "fact" => MemoryType::Fact,
        "task" => MemoryType::Task,
        "file" => MemoryType::File,
        "conversation" => MemoryType::Conversation,
        "code" => MemoryType::Code,
        "decision" => MemoryType::Decision,
        "event" => MemoryType::Event,
        "encounter" => MemoryType::Encounter,
        "experience" => MemoryType::Experience,
        _ => MemoryType::Note,
    }
}

fn parse_time(value: &str) -> DateTime<Utc> {
    DateTime::parse_from_rfc3339(value)
        .map(|t| t.with_timezone(&Utc))
        .unwrap_or_else(|_| Utc::now())
}

// ==========================================================
// SCHEDULED TASKS
// ==========================================================

use crate::experience::scheduler::{ScheduledTask, TaskSchedule, TaskStatus, TaskType};

pub fn insert_scheduled_task(conn: &Connection, task: &ScheduledTask) -> Result<()> {
    conn.execute(
        "
        INSERT OR REPLACE INTO scheduled_tasks
        (
            id,
            name,
            task_type,
            schedule,
            status,
            last_run,
            next_run,
            failure_count,
            created_at
        )
        VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9)
        ",
        params![
            task.id,
            task.name,
            serde_json::to_string(&task.task_type)?,
            serde_json::to_string(&task.schedule)?,
            serde_json::to_string(&task.status)?,
            task.last_run.map(|t| t.to_rfc3339()),
            task.next_run.map(|t| t.to_rfc3339()),
            task.failure_count,
            task.created_at.to_rfc3339()
        ],
    )?;

    Ok(())
}

pub fn get_scheduled_task(conn: &Connection, id: &str) -> Result<Option<ScheduledTask>> {
    let mut stmt = conn.prepare(
        "
        SELECT
            id,
            name,
            task_type,
            schedule,
            status,
            last_run,
            next_run,
            failure_count,
            created_at
        FROM scheduled_tasks
        WHERE id = ?1
        ",
    )?;

    let mut rows = stmt.query(params![id])?;
    
    if let Some(row) = rows.next()? {
        Ok(Some(ScheduledTask {
            id: row.get(0)?,
            name: row.get(1)?,
            task_type: serde_json::from_str(&row.get::<_, String>(2)?).unwrap_or(TaskType::Custom),
            schedule: serde_json::from_str(&row.get::<_, String>(3)?).unwrap_or(TaskSchedule::Manual),
            status: serde_json::from_str(&row.get::<_, String>(4)?).unwrap_or(TaskStatus::Scheduled),
            last_run: row.get::<_, Option<String>>(5)?.as_deref().map(parse_time),
            next_run: row.get::<_, Option<String>>(6)?.as_deref().map(parse_time),
            failure_count: row.get(7)?,
            created_at: parse_time(&row.get::<_, String>(8)?),
        }))
    } else {
        Ok(None)
    }
}

pub fn list_scheduled_tasks(conn: &Connection) -> Result<Vec<ScheduledTask>> {
    let mut stmt = conn.prepare(
        "
        SELECT
            id,
            name,
            task_type,
            schedule,
            status,
            last_run,
            next_run,
            failure_count,
            created_at
        FROM scheduled_tasks
        ORDER BY created_at DESC
        ",
    )?;

    let mut tasks = Vec::new();
    let mut rows = stmt.query([])?;
    
    while let Some(row) = rows.next()? {
        tasks.push(ScheduledTask {
            id: row.get(0)?,
            name: row.get(1)?,
            task_type: serde_json::from_str(&row.get::<_, String>(2)?).unwrap_or(TaskType::Custom),
            schedule: serde_json::from_str(&row.get::<_, String>(3)?).unwrap_or(TaskSchedule::Manual),
            status: serde_json::from_str(&row.get::<_, String>(4)?).unwrap_or(TaskStatus::Scheduled),
            last_run: row.get::<_, Option<String>>(5)?.as_deref().map(parse_time),
            next_run: row.get::<_, Option<String>>(6)?.as_deref().map(parse_time),
            failure_count: row.get(7)?,
            created_at: parse_time(&row.get::<_, String>(8)?),
        });
    }

    Ok(tasks)
}

pub fn delete_scheduled_task(conn: &Connection, id: &str) -> Result<()> {
    conn.execute("DELETE FROM scheduled_tasks WHERE id = ?1", params![id])?;
    Ok(())
}

