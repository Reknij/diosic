use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;

#[derive(Debug, Serialize, Deserialize)]
pub struct UserToCreate {
    pub alias: String,
    pub username: String,
    pub password: String,
}

#[derive(Debug, Hash, Clone, FromRow, Serialize, Deserialize)]
pub struct UserInfo {
    pub id: i64,
    pub username: String,
    pub password: String,
    pub alias: String,
    pub is_admin: bool,
}