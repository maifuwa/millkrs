use crate::agent::Agent;
use crate::db::model::User;
use anyhow::Result;
use milky_rust_sdk::MilkyClient;
use std::sync::Arc;

#[derive(Clone)]
pub struct FriendChatHandler {
    agent: Arc<Agent>,
}

impl FriendChatHandler {
    pub fn new(agent: Arc<Agent>) -> Self {
        Self { agent }
    }

    pub async fn handle(&self, user: &User, message: &str) -> Result<()> {
        self.agent.deal(user, message).await
    }
}
