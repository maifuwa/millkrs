use anyhow::{Result, anyhow};
use log::debug;
use sqlx::SqlitePool;

use super::model::{
    CreateCustomPromptRequest, CreateMasterRequest, CreateUserRequest, UpdateUserRequest, User,
    UserRelation,
};

#[derive(Clone)]
pub struct UserService {
    pool: SqlitePool,
}

impl UserService {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    pub async fn create_user(&self, req: CreateUserRequest) -> Result<User> {
        debug!("创建用户请求: id={}, name={}", req.id, req.name);

        if let Some(existing_user) = self.get_user(req.id).await? {
            debug!("用户已存在: id={}", req.id);
            return Ok(existing_user);
        }

        sqlx::query(
            r#"
            INSERT INTO users (id, name, relation)
            VALUES (?, ?, 'friend')
            "#,
        )
        .bind(req.id)
        .bind(&req.name)
        .execute(&self.pool)
        .await?;

        debug!("用户创建成功: id={}", req.id);

        self.get_user(req.id)
            .await?
            .ok_or_else(|| anyhow!("创建用户后无法查询到用户"))
    }

    pub async fn create_master(&self, req: CreateMasterRequest) -> Result<User> {
        debug!("创建 master 用户请求: id={}, name={}", req.id, req.name);

        if self.has_master().await? {
            debug!("已存在 master 用户，无法创建新的 master");
            return Err(anyhow!("已存在 master 用户，无法创建新的 master"));
        }

        sqlx::query("UPDATE users SET relation = 'master' WHERE id = ?")
            .bind(req.id)
            .execute(&self.pool)
            .await?;

        debug!("用户提升为 master 成功: id={}", req.id);

        self.get_user(req.id)
            .await?
            .ok_or_else(|| anyhow!("更新用户为 master 后无法查询到用户"))
    }

    async fn get_user(&self, user_id: i64) -> Result<Option<User>> {
        debug!("查询用户: id={}", user_id);

        let row = sqlx::query_as::<_, (i64, String, String, Option<String>, String, String)>(
            r#"
            SELECT id, name, relation, custom_prompt, created_at, updated_at
            FROM users
            WHERE id = ?
            "#,
        )
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await?;

        match row {
            Some((id, name, relation_str, custom_prompt, created_at, updated_at)) => {
                let relation = UserRelation::from_str(&relation_str)?;
                debug!(
                    "用户查询成功: id={}, name={}, relation={:?}",
                    id, name, relation
                );
                Ok(Some(User {
                    id,
                    name,
                    relation,
                    custom_prompt,
                    created_at,
                    updated_at,
                }))
            }
            None => {
                debug!("用户不存在: id={}", user_id);
                Ok(None)
            }
        }
    }

    pub async fn update_user(&self, req: UpdateUserRequest) -> Result<User> {
        debug!(
            "更新用户请求: operator_id={}, user_id={}, relation={:?}",
            req.operator_id, req.user_id, req.relation
        );

        if !self.is_master(req.operator_id).await? {
            debug!("操作者不是 master，无权限修改用户关系");
            return Err(anyhow!("只有 master 用户才能修改用户关系"));
        }

        sqlx::query("UPDATE users SET relation = ? WHERE id = ?")
            .bind(req.relation.as_str())
            .bind(req.user_id)
            .execute(&self.pool)
            .await?;

        debug!("用户更新成功: user_id={}", req.user_id);

        self.get_user(req.user_id)
            .await?
            .ok_or_else(|| anyhow!("更新用户后无法查询到用户"))
    }

    pub async fn create_custom_prompt(&self, req: CreateCustomPromptRequest) -> Result<User> {
        debug!(
            "创建/更新自定义提示词: id={}, prompt_len={}",
            req.id,
            req.custom_prompt.len()
        );

        let rows_affected = sqlx::query("UPDATE users SET custom_prompt = ? WHERE id = ?")
            .bind(&req.custom_prompt)
            .bind(req.id)
            .execute(&self.pool)
            .await?
            .rows_affected();

        if rows_affected == 0 {
            return Err(anyhow!("用户 ID {} 不存在", req.id));
        }

        debug!("自定义提示词更新成功: id={}", req.id);

        self.get_user(req.id)
            .await?
            .ok_or_else(|| anyhow!("更新自定义提示词后无法查询到用户"))
    }

    async fn is_master(&self, user_id: i64) -> Result<bool> {
        debug!("检查是否为 master: id={}", user_id);

        let user = self.get_user(user_id).await?;
        let is_master = user
            .map(|u| u.relation == UserRelation::Master)
            .unwrap_or(false);

        debug!("master 检查结果: id={}, is_master={}", user_id, is_master);
        Ok(is_master)
    }

    async fn has_master(&self) -> Result<bool> {
        debug!("检查是否存在 master 用户");

        let result = sqlx::query_scalar::<_, bool>(
            r#"
        SELECT EXISTS(
            SELECT 1 FROM users WHERE relation = 'master'
        )
        "#,
        )
        .fetch_one(&self.pool)
        .await?;

        debug!("master 存在检查结果: {}", result);
        Ok(result)
    }
}
