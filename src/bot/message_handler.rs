use crate::agent::Agent;
use crate::db::model::CreateUserRequest;
use crate::db::service::UserService;
use crate::utils::send_message;
use anyhow::{Result, anyhow};
use log::{error, info};
use milky_rust_sdk::MilkyClient;
use milky_rust_sdk::prelude::MessageEvent;
use milky_rust_sdk::utils::get_plain_text_from_segments;
use std::sync::Arc;

#[derive(Clone)]
pub struct MessageHandler {
    user_service: UserService,
    client: Arc<MilkyClient>,
    agent: Arc<Agent>,
}

impl MessageHandler {
    pub fn new(user_service: UserService, client: Arc<MilkyClient>, agent: Arc<Agent>) -> Self {
        Self {
            user_service,
            client,
            agent,
        }
    }

    pub async fn handle(&self, message: MessageEvent) {
        let sender = message.base_message().sender_id;
        match message {
            MessageEvent::Friend(msg) => match self.handle_friend_message(msg).await {
                Err(e) => {
                    error!("{e}");
                    let _ = send_message(self.client.clone(), sender, vec![e.to_string()]).await;
                }
                Ok(_) => {}
            },
            MessageEvent::Group(msg) => {
                self.handle_group_message(msg).await;
            }
            MessageEvent::Temp(msg) => {
                self.handle_temp_message(msg).await;
            }
        }
    }

    async fn handle_friend_message(
        &self,
        msg: milky_rust_sdk::prelude::FriendMessage,
    ) -> Result<()> {
        let id = msg.friend.user_id;
        let name = msg.friend.nickname;

        let user = self
            .user_service
            .create_user(CreateUserRequest { id, name })
            .await?;

        let text_content = get_plain_text_from_segments(&msg.message.segments);

        match self.agent.chat(&user, &text_content).await {
            Ok(response) => send_message(self.client.clone(), user.id, vec![response]).await,
            Err(e) => {
                return Err(anyhow!("AI处理消息失败: {}", e));
            }
        }

        Ok(())
    }

    async fn handle_group_message(&self, msg: milky_rust_sdk::prelude::GroupMessage) {
        let user_id = msg.message.sender_id;
        let group_id = msg.group.group_id;
        info!("收到群消息，用户ID: {}, 群ID: {}", user_id, group_id);
    }

    async fn handle_temp_message(&self, msg: milky_rust_sdk::prelude::TempMessage) {
        let user_id = msg.message.sender_id;
        info!("收到临时消息，用户ID: {}", user_id);
    }
}
