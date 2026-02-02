use anyhow::{Result, anyhow};
use sqlx::SqlitePool;
use tracing::debug;

use super::scheduler_model::{CreateTaskRequest, ScheduledTask, TaskCreator, TaskFrequency};

type TaskRow = (
    i64,
    String,
    String,
    i64,
    String,
    String,
    bool,
    Option<String>,
    Option<String>,
    String,
    String,
);

#[derive(Clone)]
pub struct SchedulerService {
    pool: SqlitePool,
}

impl SchedulerService {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    pub async fn create_task(&self, req: CreateTaskRequest) -> Result<ScheduledTask> {
        debug!(
            "创建定时任务: user_id={}, cron={}, created_by={:?}",
            req.target_user_id, req.cron_expr, req.created_by
        );

        let result = sqlx::query(
            r#"
            INSERT INTO scheduled_tasks (frequency, cron_expr, target_user_id, content, created_by)
            VALUES (?, ?, ?, ?, ?)
            "#,
        )
        .bind(req.frequency.as_str())
        .bind(&req.cron_expr)
        .bind(req.target_user_id)
        .bind(&req.content)
        .bind(req.created_by.as_str())
        .execute(&self.pool)
        .await?;

        let task_id = result.last_insert_rowid();
        debug!("定时任务创建成功: id={}", task_id);

        self.get_task(task_id)
            .await?
            .ok_or_else(|| anyhow!("创建任务后无法查询到任务"))
    }

    pub async fn get_task(&self, task_id: i64) -> Result<Option<ScheduledTask>> {
        debug!("查询定时任务: id={}", task_id);

        let row = sqlx::query_as::<_, TaskRow>(
            r#"
            SELECT id, frequency, cron_expr, target_user_id, content,
                   created_by, enabled, last_run_at, next_run_at, created_at, updated_at
            FROM scheduled_tasks
            WHERE id = ?
            "#,
        )
        .bind(task_id)
        .fetch_optional(&self.pool)
        .await?;

        self.map_row_to_task(row)
    }

    pub async fn get_enabled_tasks(&self) -> Result<Vec<ScheduledTask>> {
        debug!("查询所有启用的定时任务");

        let rows = sqlx::query_as::<_, TaskRow>(
            r#"
            SELECT id, frequency, cron_expr, target_user_id, content,
                   created_by, enabled, last_run_at, next_run_at, created_at, updated_at
            FROM scheduled_tasks
            WHERE enabled = 1
            "#,
        )
        .fetch_all(&self.pool)
        .await?;

        let mut tasks = Vec::new();
        for row in rows {
            if let Some(task) = self.map_row_to_task(Some(row))? {
                tasks.push(task);
            }
        }

        debug!("查询到 {} 个启用的定时任务", tasks.len());
        Ok(tasks)
    }

    pub async fn get_system_tasks_for_user(&self, user_id: i64) -> Result<Vec<ScheduledTask>> {
        debug!("查询用户的系统定时任务: user_id={}", user_id);

        let rows = sqlx::query_as::<_, TaskRow>(
            r#"
            SELECT id, frequency, cron_expr, target_user_id, content,
                   created_by, enabled, last_run_at, next_run_at, created_at, updated_at
            FROM scheduled_tasks
            WHERE target_user_id = ? AND created_by = 'system'
            "#,
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await?;

        let mut tasks = Vec::new();
        for row in rows {
            if let Some(task) = self.map_row_to_task(Some(row))? {
                tasks.push(task);
            }
        }

        debug!("查询到 {} 个系统定时任务", tasks.len());
        Ok(tasks)
    }

    pub async fn delete_system_tasks_for_user(&self, user_id: i64) -> Result<u64> {
        debug!("删除用户的系统定时任务: user_id={}", user_id);

        let result = sqlx::query(
            "DELETE FROM scheduled_tasks WHERE target_user_id = ? AND created_by = 'system'",
        )
        .bind(user_id)
        .execute(&self.pool)
        .await?;

        let deleted = result.rows_affected();
        debug!("删除了 {} 个系统定时任务", deleted);
        Ok(deleted)
    }

    pub async fn update_last_run(&self, task_id: i64) -> Result<()> {
        debug!("更新任务最后执行时间: id={}", task_id);

        sqlx::query(
            r#"
            UPDATE scheduled_tasks
            SET last_run_at = CURRENT_TIMESTAMP
            WHERE id = ?
            "#,
        )
        .bind(task_id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn disable_task(&self, task_id: i64) -> Result<()> {
        debug!("禁用定时任务: id={}", task_id);

        sqlx::query("UPDATE scheduled_tasks SET enabled = 0 WHERE id = ?")
            .bind(task_id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    fn map_row_to_task(&self, row: Option<TaskRow>) -> Result<Option<ScheduledTask>> {
        match row {
            Some((
                id,
                frequency_str,
                cron_expr,
                target_user_id,
                content,
                created_by_str,
                enabled,
                last_run_at,
                next_run_at,
                created_at,
                updated_at,
            )) => {
                let frequency = TaskFrequency::from_str(&frequency_str)?;
                let created_by = TaskCreator::from_str(&created_by_str)?;

                Ok(Some(ScheduledTask {
                    id,
                    frequency,
                    cron_expr,
                    target_user_id,
                    content,
                    created_by,
                    enabled,
                    last_run_at,
                    next_run_at,
                    created_at,
                    updated_at,
                }))
            }
            None => Ok(None),
        }
    }
}
