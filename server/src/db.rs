use std::path::Path;

use rusqlite::{Connection, Result};
use tracing::warn;

use crate::config::Config;

pub fn init(config: &Config) -> Result<Connection> {
    let conn = if let Some(data_path) = &config.data_path {
        if data_path.is_dir() {
            let db_path = Path::new(data_path).join("diosic.db");
            Connection::open(&db_path)?
        } else {
            warn!("Please ensure `data_path` is path of directory! Now will save in memory.");
            Connection::open_in_memory()?
        }
    } else {
        warn!("No input db_path in config, will save in memory!");
        Connection::open_in_memory()?
    };

    Ok(conn)
}
