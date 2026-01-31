mod bot;
mod config;
mod db;

use anyhow::Result;
use bot::Bot;
use config::Config;
use db::service::UserService;
use log::{LevelFilter, info, debug};
use milky_rust_sdk::logger;

#[tokio::main]
async fn main() -> Result<()> {
    logger::init_logger(Some(LevelFilter::Info));

    let config = Config::init()?;

    let pool = db::init_db(&config.database.url).await?;
    let user_service = UserService::new(pool);
    debug!("数据库初始化成功");

    let bot = Bot::new(&config, user_service).await?;
    let bot_handle = bot.run().await?;

    tokio::signal::ctrl_c().await?;
    info!("收到 Ctrl+C 信号，开始关闭...");

    bot_handle.shutdown().await;

    Ok(())
}
