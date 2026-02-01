mod event_handle;
mod message_handle;

use crate::agent::Agent;
use crate::config::BotConfig;
use crate::db::service::UserService;
use anyhow::{Result, bail};
use event_handle::Handler;
use log::{debug, error, info};
use milky_rust_sdk::MilkyClient;
use milky_rust_sdk::prelude::Event;
use std::sync::Arc;
use tokio::sync::{Semaphore, mpsc, oneshot};
use tokio::task::{JoinHandle, JoinSet};

pub struct Bot {
    client: Arc<MilkyClient>,
    event_rx: mpsc::Receiver<Event>,
    handler: Handler,
    max_concurrent_tasks: usize,
}

impl Bot {
    pub async fn new(
        bot_config: &BotConfig,
        user_service: UserService,
        client: Arc<MilkyClient>,
        event_rx: mpsc::Receiver<Event>,
        agent: Arc<Agent>,
    ) -> Result<Self> {
        if let Err(e) = client.connect_events().await {
            bail!("未能连接到事件流: {e}");
        }

        info!("成功链接到Milky事件流");

        let handler = Handler::new(user_service, Arc::clone(&client), agent);

        Ok(Self {
            client,
            event_rx,
            handler,
            max_concurrent_tasks: bot_config.max_concurrent_tasks,
        })
    }

    pub async fn run(mut self) -> Result<BotHandle> {
        let (ready_tx, ready_rx) = oneshot::channel();
        let (shutdown_tx, mut shutdown_rx) = oneshot::channel();

        let handler = self.handler.clone();
        let semaphore = Arc::new(Semaphore::new(self.max_concurrent_tasks));

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

                                let permit = match semaphore.clone().try_acquire_owned() {
                                    Ok(permit) => permit,
                                    Err(_) => {
                                        error!("任务队列已满，等待空闲槽位");
                                        match semaphore.clone().acquire_owned().await {
                                            Ok(permit) => permit,
                                            Err(e) => {
                                                error!("获取任务槽位失败: {e}");
                                                continue;
                                            }
                                        }
                                    }
                                };

                                let handler = handler.clone();
                                join_set.spawn(async move {
                                    let _permit = permit;
                                    if let Err(e) = handler.handle_event(event).await {
                                        error!("事件处理失败: {e}");
                                    }
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
