use crate::agent::Agent;
use crate::db::service::UserService;
use log::warn;
use milky_rust_sdk::MilkyClient;
use milky_rust_sdk::prelude::{Event, EventKind};
use std::sync::Arc;

use super::message_handler::MessageHandler;

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

    pub async fn handle_event(&self, event: Event) {
        match event.kind {
            EventKind::MessageReceive { message } => {
                self.message_handler.handle(message).await;
            }
            _ => {
                warn!("未处理的事件类型");
            }
        }
    }
}
