use std::{
    path::{Path, PathBuf},
    sync::Arc, collections::HashSet,
};

use clap::{command, Parser};
use tokio::sync::Mutex;
use tracing::{warn, Level};
use tracing_subscriber::FmtSubscriber;

mod config;
mod db;
mod library_system;
mod metadata;
mod myutil;
mod server;
mod user_system;
mod plugin_system;

#[tokio::main]
async fn main() {
    let args = Args::parse();

    #[cfg(debug_assertions)]
    {
        std::env::set_var("RUST_LOG", "actix_web=debug");
    }
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::DEBUG)
        .finish();
    tracing::subscriber::set_global_default(subscriber).unwrap();
    tracing::info!("Starting tracing!");

    let config = Arc::new(if let Some(config_path) = args.config {
        config::Config::load_from_path(&config_path)
            .await
            .expect("Load config failed!")
    } else {
        config::Config {
            libraries: to_libraries(&args.library.unwrap_or_else(||Vec::new())),
            data_path: args.data_path,
            address: args.address,
            port: args.port,
        }
    });
    let db = Arc::new(Mutex::new(
        db::init(&config).expect("Initialize database failed!"),
    ));
    let setup_config_helper = Arc::new(config::SetupConfigHelper::new(config.clone()).await);
    let users = user_system::UserSystem::new(db.clone(), setup_config_helper.clone())
        .await
        .expect("Initialize user system failed!");
    let pluginsys = plugin_system::PluginSystem::new(config.clone()).await;
    let librarys = library_system::LibrarySystem::new(config.clone(), &pluginsys).await;

    let s = server::ServerData {
        user_system: users,
        library_system: librarys,
        plugin_system: pluginsys,
        setup_config_helper
    };
    server::run(&config, s).await.expect("Run server error!");
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    // path of config file.
    #[arg(short, long)]
    config: Option<PathBuf>,

    #[arg(short, long)]
    data_path: Option<PathBuf>,

    #[arg(short, long)]
    library: Option<Vec<String>>,

    #[arg(short, long, default_value = "0.0.0.0")]
    address: String,

    #[arg(short, long, default_value = "3177")]
    port: u16,
}

fn to_libraries(title_with_paths: &Vec<String>) -> Vec<library_system::MediaLibrary> {
    let mut libraries = Vec::with_capacity(title_with_paths.len());
    let mut count_unknown = 1;
    let mut contain_names = HashSet::new();

    for tp in title_with_paths {
        let tp: Vec<&str> = tp.split(";").collect();
        if tp.len() == 2 {
            let title = tp[0].to_owned();
            let path = Path::new(tp[1]).to_path_buf();

            if contain_names.contains(&title) {
                warn!("Can't import library `{}` because already contain target library name.", title)
            }
            else if path.is_dir() {
                contain_names.insert(title.to_owned());
                libraries.push(library_system::MediaLibrary { title, path });
            }
            else {
                warn!("Can't import library `{}` with path `{}` because it not directory.", title, path.to_str().unwrap())
            }
        } else if tp.len() > 2 {
            panic!("Please ensure format `media` option is `media_title;media_path`")
        } else if tp.len() == 1 {
            let title = format!("Unknown_{}", count_unknown);
            let path = Path::new(tp[0]).to_path_buf();
            
            if contain_names.contains(&title) {
                warn!("Can't import library `{}` because already contain target library name.", title)
            }
            else if path.is_dir() {
                contain_names.insert(title.to_owned());
                libraries.push(library_system::MediaLibrary {
                    title,
                    path,
                });
                count_unknown += 1;
            }
            else {
                warn!("Can't import library `{}` with path `{}` because it not directory.", title, path.to_str().unwrap())
            }
        } else {
            warn!("Unknown media library.");
        }
    }

    libraries
}
