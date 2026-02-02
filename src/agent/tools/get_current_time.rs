use chrono::Local;
use rig::completion::ToolDefinition;
use rig::tool::Tool;
use serde::{Deserialize, Serialize};
use serde_json::json;
use tracing::debug;

#[derive(Deserialize)]
pub struct GetCurrentTimeArgs {}

#[derive(Serialize)]
pub struct TimeResult {
    current_time: String,
    timestamp: i64,
}

#[derive(Debug, thiserror::Error)]
#[error("Time error")]
pub struct TimeError;

pub struct GetCurrentTime;

impl Tool for GetCurrentTime {
    const NAME: &'static str = "get_current_time";
    type Error = TimeError;
    type Args = GetCurrentTimeArgs;
    type Output = TimeResult;

    async fn definition(&self, _prompt: String) -> ToolDefinition {
        ToolDefinition {
            name: Self::NAME.to_string(),
            description: "获取当前时间，返回格式化的时间字符串和时间戳。当用户询问现在几点、今天日期、当前时间等问题时使用此工具。".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {}
            }),
        }
    }

    async fn call(&self, _args: Self::Args) -> Result<Self::Output, Self::Error> {
        debug!("[Tool] get_current_time called");
        let now = Local::now();
        let result = TimeResult {
            current_time: now.format("%Y-%m-%d %H:%M:%S").to_string(),
            timestamp: now.timestamp(),
        };
        debug!("[Tool] get_current_time completed: {}", result.current_time);
        Ok(result)
    }
}
