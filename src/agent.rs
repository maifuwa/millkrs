mod tools;

use crate::config::LLMConfig;
use crate::db::model::{User, UserRelation};
use anyhow::Result;
use rig::agent::AgentBuilder;
use rig::client::CompletionClient;
use rig::completion::Chat;
use rig::providers::openai;
use tools::GetCurrentTime;

pub struct Agent {
    agent: rig::agent::Agent<openai::CompletionModel>,
}

impl Agent {
    pub fn new(config: &LLMConfig) -> Result<Self> {
        let client = openai::CompletionsClient::builder()
            .api_key(&config.token)
            .base_url(&config.base_url)
            .build()?;

        let model = client.completion_model(&config.model_name);

        let system_prompt = config.system_prompt()?;

        let agent = AgentBuilder::new(model)
            .preamble(&system_prompt)
            .temperature(config.temperature)
            .tool(GetCurrentTime)
            .build();

        Ok(Self { agent })
    }

    pub async fn chat(&self, user: &User, message: &str) -> Result<String> {
        let relation_str = match user.relation {
            UserRelation::Master => "master",
            UserRelation::Friend => "friend",
            UserRelation::Stranger => "stranger",
        };

        let mut prompt = format!(
            "User Info:\n- Name: {}\n- Relation: {}\n",
            user.name, relation_str
        );

        if let Some(custom_prompt) = &user.custom_prompt {
            prompt.push_str(&format!("- Custom Prompt: {}\n", custom_prompt));
        }

        prompt.push_str(&format!("\nUser Message: {}", message));

        let response: String = self.agent.chat(&prompt, vec![]).await?;
        Ok(response)
    }
}
