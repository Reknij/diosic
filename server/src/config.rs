use std::{
    collections::HashSet,
    path::{Path, PathBuf},
};

use crate::{library_system::model::LibraryInfo, meta::Meta};
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use tokio::fs;
use tracing::warn;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    pub libraries: Vec<LibraryInfo>,
    pub data_path: Option<PathBuf>,
    pub covers_cached_path: Option<PathBuf>,
    pub host: String,
    pub port: u16,
    #[serde(default)]
    pub public_url: String,
}

impl Config {
    pub async fn load_from_path(path: &PathBuf) -> Result<Config> {
        if path.is_file() {
            let content = fs::read_to_string(path).await?;
            let config = serde_json::from_str::<Config>(&content)
                .with_context(|| "Deserialize config content failed!")?;
            config.clears().await?;
            Ok(config)
        } else {
            panic!("The `config_path` is not the file.");
        }
    }

    pub async fn clears(&self) -> Result<()> {
        if let Some(covers_cached_path) = &self.covers_cached_path {
            if covers_cached_path.is_dir() {
                fs::remove_dir_all(covers_cached_path)
                    .await
                    .with_context(|| "Remove `cover cached directory` failed!")?;
            }
            fs::create_dir(covers_cached_path)
                .await
                .with_context(|| "Create `cover cached directory` failed!")?;
        }
        Ok(())
    }

    pub async fn load_from_meta(meta: Meta) -> Result<Self> {
        let config = Self {
            libraries: to_libraries(meta.library.as_ref().unwrap_or(&vec![])),
            covers_cached_path: meta.data_path.clone().map(|p| p.join("covers_cached")),
            data_path: meta.data_path,
            host: meta.host,
            port: meta.port,
            public_url: meta.public_url,
        };
        config.clears().await?;
        Ok(config)
    }
}

fn to_libraries(title_with_paths: &Vec<String>) -> Vec<LibraryInfo> {
    let mut libraries = Vec::with_capacity(title_with_paths.len());
    let mut count_unknown = 1;
    let mut contain_names = HashSet::with_capacity(title_with_paths.len());

    for tp in title_with_paths {
        let tp: Vec<&str> = tp.split(";").collect();
        match tp.len() {
            0 => {
                panic!("Please ensure at least one library is settled.")
            }
            1 => {
                let title = format!("Unknown_{}", count_unknown);
                let path = Path::new(tp[0]).to_path_buf();

                if contain_names.contains(&title) {
                    warn!(
                        "Can't import library `{}` because already contain target library name.",
                        title
                    )
                } else if path.is_dir() {
                    contain_names.insert(title.to_owned());
                    libraries.push(LibraryInfo { title, path });
                    count_unknown += 1;
                } else {
                    warn!(
                        "Can't import library `{}` with path `{}` because it is not directory.",
                        title,
                        path.to_str().unwrap()
                    )
                }
            }
            2 => {
                let title = tp[0].to_owned();
                let path = Path::new(tp[1]).to_path_buf();

                if contain_names.contains(&title) {
                    warn!(
                        "Can't import library `{}` because already contain target library name.",
                        title
                    )
                } else if path.is_dir() {
                    contain_names.insert(title.to_owned());
                    libraries.push(LibraryInfo { title, path });
                } else {
                    warn!(
                        "Can't import library `{}` with path `{}` because it not directory.",
                        title,
                        path.to_str().unwrap()
                    )
                }
            }
            _ => {
                panic!("Please ensure format `media` option is `library_title;library_dir_path`")
            }
        }
    }

    libraries
}
