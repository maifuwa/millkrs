mod config;
mod db;

use anyhow::Result;
use config::Config;
use db::service::UserService;
use log::{LevelFilter, info};
use milky_rust_sdk::logger;

#[tokio::main]
async fn main() -> Result<()> {
    logger::init_logger(Some(LevelFilter::Info));

    let config = Config::init()?;

    info!("正在初始化数据库: {}", config.database.url);
    let pool = db::init_db(&config.database.url).await?;
    info!("数据库初始化成功");

    // 创建用户服务实例
    let _user_service = UserService::new(pool);
    info!("用户服务初始化成功");

    Ok(())
}
