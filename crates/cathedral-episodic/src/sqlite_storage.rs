use crate::types::{EpisodicEntry, VectorClock};
use anyhow::Result;
use sqlx::{Row, SqlitePool, sqlite::SqlitePoolOptions};

pub struct SqliteStorage {
    pool: SqlitePool,
}

impl SqliteStorage {
    pub async fn new(database_url: &str) -> Result<Self> {
        let pool = SqlitePoolOptions::new().max_connections(5).connect(database_url).await?;

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS episodic_entries (
                id TEXT PRIMARY KEY,
                user_input TEXT NOT NULL,
                assistant_output TEXT NOT NULL,
                timestamp INTEGER NOT NULL,
                version INTEGER NOT NULL,
                vector_clock TEXT NOT NULL,
                worker_id TEXT NOT NULL,
                confidence REAL NOT NULL,
                deleted INTEGER NOT NULL DEFAULT 0
            )
            "#,
        )
        .execute(&pool)
        .await?;

        Ok(Self { pool })
    }

    pub async fn upsert(&self, entry: &EpisodicEntry) -> Result<()> {
        let clock_json = serde_json::to_string(&entry.vector_clock)?;
        sqlx::query(
            r#"
            INSERT OR REPLACE INTO episodic_entries
            (id, user_input, assistant_output, timestamp, version, vector_clock, worker_id, confidence, deleted)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#
        )
        .bind(&entry.id)
        .bind(&entry.user_input)
        .bind(&entry.assistant_output)
        .bind(entry.timestamp)
        .bind(entry.version as i64)
        .bind(&clock_json)
        .bind(&entry.worker_id)
        .bind(entry.confidence)
        .bind(if entry.deleted { 1 } else { 0 })
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn get(&self, id: &str) -> Result<Option<EpisodicEntry>> {
        let row = sqlx::query(
            r#"
            SELECT id, user_input, assistant_output, timestamp, version, vector_clock, worker_id, confidence, deleted
            FROM episodic_entries WHERE id = ?
            "#
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        if let Some(row) = row {
            let clock_str: String = row.get("vector_clock");
            let clock: VectorClock = serde_json::from_str(&clock_str)?;
            Ok(Some(EpisodicEntry {
                id: row.get("id"),
                user_input: row.get("user_input"),
                assistant_output: row.get("assistant_output"),
                timestamp: row.get("timestamp"),
                version: row.get::<i64, _>("version") as u64,
                vector_clock: clock,
                worker_id: row.get("worker_id"),
                confidence: row.get("confidence"),
                deleted: row.get::<i64, _>("deleted") == 1,
            }))
        } else {
            Ok(None)
        }
    }

    pub async fn list_all(&self) -> Result<Vec<EpisodicEntry>> {
        let rows = sqlx::query(
            r#"
            SELECT id, user_input, assistant_output, timestamp, version, vector_clock, worker_id, confidence, deleted
            FROM episodic_entries
            ORDER BY timestamp DESC
            "#
        )
        .fetch_all(&self.pool)
        .await?;

        let mut entries = Vec::new();
        for row in rows {
            let clock_str: String = row.get("vector_clock");
            let clock: VectorClock = serde_json::from_str(&clock_str)?;
            entries.push(EpisodicEntry {
                id: row.get("id"),
                user_input: row.get("user_input"),
                assistant_output: row.get("assistant_output"),
                timestamp: row.get("timestamp"),
                version: row.get::<i64, _>("version") as u64,
                vector_clock: clock,
                worker_id: row.get("worker_id"),
                confidence: row.get("confidence"),
                deleted: row.get::<i64, _>("deleted") == 1,
            });
        }
        Ok(entries)
    }

    pub async fn delete(&self, id: &str) -> Result<()> {
        sqlx::query("DELETE FROM episodic_entries WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }
}
