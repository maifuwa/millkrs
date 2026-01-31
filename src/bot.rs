mod handler;
mod message_handler;

use crate::agent::Agent;
use crate::config::Config;
use crate::db::service::UserService;
use anyhow::{Result, bail};
use handler::Handler;
use log::{debug, error, info};
use milky_rust_sdk::prelude::Event;
use milky_rust_sdk::{Communication, MilkyClient, WebSocketConfig};
use std::sync::Arc;
use tokio::sync::{mpsc, oneshot};
use tokio::task::{JoinHandle, JoinSet};

pub struct Bot {
    client: Arc<MilkyClient>,
    event_rx: mpsc::Receiver<Event>,
    handler: Handler,
}

impl Bot {
    pub async fn new(config: &Config, user_service: UserService) -> Result<Self> {
        let (event_tx, event_rx) = mpsc::channel::<Event>(100);
        let ws_config = WebSocketConfig::new(
            config.bot.endpoint.clone(),
            Option::from(config.bot.access_token.clone()),
        );
        let client = MilkyClient::new(Communication::WebSocket(ws_config), event_tx)?;
        let client = Arc::new(client);

        if let Err(e) = client.connect_events().await {
            bail!("未能连接到事件流: {e}");
        }

        info!("成功链接到Milky事件流");

        let agent = Arc::new(Agent::new(&config.llm)?);
        let handler = Handler::new(user_service, Arc::clone(&client), agent);

        Ok(Self {
            client,
            event_rx,
            handler,
        })
    }

    pub async fn run(mut self) -> Result<BotHandle> {
        let (ready_tx, ready_rx) = oneshot::channel();
        let (shutdown_tx, mut shutdown_rx) = oneshot::channel();

        let handler = self.handler.clone();
        let event_task = tokio::spawn(async move {
            let _ = ready_tx.send(());

            let mut join_set = JoinSet::new();
            loop {
                tokio::select! {
                    _ = &mut shutdown_rx => {
                        info!("事件监听器收到关闭信号");
                        break;
                    }

                    event = self.event_rx.recv() => {
                        match event {
                            Some(event) => {
                                debug!("收到事件： {event:?}");
                                let handler = handler.clone();
                                join_set.spawn(async move {
                                    handler.handle_event(event).await;
                                });
                            }
                            None => {
                                info!("事件通道已经关闭");
                                break;
                            }
                        }
                    }
                }
            }

            info!("等待事件处理完毕");
            while join_set.join_next().await.is_some() {}
        });

        if ready_rx.await.is_err() {
            bail!("事件监听器启动失败")
        }
        info!("事件监听器启动成功");

        Ok(BotHandle {
            client: self.client,
            shutdown_tx,
            event_task,
        })
    }
}

pub struct BotHandle {
    client: Arc<MilkyClient>,
    shutdown_tx: oneshot::Sender<()>,
    event_task: JoinHandle<()>,
}

impl BotHandle {
    pub async fn shutdown(self) {
        let _ = self.shutdown_tx.send(());
        self.client.shutdown().await;

        match self.event_task.await {
            Ok(_) => info!("事件处理任务已正常处理完毕"),
            Err(e) => error!("事件处理任务异常退出: {e}"),
        }
    }
}
