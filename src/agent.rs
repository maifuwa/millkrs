mod tools;

use crate::config::LLMConfig;
use crate::db::user_model::User;
use crate::scheduler::SchedulerManager;
use anyhow::Result;
use milky_rust_sdk::MilkyClient;
use rig::agent::AgentBuilder;
use rig::client::CompletionClient;
use rig::completion::Chat;
use rig::providers::openai;
use std::sync::Arc;
use tools::{CreateScheduledTask, GetCurrentTime, SendMessage};

pub struct AgentTask {
    pub target_user_id: i64,
    pub content: String,
}

pub struct Agent {
    agent: rig::agent::Agent<openai::CompletionModel>,
}

const MAX_TOOL_DEPTH: usize = 5;

impl Agent {
    pub fn new(
        config: &LLMConfig,
        client: Arc<MilkyClient>,
        scheduler_manager: Arc<SchedulerManager>,
    ) -> Result<Self> {
        let llm_client = openai::CompletionsClient::builder()
            .api_key(&config.token)
            .base_url(&config.base_url)
            .build()?;

        let model = llm_client.completion_model(&config.model_name);

        let system_prompt = config.system_prompt()?;

        let agent = AgentBuilder::new(model)
            .preamble(&system_prompt)
            .temperature(config.temperature)
            .default_max_depth(MAX_TOOL_DEPTH)
            .tool(GetCurrentTime)
            .tool(SendMessage::new(client))
            .tool(CreateScheduledTask::new(scheduler_manager))
            .build();

        Ok(Self { agent })
    }

    pub async fn deal(&self, user: &User, message: &str) -> Result<()> {
        let relation_str = user.relation.as_str();

        let mut prompt = format!(
            "User Info:\n- User ID: {}\n- Name: {}\n- Relation: {}\n",
            user.id, user.name, relation_str
        );

        if let Some(custom_prompt) = &user.custom_prompt {
            prompt.push_str(&format!("- Custom Prompt: {}\n", custom_prompt));
        }

        prompt.push_str(&format!("\nUser Message: {}\n\n", message));
        prompt.push_str("请使用 send_message 工具回复用户的消息");

        let _response: String = self.agent.chat(&prompt, vec![]).await?;
        Ok(())
    }
}
