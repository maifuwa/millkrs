use crate::agent::Agent;
use crate::db::service::UserService;
use anyhow::Result;
use milky_rust_sdk::MilkyClient;
use milky_rust_sdk::prelude::MessageEvent;
use std::sync::Arc;

use friend_message::FriendMessageHandler;
use group_message::GroupMessageHandler;
use temp_message::TempMessageHandler;

mod friend_message;
mod group_message;
mod temp_message;

#[derive(Clone)]
pub struct MessageHandler {
    friend_handler: FriendMessageHandler,
    group_handler: GroupMessageHandler,
    temp_handler: TempMessageHandler,
}

impl MessageHandler {
    pub fn new(user_service: UserService, client: Arc<MilkyClient>, agent: Arc<Agent>) -> Self {
        Self {
            friend_handler: FriendMessageHandler::new(user_service, Arc::clone(&client), agent),
            group_handler: GroupMessageHandler::new(Arc::clone(&client)),
            temp_handler: TempMessageHandler::new(client),
        }
    }

    pub async fn handle(&self, message: MessageEvent) -> Result<()> {
        match message {
            MessageEvent::Friend(msg) => {
                self.friend_handler.handle(msg).await?;
            }
            MessageEvent::Group(msg) => {
                self.group_handler.handle(msg).await?;
            }
            MessageEvent::Temp(msg) => {
                self.temp_handler.handle(msg).await?;
            }
        }
        Ok(())
    }
}
