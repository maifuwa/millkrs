pub mod scheduler_model;
pub mod scheduler_service;
pub mod user_model;
pub mod user_service;

use anyhow::Result;
use sqlx::{
    SqlitePool,
    sqlite::{SqliteConnectOptions, SqlitePoolOptions},
};
use std::str::FromStr;

pub async fn init_db(database_url: &str, max_connections: u32) -> Result<SqlitePool> {
    let options = SqliteConnectOptions::from_str(database_url)?.create_if_missing(true);

    let pool = SqlitePoolOptions::new()
        .max_connections(max_connections)
        .connect_with(options)
        .await?;

    sqlx::query("PRAGMA journal_mode = WAL")
        .execute(&pool)
        .await?;
    sqlx::query("PRAGMA synchronous = NORMAL")
        .execute(&pool)
        .await?;
    sqlx::query("PRAGMA cache_size = -64000")
        .execute(&pool)
        .await?;

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS users (
            id INTEGER PRIMARY KEY NOT NULL,
            name TEXT NOT NULL,
            relation TEXT NOT NULL DEFAULT 'guest' CHECK(relation IN ('master', 'guest', 'stranger')),
            custom_prompt TEXT,
            created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
            updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
        )
        "#,
    )
        .execute(&pool)
        .await?;

    sqlx::query(
        r#"
        CREATE TRIGGER IF NOT EXISTS update_users_timestamp
        AFTER UPDATE ON users
        FOR EACH ROW
        BEGIN
            UPDATE users SET updated_at = CURRENT_TIMESTAMP WHERE id = NEW.id;
        END
        "#,
    )
    .execute(&pool)
    .await?;

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS scheduled_tasks (
            id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
            frequency TEXT NOT NULL CHECK(frequency IN ('once', 'daily')),
            cron_expr TEXT NOT NULL,
            target_user_id INTEGER NOT NULL,
            content TEXT NOT NULL,
            created_by TEXT NOT NULL DEFAULT 'user' CHECK(created_by IN ('system', 'user')),
            enabled INTEGER NOT NULL DEFAULT 1,
            last_run_at DATETIME,
            next_run_at DATETIME,
            created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
            updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY (target_user_id) REFERENCES users(id)
        )
        "#,
    )
    .execute(&pool)
    .await?;

    sqlx::query(
        r#"
        CREATE TRIGGER IF NOT EXISTS update_scheduled_tasks_timestamp
        AFTER UPDATE ON scheduled_tasks
        FOR EACH ROW
        BEGIN
            UPDATE scheduled_tasks SET updated_at = CURRENT_TIMESTAMP WHERE id = NEW.id;
        END
        "#,
    )
    .execute(&pool)
    .await?;

    Ok(pool)
}
