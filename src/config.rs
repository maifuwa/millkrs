use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub bot: BotConfig,
    pub rig: RigConfig,
    pub search: SearchConfig,
    pub database: DatabaseConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BotConfig {
    pub endpoint: String,
    pub access_token: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RigConfig {
    pub token: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchConfig {
    pub serpapi_key: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    pub url: String,
}

impl Config {
    fn default() -> Self {
        Self {
            bot: BotConfig {
                endpoint: "wss://your-bot-endpoint".to_string(),
                access_token: "your-access-token".to_string(),
            },
            rig: RigConfig {
                token: "your-deepseek-api-token".to_string(),
            },
            search: SearchConfig {
                serpapi_key: "your-serpapi-key".to_string(),
            },
            database: DatabaseConfig {
                url: "sqlite://data.db".to_string(),
            },
        }
    }

    pub fn init() -> Result<Self> {
        let config_path = "config.toml";

        if !Path::new(config_path).exists() {
            let default_config = Config::default();

            let toml_string =
                toml::to_string_pretty(&default_config).context("无法序列化默认配置")?;

            fs::write(config_path, toml_string).context("无法创建 config.toml 文件")?;
            anyhow::bail!("请先配置 config.toml 文件后再运行程序");
        }

        let config = config::Config::builder()
            .add_source(config::File::with_name(config_path))
            .build()
            .context("无法读取配置文件")?;

        let config: Config = config.try_deserialize().context("配置文件格式错误")?;

        Ok(config)
    }
}
