use anyhow::Result;
use log::info;
use milky_rust_sdk::MilkyClient;
use milky_rust_sdk::prelude::GroupMessage;
use std::sync::Arc;

#[derive(Clone)]
pub struct GroupMessageHandler {
    #[allow(dead_code)]
    client: Arc<MilkyClient>,
}

impl GroupMessageHandler {
    pub fn new(client: Arc<MilkyClient>) -> Self {
        Self { client }
    }

    pub async fn handle(&self, msg: GroupMessage) -> Result<()> {
        let user_id = msg.message.sender_id;
        let group_id = msg.group.group_id;
        info!("收到群消息，用户ID: {}, 群ID: {}", user_id, group_id);
        Ok(())
    }
}
