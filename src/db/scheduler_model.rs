use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TaskFrequency {
    Once,
    Daily,
}

impl TaskFrequency {
    pub fn as_str(&self) -> &str {
        match self {
            TaskFrequency::Once => "once",
            TaskFrequency::Daily => "daily",
        }
    }

    pub fn from_str(s: &str) -> Result<Self> {
        match s {
            "once" => Ok(TaskFrequency::Once),
            "daily" => Ok(TaskFrequency::Daily),
            _ => Err(anyhow!("无效的任务频率: {}", s)),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TaskCreator {
    System,
    User,
}

impl TaskCreator {
    pub fn as_str(&self) -> &str {
        match self {
            TaskCreator::System => "system",
            TaskCreator::User => "user",
        }
    }

    pub fn from_str(s: &str) -> Result<Self> {
        match s {
            "system" => Ok(TaskCreator::System),
            "user" => Ok(TaskCreator::User),
            _ => Err(anyhow!("无效的创建者类型: {}", s)),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduledTask {
    pub id: i64,
    pub frequency: TaskFrequency,
    pub cron_expr: String,
    pub target_user_id: i64,
    pub content: String,
    pub created_by: TaskCreator,
    pub enabled: bool,
    pub last_run_at: Option<String>,
    pub next_run_at: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone)]
pub struct CreateTaskRequest {
    pub target_user_id: i64,
    pub frequency: TaskFrequency,
    pub cron_expr: String,
    pub content: String,
    pub created_by: TaskCreator,
}
