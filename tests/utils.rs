use admin_panel::shared::{config, db};

#[cfg(feature = "test-postgres")]
use sqlx::postgres::PgPool;
use sqlx::sqlite::SqlitePool;
use std::path::PathBuf;

pub async fn prepare_sqlite_db(temp_db_path: &PathBuf, db_name: &str) -> db::DB {
    let uri = format!("sqlite://{}?mode=rwc", temp_db_path.join(db_name).display());

    let pool = SqlitePool::connect(&uri).await.expect("connection error");

    let yaml_jobs =
        std::fs::read_to_string(PathBuf::from("tests").join("fixtures").join("jobs.yaml"))
            .expect("Failed to read YAML file");

    let jobs: Vec<db::Job> = serde_yaml::from_str(&yaml_jobs).expect("Failed to parse YAML");
    // Create the table
    sqlx::query(
        r"
        CREATE TABLE IF NOT EXISTS sqlt_loco_queue (
            id TEXT NOT NULL,
            name TEXT NOT NULL,
            task_data JSON NOT NULL,
            status TEXT NOT NULL DEFAULT 'queued',
            run_at TIMESTAMP NOT NULL,
            interval INTEGER,
            created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
            updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
        );
        ",
    )
    .execute(&pool)
    .await
    .expect("create table");

    for job in jobs {
        sqlx::query(
            r"
            INSERT INTO sqlt_loco_queue (id, name, task_data, status, run_at, interval, created_at, updated_at)
            VALUES (?, ?, ?, ?, ?, NULL, ?, ?)
            "
        )
        .bind(job.id)
        .bind(job.name)
        .bind(job.task_data.to_string())
        .bind(job.status)
        .bind(job.run_at)
        .bind(job.created_at)
        .bind(job.updated_at)
        .execute(&pool)
        .await.expect("create row");
    }

    let config = config::DBConfig::Sqlite(config::Sqlite { uri });

    db::init(config).await.expect("connect to sqlite")
}

#[cfg(feature = "test-postgres")]
pub async fn prepare_postgres_db() -> db::DB {
    let uri = std::env::var("TEST_POSTGRES_DB_CONNECTION")
        .expect("`TEST_POSTGRES_DB_CONNECTION` connection string");

    let pool = PgPool::connect(&uri).await.expect("connection error");

    let yaml_jobs =
        std::fs::read_to_string(PathBuf::from("tests").join("fixtures").join("jobs.yaml"))
            .expect("Failed to read YAML file");

    let jobs: Vec<db::Job> = serde_yaml::from_str(&yaml_jobs).expect("Failed to parse YAML");

    sqlx::query(
        r"
        DROP TABLE IF EXISTS pg_loco_queue;
      ",
    )
    .execute(&pool)
    .await
    .expect("drop table");
    sqlx::query(
        r"
        CREATE TABLE IF NOT EXISTS pg_loco_queue (
            id VARCHAR NOT NULL,
            name VARCHAR NOT NULL,
            task_data JSONB NOT NULL,
            status VARCHAR NOT NULL DEFAULT 'queued',
            run_at TIMESTAMPTZ NOT NULL,
            interval BIGINT,
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
        );
        ",
    )
    .execute(&pool)
    .await
    .expect("create table");

    for job in jobs {
        sqlx::query(
            r"
            INSERT INTO pg_loco_queue (id, name, task_data, status, run_at, interval, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, NULL, $6, $7)
            ",
        )
        .bind(job.id)
        .bind(job.name)
        .bind(job.task_data)
        .bind(job.status)
        .bind(job.run_at)
        .bind(job.created_at)
        .bind(job.updated_at)
        .execute(&pool)
        .await
        .expect("create row");
    }

    let config = config::DBConfig::Postgres(config::Postgres {
        uri,
        connect_timeout: 500,
        idle_timeout: 500,
        min_connections: 2,
        max_connections: 2,
    });

    db::init(config).await.expect("connect to postgres")
}
