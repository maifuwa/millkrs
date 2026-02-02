use crate::agent::{Agent, AgentTask};
use crate::config::LLMConfig;
use crate::db::scheduler_service::SchedulerService;
use crate::db::user_service::UserService;
use crate::scheduler::SchedulerManager;
use anyhow::Result;
use milky_rust_sdk::MilkyClient;
use std::sync::Arc;
use tokio::sync::mpsc;
use tracing::{debug, error};

pub struct Actuator {
    user_service: UserService,
    channel_capacity: usize,
}

impl Actuator {
    pub fn new(user_service: UserService, channel_capacity: usize) -> Self {
        Self {
            user_service,
            channel_capacity,
        }
    }

    pub async fn start(
        self,
        scheduler_service: SchedulerService,
        llm_config: &LLMConfig,
        client: Arc<MilkyClient>,
    ) -> Result<(Arc<Agent>, Arc<SchedulerManager>)> {
        let (task_tx, task_rx) = mpsc::channel(self.channel_capacity);

        let scheduler_manager = Arc::new(
            SchedulerManager::new(scheduler_service, self.user_service.clone(), task_tx).await?,
        );

        scheduler_manager.start().await?;
        debug!("定时任务调度器已启动");

        let agent = Arc::new(Agent::new(
            llm_config,
            client,
            Arc::clone(&scheduler_manager),
        )?);
        debug!("Agent 初始化成功");

        let user_service = self.user_service;
        let agent_clone = Arc::clone(&agent);

        tokio::spawn(async move {
            Self::run(agent_clone, user_service, task_rx).await;
        });
        debug!("Actuator 任务循环已启动");

        Ok((agent, scheduler_manager))
    }

    async fn run(
        agent: Arc<Agent>,
        user_service: UserService,
        mut task_rx: mpsc::Receiver<AgentTask>,
    ) {
        while let Some(task) = task_rx.recv().await {
            debug!(
                "收到定时任务: target_user_id={}, content={}",
                task.target_user_id, task.content
            );

            match user_service.get_user(task.target_user_id).await {
                Ok(Some(user)) => {
                    if let Err(e) = agent.deal(&user, &task.content).await {
                        error!("执行定时任务失败: {}", e);
                    }
                }
                Ok(None) => {
                    error!("用户不存在: user_id={}", task.target_user_id);
                }
                Err(e) => {
                    error!("查询用户失败: user_id={}, error={}", task.target_user_id, e);
                }
            }
        }

        debug!("Actuator 任务循环已退出");
    }
}
