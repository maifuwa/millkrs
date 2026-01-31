use anyhow::{Result, anyhow};
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
        if let Some(existing_user) = self.get_user(&req.id).await? {
            return Ok(existing_user);
        }

        sqlx::query(
            r#"
            INSERT INTO users (id, name, relation)
            VALUES (?, ?, 'friend')
            "#,
        )
        .bind(&req.id)
        .bind(&req.name)
        .execute(&self.pool)
        .await?;

        self.get_user(&req.id)
            .await?
            .ok_or_else(|| anyhow!("创建用户后无法查询到用户"))
    }

    pub async fn create_master(&self, req: CreateMasterRequest) -> Result<User> {
        if self.has_master().await? {
            return Err(anyhow!("已存在 master 用户，无法创建新的 master"));
        }

        sqlx::query("UPDATE users SET relation = 'master' WHERE id = ?")
            .bind(&req.id)
            .execute(&self.pool)
            .await?;

        self.get_user(&req.id)
            .await?
            .ok_or_else(|| anyhow!("更新用户为 master 后无法查询到用户"))
    }

    pub async fn get_user(&self, user_id: &str) -> Result<Option<User>> {
        let row = sqlx::query_as::<_, (String, String, String, Option<String>, String, String)>(
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
                Ok(Some(User {
                    id,
                    name,
                    relation,
                    custom_prompt,
                    created_at,
                    updated_at,
                }))
            }
            None => Ok(None),
        }
    }

    pub async fn update_user(&self, req: UpdateUserRequest) -> Result<User> {
        if self.is_master(&req.operator_id).await? == false {
            return Err(anyhow!("只有 master 用户才能修改用户关系"));
        }

        sqlx::query("UPDATE users SET relation = ? WHERE id = ?")
            .bind(req.relation.as_str())
            .bind(&req.user_id)
            .execute(&self.pool)
            .await?;

        self.get_user(&req.user_id)
            .await?
            .ok_or_else(|| anyhow!("更新用户后无法查询到用户"))
    }

    pub async fn create_custom_prompt(&self, req: CreateCustomPromptRequest) -> Result<User> {
        sqlx::query(
            r#"
           INSERT INTO users (id ,custom_prompt)
           VALUES (?, ?)
           ON CONFLICT(id) DO UPDATE SET
               custom_prompt = ?
           "#,
        )
        .bind(&req.id)
        .bind(&req.custom_prompt)
        .bind(&req.custom_prompt)
        .execute(&self.pool)
        .await?;

        self.get_user(&req.id)
            .await?
            .ok_or_else(|| anyhow!("更新自定义提示词后无法查询到用户"))
    }

    pub async fn is_master(&self, user_id: &str) -> Result<bool> {
        let user = self.get_user(user_id).await?;
        Ok(user
            .map(|u| u.relation == UserRelation::Master)
            .unwrap_or(false))
    }

    pub async fn has_master(&self) -> Result<bool> {
        let result = sqlx::query_scalar::<_, bool>(
            r#"
        SELECT EXISTS(
            SELECT 1 FROM users WHERE relation = 'master'
        )
        "#,
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(result)
    }
}
