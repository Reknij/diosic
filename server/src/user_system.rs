use rusqlite::{params, Connection, Error};
use std::{collections::HashMap, sync::Arc};
use tokio::sync::Mutex;

use crate::{
    config::{SetupConfig, SetupConfigHelper},
    myutil::{self, DiosicID},
};

#[derive(Debug, Clone)]
pub struct UserSystem {
    db: Arc<Mutex<Connection>>,
    users_token: Arc<Mutex<HashMap<myutil::DiosicID, UserInfo>>>,
    setup_config_helper: Arc<SetupConfigHelper>,
}

#[derive(Debug, Hash, Clone)]
pub struct UserInfo {
    pub username: String,
    pub password: String,
    pub alias: String,
    pub is_admin: bool,
}

impl UserSystem {
    pub async fn new(
        db: Arc<Mutex<Connection>>,
        setup_config_helper: Arc<SetupConfigHelper>,
    ) -> Result<Self, Error> {
        create_if_no_exists_table(&*db.lock().await)?;

        Ok(UserSystem {
            db,
            users_token: Arc::new(Mutex::new(HashMap::new())),
            setup_config_helper,
        })
    }

    pub async fn guest_enable(&self, enable: bool, pass: Option<String>) {
        let sc = SetupConfig {
            guest_enable: enable,
            guest_password: pass,
        };
        self.setup_config_helper.update(sc).await;
        self.setup_config_helper.save().await;
    }

    pub async fn create_user(&self, user: &UserInfo) -> Result<(), Error> {
        self.db.lock().await.execute(
            "INSERT INTO user (username, alias, password, is_admin) VALUES (?, ?, ?, ?)",
            params![user.username, user.alias, user.password, user.is_admin],
        )?;
        Ok(())
    }

    pub async fn delete_user(&self, username: &str) -> Result<(), Error> {
        if let Ok(user) = self.get_user(username).await {
            self.db
                .lock()
                .await
                .execute("DELETE FROM user WHERE username=?", params![username])?;
            let token: myutil::DiosicID = myutil::calc_hash(&user).to_string().into();
            self.logout(&token).await;
            Ok(())
        } else {
            Err(Error::InvalidQuery)
        }
    }

    pub async fn update_user(
        &self,
        user_old: &UserInfo,
        user_update: UserInfo,
    ) -> Result<bool, Error> {
        let n = self.db.lock().await.execute(
            "UPDATE user SET username=?, alias=?, password=?, is_admin=? WHERE username=?",
            params![
                user_update.username,
                user_update.alias,
                user_update.password,
                user_update.is_admin,
                user_old.username
            ],
        )?;
        let token: myutil::DiosicID = myutil::calc_hash(&user_old).to_string().into();
        self.logout(&token).await;
        Ok(n > 0)
    }

    pub async fn get_user(&self, username: &str) -> Result<UserInfo, String> {
        let sc = &self.setup_config_helper.setup_config().await;
        if sc.guest_enable && username.to_lowercase() == "guest" {
            let password = sc.guest_password.clone().unwrap_or("".to_owned());
            return Ok(UserInfo {
                username: "guest".to_owned(),
                password,
                alias: "Guest".to_owned(),
                is_admin: false,
            });
        }
        let db = self.db.lock().await;
        let mut stmt = db
            .prepare(
                "SELECT username, alias, password, is_admin FROM user WHERE username = ? LIMIT 1",
            )
            .unwrap();

        let mut result = stmt
            .query_map(params![username], |row| {
                Ok(UserInfo {
                    username: row.get(0)?,
                    alias: row.get(1)?,
                    password: row.get(2)?,
                    is_admin: row.get(3)?,
                })
            })
            .unwrap();

        match result.next() {
            Some(user) => user.map_err(|err| err.to_string()),
            None => Err(format!("Can't found user with `{}`!", username)),
        }
    }

    pub async fn get_users(&self, index: usize, limit: usize) -> Result<Vec<UserInfo>, String> {
        let db = self.db.lock().await;
        let mut stmt = db
            .prepare("SELECT username, alias, password, is_admin FROM user LIMIT ? OFFSET ?")
            .unwrap();

        let result = stmt
            .query_map(params![limit, index], |row| {
                Ok(UserInfo {
                    username: row.get(0)?,
                    alias: row.get(1)?,
                    password: row.get(2)?,
                    is_admin: row.get(3)?,
                })
            })
            .unwrap();

        let mut users = Vec::new();

        for r in result {
            users.push(r.map_err(|err| err.to_string())?);
        }

        Ok(users)
    }
    pub async fn get_users_by_search(
        &self,
        to_search: &str,
        index: usize,
        limit: usize,
    ) -> Result<(Vec<UserInfo>, usize), String> {
        let db = self.db.lock().await;
        let mut stmt = db
            .prepare("SELECT username, alias, password, is_admin FROM user WHERE username = ? LIMIT ? OFFSET ?")
            .unwrap();

        let result = stmt
            .query_map(params![to_search, limit, index], |row| {
                Ok(UserInfo {
                    username: row.get(0)?,
                    alias: row.get(1)?,
                    password: row.get(2)?,
                    is_admin: row.get(3)?,
                })
            })
            .unwrap();

        let mut users = Vec::new();

        for r in result {
            users.push(r.map_err(|err| err.to_string())?);
        }

        let count = db.query_row(
            "SELECT COUNT(*) FROM user WHERE username = ?",
            params![to_search],
            |row| Ok(row.get(0).unwrap()),
        );
        Ok((users, count.unwrap()))
    }

    pub async fn exists_user(&self, username: &str) -> bool {
        self.get_user(username).await.is_ok()
    }

    pub async fn have_user(&self) -> bool {
        let db = self.db.lock().await;
        let mut stmt = db.prepare("SELECT COUNT(*) FROM user").unwrap();
        let count = stmt
            .query_row(params![], |row| {
                let c: u64 = row.get(0)?;
                Ok(c)
            })
            .unwrap();

        count > 0
    }

    pub async fn login(&self, username: &str, password: &str) -> Result<DiosicID, String> {
        if let Ok(user) = self.get_user(username).await {
            if user.password == password {
                let token: myutil::DiosicID = myutil::calc_hash(&user).to_string().into();
                self.users_token
                    .lock()
                    .await
                    .insert(token.clone(), user.clone());
                Ok(token)
            } else {
                Err("Password verify failed!".into())
            }
        } else {
            Err("Can't found user!".into())
        }
    }

    pub async fn logout(&self, token: &DiosicID) -> bool {
        match self.verify(&token).await {
            Some(user) => {
                if user.username != "guest" {
                    self.users_token.lock().await.remove(&token);
                }
                true
            }
            None => false,
        }
    }

    pub async fn verify(&self, token: &DiosicID) -> Option<UserInfo> {
        self.users_token.lock().await.get(&token).map(|u| u.clone())
    }
}

pub fn create_if_no_exists_table(db: &Connection) -> Result<(), Error> {
    db.execute(
        "CREATE TABLE IF NOT EXISTS user(
        username varchar(64) NOT NULL,
        alias varchar(64) NOT NULL,
        is_admin boolean NOT NULL,
        password varchar(128) NOT NULL)",
        params![],
    )?;

    Ok(())
}
