use std::path::PathBuf;

use actix_cors::Cors;
use actix_files::NamedFile;
use actix_web::{
    web::{self},
    App, Error, HttpRequest, HttpServer,
};
use tokio::{sync::RwLock, time::Instant};
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

pub struct ServerData {
    pub user_system: UserSystem,
    pub library_system: LibrarySystem,
    pub plugin_system: PluginSystem,
}

pub async fn run(config: &Config, s: ServerData) -> Result<(), std::io::Error> {
    info!("Running server in {}:{}", &config.address, &config.port);
    let usersys = web::Data::new(s.user_system);
    let libsys = web::Data::new(RwLock::new(s.library_system));
    let plgsys = web::Data::new(s.plugin_system);
    let start = web::Data::new(Instant::now());

    HttpServer::new(move || {
        App::new()
            .wrap(TracingLogger::default())
            .wrap(Cors::permissive())
            .app_data(usersys.clone())
            .app_data(libsys.clone())
            .app_data(plgsys.clone())
            .app_data(start.clone())
            .service(
                web::scope("api")
                    .service(api::get_server_info)
                    .service(api::require_setup)
                    .service(api::setup)
                    .service(api::get_media_file)
                    .service(api::get_media_cover)
                    .service(api::get_media_info)
                    .service(api::search_media)
                    .service(api::get_libraries)
                    .service(api::get_albums)
                    .service(api::get_categories)
                    .service(api::get_artists)
                    .service(api::get_genres)
                    .service(api::get_years)
                    .service(api::get_library_info)
                    .service(api::get_album_info)
                    .service(api::get_category_info)
                    .service(api::get_artist_info)
                    .service(api::get_genre_info)
                    .service(api::get_year_info)
                    .service(api::get_medias)
                    .service(api::create_user)
                    .service(api::delete_user)
                    .service(api::get_user)
                    .service(api::get_users)
                    .service(api::search_user)
                    .service(api::update_user)
                    .service(api::login_user)
                    .service(api::logout_user)
                    .service(api::get_current_user)
                    .service(api::scan_libraries),
            )
            .service(
                actix_files::Files::new("/", "./webpage")
                    .show_files_listing()
                    .index_file("index.html"),
            )
            .default_service(web::to(index))
    })
    .bind((config.address.to_owned(), config.port))?
    .run()
    .await?;

    Ok(())
}

async fn index(_req: HttpRequest) -> Result<NamedFile, Error> {
    let path: PathBuf = "./webpage/index.html".parse().unwrap();
    Ok(NamedFile::open(path)?)
}
