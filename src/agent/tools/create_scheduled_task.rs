use crate::db::scheduler_model::{CreateTaskRequest, TaskCreator, TaskFrequency};
use crate::scheduler::SchedulerManager;
use rig::completion::ToolDefinition;
use rig::tool::Tool;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::sync::Arc;
use tracing::debug;

#[derive(Deserialize)]
pub struct CreateScheduledTaskArgs {
    pub user_id: i64,
    pub content: String,
    pub cron_expr: String,
    pub frequency: String,
}

#[derive(Serialize)]
pub struct CreateScheduledTaskResult {
    pub success: bool,
    pub task_id: i64,
    pub message: String,
}

#[derive(Debug, thiserror::Error)]
#[error("Create scheduled task error: {0}")]
pub struct CreateScheduledTaskError(String);

pub struct CreateScheduledTask {
    manager: Arc<SchedulerManager>,
}

impl CreateScheduledTask {
    pub fn new(manager: Arc<SchedulerManager>) -> Self {
        Self { manager }
    }
}

impl Tool for CreateScheduledTask {
    const NAME: &'static str = "create_scheduled_task";
    type Error = CreateScheduledTaskError;
    type Args = CreateScheduledTaskArgs;
    type Output = CreateScheduledTaskResult;

    async fn definition(&self, _prompt: String) -> ToolDefinition {
        ToolDefinition {
            name: Self::NAME.to_string(),
            description: "为用户创建定时任务/日程提醒。可以创建一次性任务或每日重复任务。创建成功后需要告知用户任务已创建。"
                .to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "user_id": {
                        "type": "integer",
                        "description": "要提醒的用户ID"
                    },
                    "content": {
                        "type": "string",
                        "description": "提醒内容的完整prompt，例如：'请用自然、友好的方式提醒用户该开会了'"
                    },
                    "cron_expr": {
                        "type": "string",
                        "description": "Cron表达式，格式为：秒 分 时 日 月 星期。例如：'0 0 8 * * *' 表示每天8点，'0 30 18 2 2 *' 表示2月2日18:30"
                    },
                    "frequency": {
                        "type": "string",
                        "enum": ["once", "daily"],
                        "description": "任务频率：once表示执行一次，daily表示每天执行"
                    }
                },
                "required": ["user_id", "content", "cron_expr", "frequency"]
            }),
        }
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        debug!(
            "[Tool] create_scheduled_task called: user_id={}, frequency={}, cron={}",
            args.user_id, args.frequency, args.cron_expr
        );

        let frequency = TaskFrequency::from_str(&args.frequency)
            .map_err(|e| CreateScheduledTaskError(e.to_string()))?;

        let req = CreateTaskRequest {
            target_user_id: args.user_id,
            frequency,
            cron_expr: args.cron_expr.clone(),
            content: args.content.clone(),
            created_by: TaskCreator::User,
        };

        let task = self
            .manager
            .add_task(req)
            .await
            .map_err(|e| CreateScheduledTaskError(e.to_string()))?;

        debug!(
            "[Tool] create_scheduled_task completed: task_id={}",
            task.id
        );

        Ok(CreateScheduledTaskResult {
            success: true,
            task_id: task.id,
            message: format!("已创建定时任务，频率：{:?}", frequency),
        })
    }
}
