use std::time::Duration;

use crate::shared::config;
use sqlx::{
    postgres::{PgPool, PgPoolOptions},
    sqlite::SqlitePool,
    QueryBuilder, Result,
};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Default, Serialize)]
pub struct StatusSats {
    pub completed: i64,
    pub processing: i64,
    pub cancelled: i64,
    pub queued: i64,
    pub failed: i64,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Job {
    pub id: String,
    pub name: String,
    pub task_data: serde_json::Value,
    pub status: String,
    pub run_at: DateTime<Utc>,
    pub interval: Option<i64>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

pub struct Pg {
    pub pool: PgPool,
}

impl Pg {
    #[must_use]
    pub const fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

pub struct Sqlite {
    pub pool: SqlitePool,
}

impl Sqlite {
    #[must_use]
    pub const fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

pub enum DB {
    Postgres(Pg),
    Sqlite(Sqlite),
}

pub enum JobStatus {
    Queued,
    Completed,
    Processing,
    Failed,
    Cancelled,
}

impl std::fmt::Display for JobStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let status = match self {
            Self::Queued => "queued",
            Self::Completed => "completed",
            Self::Processing => "processing",
            Self::Failed => "failed",
            Self::Cancelled => "cancelled",
        };
        write!(f, "{status}")
    }
}

/// Initializes a database connection based on the provided worker configuration.
///
/// # Errors
/// When could not connect to the connector
pub async fn init(config: config::DBConfig) -> Result<DB> {
    let executor = match config {
        config::DBConfig::Postgres(cnf) => {
            let pool = PgPoolOptions::new()
                .acquire_timeout(Duration::from_millis(cnf.connect_timeout))
                .idle_timeout(Some(Duration::from_millis(cnf.idle_timeout)))
                .min_connections(cnf.min_connections)
                .max_connections(cnf.max_connections)
                .connect(&cnf.uri)
                .await?;
            DB::Postgres(Pg::new(pool))
        }
        config::DBConfig::Sqlite(cnf) => {
            let pool = SqlitePool::connect(&cnf.uri).await?;
            DB::Sqlite(Sqlite::new(pool))
        }
    };

    Ok(executor)
}

impl DB {
    /// Retrieves status statistics from the job queue.
    ///
    /// # Errors
    ///
    /// Returns in case of query error
    pub async fn status_stats(&self) -> sqlx::Result<StatusSats> {
        let status: Vec<(String, i64)> = match self {
            Self::Postgres(pg) => {
                sqlx::query_as::<_, (String, i64)>(
                    "SELECT status, COUNT(*) FROM pg_loco_queue GROUP BY status",
                )
                .fetch_all(&pg.pool)
                .await?
            }
            Self::Sqlite(sqlite) => {
                sqlx::query_as::<_, (String, i64)>(
                    "SELECT status, COUNT(*) FROM sqlt_loco_queue GROUP BY status",
                )
                .fetch_all(&sqlite.pool)
                .await?
            }
        };

        let counts: std::collections::HashMap<String, i64> = status.into_iter().collect();

        Ok(StatusSats {
            completed: *counts.get("completed").unwrap_or(&0),
            processing: *counts.get("processing").unwrap_or(&0),
            cancelled: *counts.get("cancelled").unwrap_or(&0),
            queued: *counts.get("queued").unwrap_or(&0),
            failed: *counts.get("failed").unwrap_or(&0),
        })
    }

    /// Retrieves job entries from the queue with optional filtering.
    ///
    /// # Errors
    ///
    /// Returns in case of query error
    pub async fn get_jobs(
        &self,
        search: Option<String>,
        status: Option<String>,
    ) -> sqlx::Result<Vec<Job>> {
        match self {
            Self::Postgres(pg) => {
                let mut query_builder =
                    QueryBuilder::<sqlx::Postgres>::new("SELECT * FROM pg_loco_queue WHERE 1=1 ");

                if let Some(status_filter) = status.filter(|s| !s.is_empty()) {
                    query_builder
                        .push(" AND status = ")
                        .push_bind(status_filter);
                }

                if let Some(search_filter) = search.filter(|s| !s.is_empty()) {
                    query_builder
                        .push(" AND name ILIKE ")
                        .push_bind(format!("%{search_filter}%"));
                }

                query_builder.push(" ORDER BY created_at DESC");

                query_builder
                    .build_query_as::<Job>()
                    .fetch_all(&pg.pool)
                    .await
            }
            Self::Sqlite(sqlite) => {
                let mut query_builder =
                    QueryBuilder::<sqlx::Sqlite>::new("SELECT * FROM sqlt_loco_queue WHERE 1=1 ");

                if let Some(status_filter) = status.filter(|s| !s.is_empty()) {
                    query_builder
                        .push(" AND status = ")
                        .push_bind(status_filter);
                }

                if let Some(search_filter) = search.filter(|s| !s.is_empty()) {
                    query_builder
                        .push(" AND name LIKE ")
                        .push_bind(format!("%{search_filter}%"));
                }

                query_builder.push(" ORDER BY created_at DESC");

                query_builder
                    .build_query_as::<Job>()
                    .fetch_all(&sqlite.pool)
                    .await
            }
        }
    }

    /// Retrieves a specific job by its ID from the job queue.
    ///
    /// # Errors
    ///
    /// Returns in case of query error
    pub async fn get_job(&self, job_id: &str) -> sqlx::Result<Option<Job>> {
        match self {
            Self::Postgres(pg) => {
                let row = sqlx::query_as::<_, Job>("SELECT * FROM pg_loco_queue WHERE id = $1")
                    .bind(job_id)
                    .fetch_optional(&pg.pool)
                    .await?;
                Ok(row)
            }
            Self::Sqlite(sqlite) => {
                let row = sqlx::query_as::<_, Job>("SELECT * FROM sqlt_loco_queue WHERE id = ?")
                    .bind(job_id)
                    .fetch_optional(&sqlite.pool)
                    .await?;
                Ok(row)
            }
        }
    }

    /// Deletes a specific job by its ID from the job queue.
    ///
    /// # Errors
    ///
    /// Returns in case of query error
    pub async fn delete_job(&self, job_id: &str) -> sqlx::Result<()> {
        match self {
            Self::Postgres(pg) => {
                sqlx::query("DELETE FROM pg_loco_queue WHERE id = $1")
                    .bind(job_id)
                    .execute(&pg.pool)
                    .await?;
            }
            Self::Sqlite(sqlite) => {
                sqlx::query("DELETE FROM sqlt_loco_queue WHERE id = ?")
                    .bind(job_id)
                    .execute(&sqlite.pool)
                    .await?;
            }
        };
        Ok(())
    }

    /// Updates the status of a specific job in the job queue.
    ///
    /// # Errors
    ///
    /// Returns in case of query error
    pub async fn update_status(&self, job_id: &str, status: &JobStatus) -> sqlx::Result<Job> {
        let status_string = status.to_string();

        match self {
            Self::Postgres(pg) => {
                sqlx::query_as::<_, Job>(
                    "UPDATE pg_loco_queue 
                 SET status = $1 
                 WHERE id = $2 
                 RETURNING *",
                )
                .bind(&status_string)
                .bind(job_id)
                .fetch_one(&pg.pool)
                .await
            }
            Self::Sqlite(sqlite) => {
                sqlx::query_as::<_, Job>(
                    "UPDATE sqlt_loco_queue 
                 SET status = ? 
                 WHERE id = ? 
                 RETURNING *",
                )
                .bind(&status_string)
                .bind(job_id)
                .fetch_one(&sqlite.pool)
                .await
            }
        }
    }
}
