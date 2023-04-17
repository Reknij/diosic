use std::{path::PathBuf, sync::Arc};

use serde::{Deserialize, Serialize};
use tokio::{fs, sync::RwLock};
use tracing::{error, info};

use crate::library_system::MediaLibrary;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    pub libraries: Vec<MediaLibrary>,
    pub data_path: Option<PathBuf>,
    pub address: String,
    pub port: u16,
}

#[derive(Debug, Clone)]
pub struct SetupConfigHelper {
    config: Arc<Config>,
    setup_config: Arc<RwLock<SetupConfig>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SetupConfig {
    pub guest_enable: bool,
    pub guest_password: Option<String>,
}

impl Config {
    pub async fn load_from_path(path: &PathBuf) -> Result<Config, ()> {
        if path.is_file() {
            let content = fs::read_to_string(path).await;
            match content {
                Ok(content) => {
                    let config = serde_json::from_str::<Config>(&content);
                    match config {
                        Ok(config) => Ok(config),
                        Err(err) => {
                            error!("{:?}", err);
                            Err(())
                        }
                    }
                }
                Err(err) => {
                    error!("{:?}", err);
                    Err(())
                }
            }
        } else {
            error!("Is not a file!");
            panic!("Is not a file!");
        }
    }
}

impl SetupConfigHelper {
    pub async fn new(config: Arc<Config>) -> SetupConfigHelper {
        let default_setup_config = SetupConfig {
            guest_enable: false,
            guest_password: None,
        };

        if let Some(dp) = &config.data_path {
            let path = dp.join("setup.json");
            if path.exists() {
                let content = fs::read_to_string(path).await;
                match content {
                    Ok(content) => {
                        let setup_config = serde_json::from_str::<SetupConfig>(&content);
                        match setup_config {
                            Ok(setup_config) => {
                                return SetupConfigHelper {
                                    config,
                                    setup_config: Arc::new(RwLock::new(setup_config)),
                                }
                            }
                            Err(err) => {
                                error!("{:?}", err);
                            }
                        }
                    }
                    Err(err) => {
                        error!("{:?}", err);
                    }
                }
            }
        }

        SetupConfigHelper {
            config,
            setup_config: Arc::new(RwLock::new(default_setup_config)),
        }
    }

    pub async fn setup_config(&self)-> tokio::sync::RwLockReadGuard<'_, SetupConfig, > {
        self.setup_config.read().await
    }

    pub async fn update(&self, setup: SetupConfig) {
        *self.setup_config.write().await = setup;
    }

    pub async fn save(&self) {
        if let Some(dp) = &self.config.data_path {
            let path = dp.join("setup.json");
            if path.exists() {
                fs::remove_file(&path).await.unwrap();
            }
            let content = serde_json::to_vec(&*self.setup_config.read().await);
                match content {
                    Ok(content) => {
                        match fs::write(path, content).await {
                            Ok(_)=> {
                                info!("save setup config success.");
                            }
                            Err(err)=> {
                                error!("save setup config occurs: {}", err);
                            }
                        };
                    }
                    Err(err) => {
                        error!("failed to serialize json to vec: {:?}", err);
                    }
                }
        }
    }
}
