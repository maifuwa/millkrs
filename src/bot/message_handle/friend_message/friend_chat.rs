use crate::agent::Agent;
use crate::db::model::User;
use crate::utils::send_message;
use anyhow::Result;
use milky_rust_sdk::MilkyClient;
use std::sync::Arc;

#[derive(Clone)]
pub struct FriendChatHandler {
    client: Arc<MilkyClient>,
    agent: Arc<Agent>,
}

impl FriendChatHandler {
    pub fn new(client: Arc<MilkyClient>, agent: Arc<Agent>) -> Self {
        Self { client, agent }
    }

    pub async fn handle(&self, user: &User, message: &str) -> Result<()> {
        let result = self.process_chat(user, message).await;

        if let Err(e) = result {
            let _ = send_message(
                self.client.clone(),
                user.id,
                vec![format!("AI处理消息失败: {}", e)],
            )
            .await;
            return Err(e);
        }

        Ok(())
    }

    async fn process_chat(&self, user: &User, message: &str) -> Result<()> {
        let response = self.agent.chat(user, message).await?;
        send_message(self.client.clone(), user.id, vec![response]).await;
        Ok(())
    }
}
