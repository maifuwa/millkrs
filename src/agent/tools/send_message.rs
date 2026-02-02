use milky_rust_sdk::MilkyClient;
use milky_rust_sdk::prelude::{OutgoingSegment, TextData};
use rig::completion::ToolDefinition;
use rig::tool::Tool;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::sync::Arc;
use tracing::debug;

#[derive(Deserialize)]
pub struct SendMessageArgs {
    pub user_id: i64,
    pub messages: Vec<String>,
}

#[derive(Serialize)]
pub struct SendMessageResult {
    pub success: bool,
    pub sent_count: usize,
}

#[derive(Debug, thiserror::Error)]
#[error("Send message error: {0}")]
pub struct SendMessageError(String);

pub struct SendMessage {
    client: Arc<MilkyClient>,
}

impl SendMessage {
    pub fn new(client: Arc<MilkyClient>) -> Self {
        Self { client }
    }
}

impl Tool for SendMessage {
    const NAME: &'static str = "send_message";
    type Error = SendMessageError;
    type Args = SendMessageArgs;
    type Output = SendMessageResult;

    async fn definition(&self, _prompt: String) -> ToolDefinition {
        ToolDefinition {
            name: Self::NAME.to_string(),
            description: "发送一条或多条消息给用户。messages参数是一个字符串数组，每个字符串会作为单独的一条消息发送。\n\n重要规则：\n1. 每次互动必须调用此工具与用户互动\n2. 每次互动只能调用一次此工具".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "user_id": {
                        "type": "integer",
                        "description": "接收消息的用户ID"
                    },
                    "messages": {
                        "type": "array",
                        "items": {
                            "type": "string"
                        },
                        "description": "要发送的消息列表，每条消息会单独发送"
                    }
                },
                "required": ["user_id", "messages"]
            }),
        }
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        debug!(
            "[Tool] send_message called: user_id={}, message_count={}",
            args.user_id,
            args.messages.len()
        );
        let mut sent_count = 0;

        for msg in args.messages {
            let segments = vec![OutgoingSegment::Text(TextData { text: msg })];

            match self
                .client
                .send_private_message(args.user_id, segments)
                .await
            {
                Ok(_) => sent_count += 1,
                Err(e) => {
                    debug!(
                        "[Tool] send_message failed at message {}: {}",
                        sent_count + 1,
                        e
                    );
                    return Err(SendMessageError(format!(
                        "发送第{}条消息失败: {}",
                        sent_count + 1,
                        e
                    )));
                }
            }
        }

        debug!("[Tool] send_message completed: sent_count={}", sent_count);
        Ok(SendMessageResult {
            success: true,
            sent_count,
        })
    }
}
