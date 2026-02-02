use crate::agent::AgentTask;
use crate::db::scheduler_model::{CreateTaskRequest, ScheduledTask, TaskCreator, TaskFrequency};
use crate::db::scheduler_service::SchedulerService;
use crate::db::user_model::UserRelation;
use crate::db::user_service::UserService;
use anyhow::Result;
use chrono::Local;
use rand::Rng;
use tokio::sync::{Mutex, mpsc};
use tokio_cron_scheduler::{Job, JobScheduler};
use tracing::{debug, error, info};

struct RandomTask {
    content: &'static str,
    hour_range: (u32, u32),
}

const RANDOM_TASKS: &[RandomTask] = &[
    RandomTask {
        content: "现在是早上，请给用户发送一条温馨的早安问候，可以包含今日天气、励志语句等。",
        hour_range: (7, 9),
    },
    RandomTask {
        content: "现在是中午，请给用户发送一条午间问候，提醒用户注意休息和用餐。",
        hour_range: (11, 13),
    },
    RandomTask {
        content: "现在是晚上，请给用户发送一条晚安问候，祝用户有个好梦。",
        hour_range: (21, 23),
    },
    RandomTask {
        content: "请用可爱、关心的语气提醒用户喝水，保持身体健康。",
        hour_range: (9, 18),
    },
    RandomTask {
        content: "请用关心的语气提醒用户起来活动一下，避免久坐对身体的伤害。",
        hour_range: (10, 17),
    },
];

pub struct SchedulerManager {
    scheduler: Mutex<JobScheduler>,
    service: SchedulerService,
    user_service: UserService,
    task_tx: mpsc::Sender<AgentTask>,
}

impl SchedulerManager {
    pub async fn new(
        service: SchedulerService,
        user_service: UserService,
        task_tx: mpsc::Sender<AgentTask>,
    ) -> Result<Self> {
        let scheduler = JobScheduler::new().await?;

        Ok(Self {
            scheduler: Mutex::new(scheduler),
            service,
            user_service,
            task_tx,
        })
    }

    pub async fn start(&self) -> Result<()> {
        self.initialize_random_tasks().await?;
        self.schedule_daily_random_task_update().await?;
        self.load_tasks().await?;
        self.scheduler.lock().await.start().await?;
        info!("定时任务调度器已启动");
        Ok(())
    }

    pub async fn shutdown(&self) {
        match self.scheduler.lock().await.shutdown().await {
            Ok(_) => info!("定时任务调度器已关闭"),
            Err(e) => error!("调度器关闭失败: {}", e),
        }
    }

    async fn load_tasks(&self) -> Result<()> {
        info!("加载已启用的定时任务");

        let tasks = self.service.get_enabled_tasks().await?;

        for task in tasks {
            if let Err(e) = self
                .schedule_task(
                    task.id,
                    &task.cron_expr,
                    task.frequency,
                    task.target_user_id,
                    task.content.clone(),
                )
                .await
            {
                error!("加载任务失败: id={}, error={}", task.id, e);
            }
        }

        Ok(())
    }

    async fn initialize_random_tasks(&self) -> Result<()> {
        info!("初始化随机定时任务");

        let users = self.user_service.get_all_users().await?;

        for user in users {
            if user.relation == UserRelation::Stranger {
                continue;
            }

            let existing_tasks = self.service.get_system_tasks_for_user(user.id).await?;

            let should_recreate = if existing_tasks.is_empty() {
                true
            } else {
                let now = Local::now();
                existing_tasks.iter().any(|task| {
                    if let Ok(created) =
                        chrono::NaiveDateTime::parse_from_str(&task.created_at, "%Y-%m-%d %H:%M:%S")
                    {
                        created.date() < now.date_naive()
                    } else {
                        true
                    }
                })
            };

            if should_recreate {
                info!("为用户 {} 重新创建随机定时任务", user.id);
                self.service.delete_system_tasks_for_user(user.id).await?;
                self.create_random_tasks_for_user(user.id).await?;
            }
        }

        Ok(())
    }

    async fn schedule_daily_random_task_update(&self) -> Result<()> {
        let service = self.service.clone();
        let user_service = self.user_service.clone();

        let job = Job::new_async("0 0 1 * * *", move |_uuid, _lock| {
            let service = service.clone();
            let user_service = user_service.clone();

            Box::pin(async move {
                info!("执行每日随机任务重新生成");

                match user_service.get_all_users().await {
                    Ok(users) => {
                        for user in users {
                            if user.relation == UserRelation::Stranger {
                                continue;
                            }

                            if let Err(e) = service.delete_system_tasks_for_user(user.id).await {
                                error!("删除用户系统任务失败: user_id={}, error={}", user.id, e);
                                continue;
                            }

                            if let Err(e) = create_random_tasks(&service, user.id).await {
                                error!("创建随机任务失败: user_id={}, error={}", user.id, e);
                            }
                        }
                    }
                    Err(e) => error!("获取用户列表失败: {}", e),
                }
            })
        })?;

        self.scheduler.lock().await.add(job).await?;
        info!("已添加每日凌晨1点随机任务更新调度");

        Ok(())
    }

    async fn create_random_tasks_for_user(&self, user_id: i64) -> Result<()> {
        for random_task in RANDOM_TASKS {
            let cron_expr = generate_random_cron(random_task.hour_range);

            let req = CreateTaskRequest {
                target_user_id: user_id,
                frequency: TaskFrequency::Once,
                cron_expr: cron_expr.clone(),
                content: random_task.content.to_string(),
                created_by: TaskCreator::System,
            };

            let task = self.service.create_task(req).await?;

            self.schedule_task(
                task.id,
                &task.cron_expr,
                task.frequency,
                user_id,
                task.content,
            )
            .await?;
        }

        Ok(())
    }

    pub async fn schedule_task(
        &self,
        task_id: i64,
        cron_expr: &str,
        frequency: TaskFrequency,
        target_user_id: i64,
        content: String,
    ) -> Result<()> {
        let service = self.service.clone();
        let task_tx = self.task_tx.clone();

        let job = Job::new_async(cron_expr, move |uuid, lock| {
            let service = service.clone();
            let task_tx = task_tx.clone();
            let content = content.clone();

            Box::pin(async move {
                debug!("定时任务触发: id={}", task_id);

                let agent_task = AgentTask {
                    target_user_id,
                    content,
                };

                if let Err(e) = task_tx.send(agent_task).await {
                    error!("发送任务到 Agent 失败: {}", e);
                }

                if let Err(e) = service.update_last_run(task_id).await {
                    error!("更新任务执行时间失败: {}", e);
                }

                if frequency == TaskFrequency::Once {
                    if let Err(e) = service.disable_task(task_id).await {
                        error!("禁用一次性任务失败: {}", e);
                    }

                    if let Err(e) = lock.remove(&uuid).await {
                        error!("从调度器移除一次性任务失败: {}", e);
                    } else {
                        debug!("已从调度器移除一次性任务: id={}", task_id);
                    }
                }
            })
        })?;

        self.scheduler.lock().await.add(job).await?;
        debug!("任务已添加到调度器: id={}, cron={}", task_id, cron_expr);

        Ok(())
    }

    pub async fn add_task(&self, req: CreateTaskRequest) -> Result<ScheduledTask> {
        let task = self.service.create_task(req).await?;

        self.schedule_task(
            task.id,
            &task.cron_expr,
            task.frequency,
            task.target_user_id,
            task.content.clone(),
        )
        .await?;

        Ok(task)
    }
}

async fn create_random_tasks(service: &SchedulerService, user_id: i64) -> Result<()> {
    for random_task in RANDOM_TASKS {
        let cron_expr = generate_random_cron(random_task.hour_range);

        let req = CreateTaskRequest {
            target_user_id: user_id,
            frequency: TaskFrequency::Once,
            cron_expr,
            content: random_task.content.to_string(),
            created_by: TaskCreator::System,
        };

        service.create_task(req).await?;
    }

    Ok(())
}

fn generate_random_cron(hour_range: (u32, u32)) -> String {
    let mut rng = rand::rng();
    let hour = rng.random_range(hour_range.0..=hour_range.1);
    let minute = rng.random_range(0..60);
    format!("0 {} {} * * *", minute, hour)
}
