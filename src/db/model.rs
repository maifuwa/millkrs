use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum UserRelation {
    Master,
    Friend,
    Stranger,
}

impl UserRelation {
    pub fn as_str(&self) -> &str {
        match self {
            UserRelation::Master => "master",
            UserRelation::Friend => "friend",
            UserRelation::Stranger => "stranger",
        }
    }

    pub fn from_str(s: &str) -> Result<Self> {
        match s {
            "master" => Ok(UserRelation::Master),
            "friend" => Ok(UserRelation::Friend),
            "stranger" => Ok(UserRelation::Stranger),
            _ => Err(anyhow!("无效的用户关系类型: {}", s)),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: i64,
    pub name: String,
    pub relation: UserRelation,
    pub custom_prompt: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone)]
pub struct CreateUserRequest {
    pub id: i64,
    pub name: String,
}

#[derive(Debug, Clone)]
pub struct CreateMasterRequest {
    pub id: i64,
    pub name: String,
}

#[derive(Debug, Clone)]
pub struct CreateCustomPromptRequest {
    pub id: i64,
    pub custom_prompt: String,
}

#[derive(Debug, Clone)]
pub struct UpdateUserRequest {
    pub operator_id: i64,
    pub user_id: i64,
    pub relation: UserRelation,
}
