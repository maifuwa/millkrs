mod agent;
mod bot;
mod config;
mod db;
mod utils;

use agent::Agent;
use anyhow::Result;
use bot::Bot;
use config::Config;
use db::service::UserService;
use log::{LevelFilter, debug, info};
use milky_rust_sdk::logger;
use milky_rust_sdk::prelude::Event;
use milky_rust_sdk::{Communication, MilkyClient, WebSocketConfig};
use std::sync::Arc;
use tokio::sync::mpsc;

#[tokio::main]
async fn main() -> Result<()> {
    logger::init_logger(Some(LevelFilter::Debug));

    let config = Config::init()?;

    let pool = db::init_db(&config.database.url, config.database.max_connections).await?;
    let user_service = UserService::new(pool);
    debug!("数据库初始化成功");

    let (event_tx, event_rx) = mpsc::channel::<Event>(config.bot.event_channel_capacity);
    let ws_config = WebSocketConfig::new(
        config.bot.endpoint.clone(),
        Option::from(config.bot.access_token.clone()),
    );
    let client = Arc::new(MilkyClient::new(
        Communication::WebSocket(ws_config),
        event_tx,
    )?);
    debug!("MilkyClient 初始化成功");

    let agent = Arc::new(Agent::new(&config.llm, Arc::clone(&client))?);
    debug!("Agent 初始化成功");

    let bot = Bot::new(&config.bot, user_service, client, event_rx, agent).await?;
    let bot_handle = bot.run().await?;

    tokio::signal::ctrl_c().await?;
    info!("收到 Ctrl+C 信号，开始关闭...");

    bot_handle.shutdown().await;

    Ok(())
}
