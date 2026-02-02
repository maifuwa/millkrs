use crate::agent::Agent;
use crate::db::user_service::UserService;
use anyhow::Result;
use milky_rust_sdk::MilkyClient;
use milky_rust_sdk::prelude::{Event, EventKind};
use std::sync::Arc;
use tracing::warn;

use super::message_handle::MessageHandler;

#[derive(Clone)]
pub struct Handler {
    message_handler: MessageHandler,
}

impl Handler {
    pub fn new(user_service: UserService, client: Arc<MilkyClient>, agent: Arc<Agent>) -> Self {
        Self {
            message_handler: MessageHandler::new(user_service, client, agent),
        }
    }

    pub async fn handle_event(&self, event: Event) -> Result<()> {
        match event.kind {
            EventKind::MessageReceive { message } => {
                self.message_handler.handle(message).await?;
            }
            _ => {
                warn!("未处理的事件类型");
            }
        }
        Ok(())
    }
}
