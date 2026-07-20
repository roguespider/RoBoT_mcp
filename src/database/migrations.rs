// src/database/migrations.rs

use anyhow::Result;
use rusqlite::Connection;

use crate::database::sqlite::SqliteDatabase;

/// Run all pending migrations.
pub fn run(database: &SqliteDatabase) -> Result<()> {
    let conn = database.connection()?;

    run_migrations(&conn)
}

/// Execute migration sequence.
fn run_migrations(conn: &Connection) -> Result<()> {
    create_migration_table(conn)?;

    let version = current_version(conn)?;

    match version {
        0 => {
            migration_001_initial_memory(conn)?;
            set_version(conn, 1)?;
        }

        1 => {
            migration_002_add_decision_memory(conn)?;
            set_version(conn, 2)?;
        }

        2 => {
            migration_003_add_memory_sources(conn)?;
            set_version(conn, 3)?;
        }

        3 => {
            migration_004_add_events(conn)?;
            set_version(conn, 4)?;
        }

        4 => {
            migration_005_add_reputations(conn)?;
            set_version(conn, 5)?;
        }

        5 => {
            migration_006_add_scheduled_tasks(conn)?;
            set_version(conn, 6)?;
        }

        _ => {}
    }

    Ok(())
}

// ==========================================================
// Migration tracking
// ==========================================================

fn create_migration_table(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        "
        CREATE TABLE IF NOT EXISTS schema_version (

            version INTEGER NOT NULL

        );

        INSERT INTO schema_version(version)

        SELECT 0

        WHERE NOT EXISTS
        (
            SELECT 1 FROM schema_version
        );
        ",
    )?;

    Ok(())
}

fn current_version(conn: &Connection) -> Result<i32> {
    let version = conn.query_row("SELECT version FROM schema_version", [], |row| row.get(0))?;

    Ok(version)
}

fn set_version(conn: &Connection, version: i32) -> Result<()> {
    conn.execute("UPDATE schema_version SET version=?1", [version])?;

    Ok(())
}

// ==========================================================
// Migration 001
// Core memory
// ==========================================================

fn migration_001_initial_memory(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        "
        CREATE TABLE IF NOT EXISTS memories (

            id TEXT PRIMARY KEY,

            content TEXT NOT NULL,

            memory_type TEXT NOT NULL,

            confidence REAL DEFAULT 0.5,

            importance REAL DEFAULT 0.5,

            created_at TEXT NOT NULL,

            updated_at TEXT NOT NULL

        );


        CREATE INDEX IF NOT EXISTS idx_memory_type

        ON memories(memory_type);


        ",
    )?;

    Ok(())
}

// ==========================================================
// Migration 002
// Decision memory
// ==========================================================

fn migration_002_add_decision_memory(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        "
        CREATE TABLE IF NOT EXISTS decisions (

            id TEXT PRIMARY KEY,

            task TEXT NOT NULL,

            chosen_workflow TEXT NOT NULL,

            alternatives TEXT,

            reasoning TEXT,

            result TEXT,

            success INTEGER,

            confidence REAL DEFAULT 0.5,

            created_at TEXT NOT NULL

        );


        CREATE INDEX IF NOT EXISTS idx_decision_task

        ON decisions(task);


        ",
    )?;

    Ok(())
}

// ==========================================================
// Migration 003
// Source tracking
// ==========================================================

fn migration_003_add_memory_sources(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        "
        CREATE TABLE IF NOT EXISTS memory_sources (

            id TEXT PRIMARY KEY,

            memory_id TEXT NOT NULL,

            source_type TEXT,

            source_name TEXT,

            source_location TEXT,

            created_at TEXT NOT NULL

        );


        CREATE INDEX IF NOT EXISTS idx_source_memory

        ON memory_sources(memory_id);


        ",
    )?;

    Ok(())
}

// ==========================================================
// Migration 004
// Event history
// ==========================================================

fn migration_004_add_events(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        "
        CREATE TABLE IF NOT EXISTS events (

            id TEXT PRIMARY KEY,

            event_type TEXT NOT NULL,

            description TEXT NOT NULL,

            related_id TEXT,

            created_at TEXT NOT NULL

        );


        CREATE INDEX IF NOT EXISTS idx_event_type

        ON events(event_type);


        ",
    )?;

    Ok(())
}

// ==========================================================
// Migration 005
// Reputation tracking
// ==========================================================

fn migration_005_add_reputations(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        "
        CREATE TABLE IF NOT EXISTS reputations (

            id TEXT PRIMARY KEY,

            score REAL NOT NULL,

            factors TEXT NOT NULL,

            observations INTEGER NOT NULL,

            successes INTEGER NOT NULL,

            failures INTEGER NOT NULL,

            updated_at TEXT NOT NULL,

            history TEXT NOT NULL

        );


        CREATE INDEX IF NOT EXISTS idx_reputation_score

        ON reputations(score);


        ",
    )?;

    Ok(())
}

// ==========================================================
// Migration 006
// Scheduled tasks persistence
// ==========================================================

fn migration_006_add_scheduled_tasks(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        "
        CREATE TABLE IF NOT EXISTS scheduled_tasks (

            id TEXT PRIMARY KEY,

            name TEXT NOT NULL,

            task_type TEXT NOT NULL,

            schedule TEXT NOT NULL,

            status TEXT NOT NULL,

            last_run TEXT,

            next_run TEXT,

            failure_count INTEGER DEFAULT 0,

            created_at TEXT NOT NULL

        );


        CREATE INDEX IF NOT EXISTS idx_task_status

        ON scheduled_tasks(status);

        CREATE INDEX IF NOT EXISTS idx_task_next_run

        ON scheduled_tasks(next_run);


        ",
    )?;

    Ok(())
}

