use std::sync::Arc;

use clap::Parser;
use tracing::Level;
use tracing_subscriber::FmtSubscriber;

use crate::meta::{Commands, Meta};

mod config;
mod db;
mod library_system;
mod meta;
mod myutil;
mod plugin_system;
mod server;
mod user_system;

#[tokio::main]
async fn main() {
    let meta = Meta::parse();

    #[cfg(debug_assertions)]
    {
        std::env::set_var("RUST_LOG", "actix_web=debug");
    }
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber).unwrap();
    tracing::info!("Starting tracing!");

    match meta.command {
        Commands::Serve => {
            let config = Arc::new(if let Some(config_path) = meta.config {
                config::Config::load_from_path(&config_path)
                    .await
                    .expect("Load config from path failed!")
            } else {
                config::Config::load_from_meta(meta)
                    .await
                    .expect("Load config from meta failed!")
            });
            let db = db::init(config.clone())
                .await
                .expect("Initialize database failed!");
            let user_system = user_system::UserSystem::new(db.clone())
                .await
                .expect("Initialize user system failed!");
            let plugin_system = plugin_system::PluginSystem::new(config.clone()).await;
            plugin_system.reload().await;
            let library_system =
                library_system::LibrarySystem::new(db.clone(), config.clone()).await;
            library_system.reload(&plugin_system).await;
            let s = server::AppState {
                user_system,
                library_system,
                plugin_system,
                config: config.clone(),
            };
            server::run(config, s).await.expect("Run server error!");
        }
    }
}
