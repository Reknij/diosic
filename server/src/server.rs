use std::{path::PathBuf, sync::Arc};

use actix_cors::Cors;
use actix_files::NamedFile;
use actix_web::{
    web::{self},
    App, Error, HttpRequest, HttpServer,
};
use tokio::time::Instant;
use tracing::info;
use tracing_actix_web::TracingLogger;

use crate::{
    config::Config, library_system::LibrarySystem, plugin_system::PluginSystem,
    user_system::UserSystem,
};

mod api;
mod dto;
mod error;
mod from_requests;

pub struct AppState {
    pub user_system: UserSystem,
    pub library_system: LibrarySystem,
    pub plugin_system: PluginSystem,
    pub config: Arc<Config>,
}

pub async fn run(config: Arc<Config>, s: AppState) -> Result<(), std::io::Error> {
    info!("Running server in {}:{}", &config.host, &config.port);
    let state = web::Data::new(s);
    let start = web::Data::new(Instant::now());

    HttpServer::new(move || {
        App::new()
            .wrap(TracingLogger::default())
            .wrap(Cors::permissive())
            .app_data(state.clone())
            .app_data(start.clone())
            .service(
                web::scope("api")
                    .service(api::get_server_info)
                    .service(api::setup)
                    .service(api::get_media_file)
                    .service(api::get_media_cover)
                    .service(api::get_media_info)
                    .service(api::get_medias)
                    .service(api::get_sources)
                    .service(api::get_medias)
                    .service(api::create_user)
                    .service(api::delete_user)
                    .service(api::get_user)
                    .service(api::get_users)
                    .service(api::update_user)
                    .service(api::login_user)
                    .service(api::logout_user)
                    .service(api::get_current_user)
                    .service(api::reload_medias)
                    .service(api::reload_plugins),
            )
            .service(
                actix_files::Files::new("/", "./webpage")
                    .show_files_listing()
                    .index_file("index.html"),
            )
            .default_service(web::to(index))
    })
    .bind((config.host.to_owned(), config.port))?
    .run()
    .await?;

    Ok(())
}

async fn index(_req: HttpRequest) -> Result<NamedFile, Error> {
    let path: PathBuf = "./webpage/index.html".parse().unwrap();
    Ok(NamedFile::open(path)?)
}
