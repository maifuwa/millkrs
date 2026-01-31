use crate::db::model::CreateUserRequest;
use crate::db::service::UserService;
use anyhow::Result;
use log::{error, info};
use milky_rust_sdk::MilkyClient;
use milky_rust_sdk::prelude::{IncomingSegment, MessageEvent, OutgoingSegment, TextData};
use std::sync::Arc;

#[derive(Clone)]
pub struct MessageHandler {
    user_service: UserService,
    client: Arc<MilkyClient>,
}

impl MessageHandler {
    pub fn new(user_service: UserService, client: Arc<MilkyClient>) -> Self {
        Self {
            user_service,
            client,
        }
    }

    pub async fn handle(&self, message: MessageEvent) {
        match message {
            MessageEvent::Friend(msg) => match self.handle_friend_message(msg).await {
                Err(e) => {
                    error!("{e}")
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

        let segments: Vec<OutgoingSegment> = msg
            .message
            .segments
            .into_iter()
            .filter_map(|seg| match seg {
                IncomingSegment::Text { text } => Some(OutgoingSegment::Text(TextData { text })),
                _ => None,
            })
            .collect();

        if !segments.is_empty() {
            if let Err(e) = self.client.send_private_message(user.id, segments).await {
                error!("发送私聊消息失败: {}", e);
            } else {
                info!("成功复读好友消息");
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
