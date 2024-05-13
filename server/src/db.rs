use std::{path::Path, sync::Arc, time::Duration};

use crate::config::Config;
use anyhow::Result;
use sqlx::{
    sqlite::{SqliteConnectOptions, SqlitePoolOptions},
    Pool, Sqlite,
};
use tracing::warn;

pub async fn init(config: Arc<Config>) -> Result<Pool<Sqlite>> {
    let options = config.data_path.as_ref().map(Path::new).and_then(|p| {
        if p.is_dir() {
            Some(
                SqliteConnectOptions::new()
                    .filename(p.join("diosic.db"))
                    .busy_timeout(Duration::from_millis(6000))
                    .create_if_missing(true),
            )
        } else {
            None
        }
    });
    let pool_options = SqlitePoolOptions::new().max_connections(12);
    Ok(match options {
        Some(options) => pool_options.connect_with(options).await?,
        None => {
            warn!("Don't exists the data path. Will use memory database.");
            pool_options.connect("sqlite::memory:").await?
        }
    })
}
