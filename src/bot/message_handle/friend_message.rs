use crate::agent::Agent;
use crate::db::model::CreateUserRequest;
use crate::db::service::UserService;
use anyhow::Result;
use log::error;
use milky_rust_sdk::MilkyClient;
use milky_rust_sdk::prelude::FriendMessage;
use milky_rust_sdk::utils::get_plain_text_from_segments;
use std::sync::Arc;

use friend_chat::FriendChatHandler;
use friend_command::FriendCommandHandler;

mod friend_chat;
mod friend_command;

#[derive(Clone)]
pub struct FriendMessageHandler {
    user_service: UserService,
    command_handler: FriendCommandHandler,
    chat_handler: FriendChatHandler,
}

impl FriendMessageHandler {
    pub fn new(user_service: UserService, client: Arc<MilkyClient>, agent: Arc<Agent>) -> Self {
        Self {
            user_service: user_service.clone(),
            command_handler: FriendCommandHandler::new(user_service, Arc::clone(&client)),
            chat_handler: FriendChatHandler::new(agent),
        }
    }

    pub async fn handle(&self, msg: FriendMessage) -> Result<()> {
        let id = msg.friend.user_id;
        let name = msg.friend.nickname;

        let user = match self
            .user_service
            .create_user(CreateUserRequest { id, name })
            .await
        {
            Ok(user) => user,
            Err(e) => {
                error!("创建用户失败: {e}");
                return Err(e);
            }
        };

        let text_content = get_plain_text_from_segments(&msg.message.segments);

        if text_content.starts_with('/') {
            self.command_handler.handle(user.id, &text_content).await?;
        } else {
            self.chat_handler.handle(&user, &text_content).await?;
        }

        Ok(())
    }
}
