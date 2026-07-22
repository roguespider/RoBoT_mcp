// /src/experience/reputation/repository.rs

use anyhow::{Context, Result};
use rusqlite::{Connection, params};

use super::reputation::Reputation;

use serde_json;

pub struct ReputationRepository<'a> {
    conn: &'a Connection,
}

impl<'a> ReputationRepository<'a> {
    pub fn new(conn: &'a Connection) -> Self {
        Self { conn }
    }

    pub fn save(&self, reputation: &Reputation) -> Result<()> {
        let factors = serde_json::to_string(&reputation.factors)
            .context("failed serializing reputation factors")?;

        let history = serde_json::to_string(&reputation.history)
            .context("failed serializing reputation history")?;

        self.conn
            .execute(
                "
            INSERT INTO reputations (
                id,
                score,
                factors,
                observations,
                successes,
                failures,
                updated_at,
                history
            )
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)

            ON CONFLICT(id)
            DO UPDATE SET

                score = excluded.score,
                factors = excluded.factors,
                observations = excluded.observations,
                successes = excluded.successes,
                failures = excluded.failures,
                updated_at = excluded.updated_at,
                history = excluded.history
            ",
                params![
                    reputation.id,
                    reputation.score,
                    factors,
                    reputation.observations,
                    reputation.successes,
                    reputation.failures,
                    reputation.updated_at.to_rfc3339(),
                    history,
                ],
            )
            .context("failed saving reputation")?;

        Ok(())
    }

    pub fn load(&self, id: &str) -> Result<Option<Reputation>> {
        let mut stmt = self.conn.prepare(
            "
                SELECT
                    id,
                    score,
                    factors,
                    observations,
                    successes,
                    failures,
                    updated_at,
                    history

                FROM reputations

                WHERE id = ?1
                ",
        )?;

        let result = stmt.query_row(params![id], |row| {
            let factors_json: String = row.get(2)?;

            let history_json: String = row.get(7)?;

            Ok(Reputation {
                id: row.get(0)?,

                score: row.get(1)?,

                factors: serde_json::from_str(&factors_json).unwrap_or_default(),

                observations: row.get(3)?,

                successes: row.get(4)?,

                failures: row.get(5)?,

                updated_at: chrono::DateTime::parse_from_rfc3339(&row.get::<_, String>(6)?)
                    .map_err(|e| anyhow::anyhow!("Invalid datetime format: {}", e))?
                    .with_timezone(&chrono::Utc),

                history: serde_json::from_str(&history_json).unwrap_or_default(),
            })
        });

        match result {
            Ok(rep) => Ok(Some(rep)),

            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),

            Err(e) => Err(e.into()),
        }
    }

    pub fn delete(&self, id: &str) -> Result<()> {
        self.conn.execute(
            "
            DELETE FROM reputations
            WHERE id = ?1
            ",
            params![id],
        )?;

        Ok(())
    }
}
