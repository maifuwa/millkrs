use anyhow::Result;
use log::info;
use milky_rust_sdk::MilkyClient;
use milky_rust_sdk::prelude::TempMessage;
use std::sync::Arc;

#[derive(Clone)]
pub struct TempMessageHandler {
    #[allow(dead_code)]
    client: Arc<MilkyClient>,
}

impl TempMessageHandler {
    pub fn new(client: Arc<MilkyClient>) -> Self {
        Self { client }
    }

    pub async fn handle(&self, msg: TempMessage) -> Result<()> {
        let user_id = msg.message.sender_id;
        info!("收到临时消息，用户ID: {}", user_id);
        Ok(())
    }
}
