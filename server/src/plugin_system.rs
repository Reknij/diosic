pub mod model;

use std::{collections::HashMap, fmt::Debug, path::PathBuf, sync::Arc};

use crate::config::Config;
use tokio::sync::RwLock;
use tracing::info;
use walkdir::WalkDir;

use self::model::PluginsContext;

#[derive(Clone)]
pub struct PluginSystem {
    config: Arc<Config>,
    plugins: Arc<RwLock<HashMap<String, PathBuf>>>,
}

impl Debug for PluginSystem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PluginSystem")
            .field("config", &self.config)
            .finish()
    }
}

impl PluginSystem {
    pub async fn new(config: Arc<Config>) -> Self {
        PluginSystem {
            config,
            plugins: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn scan(config: Arc<Config>) -> HashMap<String, PathBuf> {
        info!("Scanning all plugin..");
        let mut plugins = HashMap::new();

        let plugins_dir = config
            .data_path
            .clone()
            .map(|p| {
                let p = p.join("plugins");
                if p.is_dir() {
                    Some(p)
                } else {
                    None
                }
            })
            .unwrap_or(None);
        if let Some(plugins_path) = plugins_dir {
            let wasm_dirs = WalkDir::new(plugins_path)
                .follow_links(false)
                .into_iter()
                .filter_map(|file| {
                    file.ok().and_then(|f| {
                        let dir = f.path();
                        if dir.is_dir() {
                            let filename = dir
                                .file_name()
                                .map(|s| {
                                    if let Some(str) = s.to_str() {
                                        Some(str)
                                    } else {
                                        None
                                    }
                                })
                                .unwrap_or(None);
                            if filename.is_some() && filename.unwrap().starts_with("diosic-plugin-")
                            {
                                return Some(f);
                            }
                        }
                        None
                    })
                });
            for dir in wasm_dirs {
                let dir_path = dir.path();
                let main_wasm_file = dir_path.join("main.wasm");
                if !main_wasm_file.is_file() {
                    continue;
                }
                let name = dir.file_name().to_string_lossy().into_owned();
                plugins.insert(name, main_wasm_file);
            }
        }
        info!("Joined {} plugins.", plugins.len());
        plugins
    }
    pub async fn reload(&self) {
        *self.plugins.write().await = Self::scan(self.config.clone()).await;
    }

    pub async fn exists_plugins(&self) -> bool {
        self.plugins.read().await.len() > 0
    }

    pub async fn init_plugins_context(&self) -> PluginsContext {
        info!("Initialing plugins context..");
        let plugins = self.plugins.read().await;
        PluginsContext::new(&plugins).await
    }
}
