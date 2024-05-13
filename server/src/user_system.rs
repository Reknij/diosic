use crate::myutil;
use anyhow::Result;
use sqlx::{Pool, QueryBuilder, Row, Sqlite};
use std::{collections::HashMap, sync::Arc};
use tokio::sync::RwLock;

use self::model::{UserInfo, UserToCreate};

pub mod model;

#[derive(Debug, Clone)]
pub struct UserSystem {
    db: Pool<Sqlite>,
    user_tokens: Arc<RwLock<HashMap<String, UserInfo>>>,
}

impl UserSystem {
    pub async fn new(db: Pool<Sqlite>) -> Result<Self> {
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS users(
                id INTEGER PRIMARY KEY,
                username varchar(128) NOT NULL,
                password varchar(128) NOT NULL,
                alias varchar(128) NOT NULL,
                is_admin BOOLEAN NOT NULL
            );
            CREATE INDEX IF NOT EXISTS ui_username ON users (username);
            CREATE INDEX IF NOT EXISTS ui_password ON users (password);
            CREATE INDEX IF NOT EXISTS ui_alias ON users (alias);
            CREATE INDEX IF NOT EXISTS ui_is_admin ON users (is_admin);",
        )
        .execute(&db)
        .await
        .unwrap();
        Ok(UserSystem {
            db,
            user_tokens: Arc::new(RwLock::new(HashMap::new())),
        })
    }

    pub async fn is_guest_enabled(&self) -> bool {
        sqlx::query("SELECT id FROM users WHERE username='guest' LIMIT 1")
            .fetch_optional(&self.db)
            .await
            .expect("Check guest enabled failed!")
            .is_some()
    }

    pub async fn is_guest_password_required(&self) -> bool {
        if let Some(row) = sqlx::query("SELECT password FROM users WHERE username='guest' LIMIT 1")
            .fetch_optional(&self.db)
            .await
            .expect("Check guest enabled failed!")
        {
            let password: String = row.get("password");
            password != ""
        } else {
            false
        }
    }

    pub async fn create_user(&self, v: UserToCreate, is_admin: bool) -> Result<()> {
        sqlx::query("INSERT INTO users (username, alias, password, is_admin) VALUES (?, ?, ?, ?)")
            .bind(&v.username)
            .bind(&v.alias)
            .bind(&v.password)
            .bind(is_admin)
            .execute(&self.db)
            .await?;
        Ok(())
    }

    pub async fn delete_user(&self, user: &UserInfo) -> Result<bool> {
        let r = sqlx::query("DELETE FROM users WHERE username=?")
            .bind(&user.username)
            .execute(&self.db)
            .await?;
        let token = self.get_user_token(&user);
        self.logout(&token).await;
        Ok(r.rows_affected() > 0)
    }

    pub async fn update_user(&self, user_old: &UserInfo, user_update: UserToCreate) -> bool {
        let r = sqlx::query("UPDATE users SET username=?, alias=?, password=? WHERE username=?")
            .bind(&user_update.username)
            .bind(&user_update.alias)
            .bind(&user_update.password)
            .bind(&user_old.username)
            .execute(&self.db)
            .await
            .expect("Update user failed!");
        let token = self.get_user_token(&user_old);
        self.logout(&token).await;
        r.rows_affected() > 0
    }

    pub async fn get_user(&self, username: &str) -> UserInfo {
        sqlx::query_as::<_, UserInfo>("SELECT * FROM users WHERE username = ? LIMIT 1")
            .bind(username)
            .fetch_one(&self.db)
            .await
            .expect("Get User failed!")
    }

    fn get_users_core_query(&self, main: &str, to_search: Option<&str>) -> QueryBuilder<Sqlite> {
        let mut builder = QueryBuilder::new(main);
        if let Some(s) = to_search {
            let s = format!("%{s}%");
            builder.push(" WHERE username = ").push_bind(s);
        }
        builder
    }

    pub async fn get_users(
        &self,
        index: usize,
        limit: usize,
        to_search: Option<&str>,
    ) -> Vec<UserInfo> {
        self.get_users_core_query("SELECT * FROM users", to_search)
            .push(" LIMIT ")
            .push_bind(limit as i64)
            .push(" OFFSET ")
            .push_bind((index * limit) as i64)
            .build_query_as()
            .fetch_all(&self.db)
            .await
            .expect("Get all users failed!")
    }

    pub async fn get_total_user(&self, to_search: Option<&str>) -> usize {
        self.get_users_core_query("SELECT COUNT(1) AS count FROM users", to_search)
            .build()
            .fetch_optional(&self.db)
            .await
            .expect("Get total media failed!")
            .map(|row| row.get::<u32, _>("count") as usize)
            .unwrap_or(0)
    }

    pub async fn exists_user(&self, username: &str) -> bool {
        sqlx::query("SELECT id FROM users WHERE username = ? LIMIT 1")
            .bind(username)
            .fetch_optional(&self.db)
            .await
            .expect("Check exists user failed!")
            .is_some()
    }

    pub async fn contains_users(&self) -> bool {
        sqlx::query("SELECT id FROM users LIMIT 1")
            .fetch_optional(&self.db)
            .await
            .expect("Check contains users failed!")
            .is_some()
    }

    pub fn get_user_token(&self, user: &UserInfo) -> String {
        let ident = format!("{}+{}", user.username, user.password);
        myutil::calc_hash(&ident).to_string()
    }

    pub async fn login(&self, username: &str, password: &str) -> Option<String> {
        if self.exists_user(username).await {
            let user = self.get_user(username).await;
            if user.password == password {
                let token = self.get_user_token(&user);
                self.user_tokens
                    .write()
                    .await
                    .insert(token.clone(), user.clone());
                Some(token)
            } else {
                None
            }
        } else {
            None
        }
    }

    pub async fn logout(&self, token: &str) -> bool {
        match self.verify(token).await {
            Some(user) => {
                if user.username != "guest" {
                    self.user_tokens.write().await.remove(token);
                }
                true
            }
            None => false,
        }
    }

    pub async fn verify(&self, token: &str) -> Option<UserInfo> {
        self.user_tokens.read().await.get(token).map(|u| u.clone())
    }
}
