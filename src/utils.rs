use milky_rust_sdk::MilkyClient;
use milky_rust_sdk::prelude::{OutgoingSegment, TextData};
use std::sync::Arc;
use tracing::error;

pub async fn send_message(milky_client: Arc<MilkyClient>, sender: i64, messages: Vec<String>) {
    for msg in messages {
        let segments = vec![OutgoingSegment::Text(TextData { text: msg })];

        if let Err(e) = milky_client.send_private_message(sender, segments).await {
            error!("发送私聊消息失败: {}", e);
        }
    }
}
