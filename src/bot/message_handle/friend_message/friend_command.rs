use crate::db::model::{
    CreateCustomPromptRequest, CreateMasterRequest, UpdateUserRequest, UserRelation,
};
use crate::db::service::UserService;
use crate::utils::send_message;
use anyhow::Result;
use milky_rust_sdk::MilkyClient;
use std::sync::Arc;

#[derive(Debug, Clone, PartialEq)]
pub enum Command {
    CreateMaster,
    CreateCustomPrompt(String),
    UpdateUser(i64, String),
    All,
    Unknown(String),
}

impl Command {
    pub fn parse(input: &str) -> Self {
        let parts: Vec<&str> = input.splitn(2, ' ').collect();
        let cmd = parts[0];
        let args = parts.get(1).copied();

        match cmd {
            "/create_master" => Command::CreateMaster,
            "/create_custom_prompt" => {
                if let Some(prompt) = args {
                    Command::CreateCustomPrompt(prompt.to_string())
                } else {
                    Command::Unknown(cmd.to_string())
                }
            }
            "/update_user" => {
                if let Some(args_str) = args {
                    let args_parts: Vec<&str> = args_str.splitn(2, ' ').collect();
                    if args_parts.len() == 2 {
                        if let Ok(user_id) = args_parts[0].parse::<i64>() {
                            Command::UpdateUser(user_id, args_parts[1].to_string())
                        } else {
                            Command::Unknown(cmd.to_string())
                        }
                    } else {
                        Command::Unknown(cmd.to_string())
                    }
                } else {
                    Command::Unknown(cmd.to_string())
                }
            }
            "/all" => Command::All,
            _ => Command::Unknown(cmd.to_string()),
        }
    }
}

#[derive(Clone)]
pub struct FriendCommandHandler {
    user_service: UserService,
    client: Arc<MilkyClient>,
}

impl FriendCommandHandler {
    pub fn new(user_service: UserService, client: Arc<MilkyClient>) -> Self {
        Self {
            user_service,
            client,
        }
    }

    pub async fn handle(&self, user_id: i64, command: &str) -> Result<()> {
        let result = self.process_command(user_id, command).await;

        if let Err(e) = result {
            let _ = send_message(
                self.client.clone(),
                user_id,
                vec![format!("命令执行失败: {}", e)],
            )
            .await;
            return Err(e);
        }

        Ok(())
    }

    async fn process_command(&self, user_id: i64, command: &str) -> Result<()> {
        let cmd = Command::parse(command);

        match cmd {
            Command::CreateMaster => self.cmd_create_master(user_id).await,
            Command::CreateCustomPrompt(prompt) => {
                self.cmd_create_custom_prompt(user_id, &prompt).await
            }
            Command::UpdateUser(target_user_id, relation) => {
                self.cmd_update_user(user_id, target_user_id, &relation)
                    .await
            }
            Command::All => self.cmd_all(user_id).await,
            Command::Unknown(cmd_str) => {
                if cmd_str.starts_with("/create_custom_prompt") {
                    send_message(
                        self.client.clone(),
                        user_id,
                        vec!["用法: /create_custom_prompt [prompt]".to_string()],
                    )
                    .await;
                } else if cmd_str.starts_with("/update_user") {
                    send_message(
                        self.client.clone(),
                        user_id,
                        vec!["用法: /update_user [user_id] [relation]".to_string()],
                    )
                    .await;
                } else {
                    send_message(
                        self.client.clone(),
                        user_id,
                        vec![format!("未知命令: {}，使用 /all 查看所有命令", cmd_str)],
                    )
                    .await;
                }
                Ok(())
            }
        }
    }

    async fn cmd_create_master(&self, user_id: i64) -> Result<()> {
        self.user_service
            .create_master(CreateMasterRequest {
                id: user_id,
                name: String::new(),
            })
            .await?;

        send_message(
            self.client.clone(),
            user_id,
            vec!["成功创建 master 用户".to_string()],
        )
        .await;

        Ok(())
    }

    async fn cmd_create_custom_prompt(&self, user_id: i64, prompt: &str) -> Result<()> {
        self.user_service
            .create_custom_prompt(CreateCustomPromptRequest {
                id: user_id,
                custom_prompt: prompt.to_string(),
            })
            .await?;

        send_message(
            self.client.clone(),
            user_id,
            vec!["自定义提示词设置成功".to_string()],
        )
        .await;

        Ok(())
    }

    async fn cmd_update_user(
        &self,
        operator_id: i64,
        target_user_id: i64,
        relation: &str,
    ) -> Result<()> {
        let user_relation = UserRelation::from_str(relation)?;

        self.user_service
            .update_user(UpdateUserRequest {
                operator_id,
                user_id: target_user_id,
                relation: user_relation,
            })
            .await?;

        send_message(
            self.client.clone(),
            operator_id,
            vec![format!(
                "成功更新用户 {} 的关系为 {}",
                target_user_id, relation
            )],
        )
        .await;

        Ok(())
    }

    async fn cmd_all(&self, user_id: i64) -> Result<()> {
        let message = [
            "可用命令列表:".to_string(),
            "1. /create_master - 创建 master 用户（仅限首次）".to_string(),
            "2. /create_custom_prompt [prompt] - 设置自定义提示词".to_string(),
            "3. /update_user [user_id] [relation] - 修改用户关系（仅 master）".to_string(),
            "4. /all - 查看所有命令".to_string(),
        ]
        .join("\n");

        send_message(self.client.clone(), user_id, vec![message]).await;
        Ok(())
    }
}
