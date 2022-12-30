use std::path::PathBuf;

use serde::{Serialize, Deserialize};
use tokio::fs;
use tracing::error;

use crate::library_sistem::MediaLibrary;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    pub libraries: Vec<MediaLibrary>,
    pub data_path: Option<PathBuf>,
    pub address: String,
    pub port: u16,
}

impl Config {
    pub async fn load_from_path(path: &PathBuf)-> Result<Config, ()> {
        if path.is_file() {
            let content = fs::read_to_string(path).await;
            match content {
                Ok(content) => {
            let config = serde_json::from_str::<Config>(&content);
                    match config {
                        Ok(config)=> Ok(config),
                        Err(err)=> {
                            error!("{:?}", err);
                            Err(())
                        }
                    }

                },
                Err(err)=> {
                    error!("{:?}", err);
                    Err(())
                }
            }           
        }
        else {
            error!("Is not a file!");
            panic!("Is not a file!");
        }
    }
}