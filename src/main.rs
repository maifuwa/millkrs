mod config;

use anyhow::Result;
use config::Config;
use log::{LevelFilter, info};
use milky_rust_sdk::logger;

#[tokio::main]
async fn main() -> Result<()> {
    logger::init_logger(Some(LevelFilter::Info));
    // 初始化配置
    let config = Config::init()?;

    info!("配置加载成功！");
    info!("Bot 端点: {}", config.bot.endpoint);
    info!("数据库: {}", config.database.url);

    // TODO: 启动 QQ Bot 和其他服务

    Ok(())
}
