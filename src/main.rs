mod agent;
mod bot;
mod config;
mod db;
mod logger;
mod scheduler;
mod utils;

use anyhow::Result;
use bot::Bot;
use config::Config;
use db::scheduler_service::SchedulerService;
use db::user_service::UserService;
use milky_rust_sdk::prelude::Event;
use milky_rust_sdk::{Communication, MilkyClient, WebSocketConfig};
use scheduler::Actuator;
use std::sync::Arc;
use tokio::sync::mpsc;
use tracing::{debug, info};

#[tokio::main]
async fn main() -> Result<()> {
    logger::init();

    let config = Config::init()?;

    let pool = db::init_db(&config.database.url, config.database.max_connections).await?;
    let user_service = UserService::new(pool.clone());
    let scheduler_service = SchedulerService::new(pool);
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

    let actuator = Actuator::new(user_service.clone(), config.bot.agent_task_channel_capacity);
    let (agent, scheduler_manager) = actuator
        .start(scheduler_service, &config.llm, Arc::clone(&client))
        .await?;
    debug!("Actuator 初始化成功");

    let bot = Bot::new(&config.bot, user_service, client, event_rx, agent).await?;
    let bot_handle = bot.run().await?;
    debug!("Bot 初始化成功");

    tokio::signal::ctrl_c().await?;
    info!("收到 Ctrl+C 信号，开始关闭...");

    scheduler_manager.shutdown().await;
    bot_handle.shutdown().await;

    Ok(())
}
