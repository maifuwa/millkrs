use log::error;
use milky_rust_sdk::MilkyClient;
use milky_rust_sdk::prelude::{OutgoingSegment, TextData};
use std::sync::Arc;

pub async fn send_message(milky_client: Arc<MilkyClient>, sender: i64, message: Vec<String>) {
    let message = message
        .iter()
        .map(|s| {
            OutgoingSegment::Text(TextData {
                text: s.to_string(),
            })
        })
        .collect();

    match milky_client.send_private_message(sender, message).await {
        Err(e) => error!("发送私聊消息失败: {}", e),
        Ok(_) => (),
    }
}
