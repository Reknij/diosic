use std::sync::Arc;

use actix_files::NamedFile;
use actix_web::{
    delete, get, post, put,
    web::{self, Json},
    HttpResponse,
};
use tokio::{sync::RwLock, time::Instant};
use tracing::error;

use crate::myutil::DiosicID;

use super::dto;
use super::error::APIErrorType::*;
use super::error::*;
use super::from_requests::*;

type LibSys = web::Data<RwLock<crate::library_system::LibrarySystem>>;
type UserSys = web::Data<crate::user_system::UserSystem>;
type PlgSys = web::Data<RwLock<crate::plugin_system::PluginSystem>>;
type SCHelper = web::Data<Arc<crate::config::SetupConfigHelper>>;

#[get("/setup_info")]
pub async fn setup_info(user_system: UserSys, schelper: SCHelper) -> Json<dto::SetupInfo> {
    let c = schelper.setup_config().await;
    Json(dto::SetupInfo {
        admin_required: !user_system.have_user().await,
        guest_enable: c.guest_enable,
        guest_password_required: c.guest_password.is_some(),
    })
}

#[post("/setup")]
pub async fn setup(
    user_system: UserSys,
    setup: Json<dto::ToSetup>,
    permission: UserPermission,
) -> Result<HttpResponse, APIError> {
    let mut setup = setup.0.clone();
    let setup_fn = || async {
        setup.process();
        if setup.guest_enable {
            if let Some(pass) = &setup.guest_password {
                if pass.len() < 8 {
                    return Err(
                        APIError::with(Unexpected).note("Password length must more than 8.")
                    );
                }
            }
        }
        user_system
            .create_user(&setup.admin.into())
            .await
            .map_err(|err| {
                APIError::with(Unexpected).note(format!("Create user error: {}", err.to_string()))
            })?;
        user_system
            .guest_enable(setup.guest_enable, setup.guest_password)
            .await;
        Ok(HttpResponse::Ok().finish())
    };
    if user_system.have_user().await {
        if permission.is_admin() {
            match user_system
                .delete_user(&permission.get_owner().unwrap().username)
                .await
            {
                Ok(_) => (),
                Err(err) => error!("can't delete current admin: {}", err),
            };
            setup_fn().await
        } else {
            Err(APIError::with(NoPermission).note("Only admin can setup again."))
        }
    } else {
        setup_fn().await
    }
}

#[get("/media_file/{id}")]
pub async fn get_media_file(
    info: web::Path<(DiosicID,)>,
    library_system: LibSys,
    permission: UserPermission,
) -> Result<NamedFile, APIError> {
    if !permission.exists_owner() {
        return Err(APIError::with(NoPermission).note("Please log in first!"));
    }
    match library_system
        .read()
        .await
        .get_media_file_by_id(&info.0)
        .await
    {
        Some(media) => NamedFile::open_async(&media)
            .await
            .map_err(|err| APIError::with(Unexpected).note(err.to_string())),
        None => Err(APIError::with(NoFound).note("Can't get target media by hash id.")),
    }
}

#[get("/media_cover/{id}")]
pub async fn get_media_cover(
    info: web::Path<(DiosicID,)>,
    library_system: LibSys,
    permission: UserPermission,
) -> Result<NamedFile, APIError> {
    if !permission.exists_owner() {
        return Err(APIError::with(NoPermission).note("Please log in first!"));
    }

    match library_system
        .read()
        .await
        .get_media_cover_by_id(&info.0)
        .await
    {
        Some(img) => NamedFile::open_async(&img)
            .await
            .map_err(|err| APIError::with(Unexpected).note(err.to_string())),
        None => Err(APIError::with(NoFound).note("Can't get target media cover.")),
    }
}

#[get("/media_info/{id}")]
pub async fn get_media_info(
    info: web::Path<(DiosicID,)>,
    library_system: LibSys,
    permission: UserPermission,
) -> Result<Json<dto::MediaInfo>, APIError> {
    if !permission.exists_owner() {
        return Err(APIError::with(NoPermission).note("Please log in first!"));
    }
    match library_system
        .read()
        .await
        .get_media_info_by_id(&info.0)
        .await
    {
        Some(info) => Ok(Json(info.into())),
        None => Err(APIError::with(NoFound).note("No found media with id!")),
    }
}

#[get("/medias/search")]
pub async fn search_media(
    query: web::Query<dto::SearchMediaQuery>,
    library_system: LibSys,
    permission: UserPermission,
) -> Result<Json<dto::SearchResult<dto::MediaInfo>>, APIError> {
    if !permission.exists_owner() {
        return Err(APIError::with(NoPermission).note("Please log in first!"));
    }
    let libsys_read = library_system.read().await;
    let is_filter = query.source.is_some() && query.filter.is_some();
    let source = query.source.clone().unwrap_or_else(|| "".to_owned());
    let filter = query.filter.clone().unwrap_or_else(|| "".to_owned());
    let medias = libsys_read
        .get_medias_by_search(
            &query.content,
            &source,
            &filter,
            is_filter,
            query.index,
            query.limit,
        )
        .await;

    let result: Vec<dto::MediaInfo> = medias.iter().map(|m| dto::MediaInfo::from(m)).collect();
    Ok(Json(dto::SearchResult {
        content: result,
        length: medias.len(),
    }))
}

#[get("/libraries")]
pub async fn get_libraries(
    library_system: LibSys,
    permission: UserPermission,
) -> Result<Json<Vec<dto::MediaSourceInfo>>, APIError> {
    if !permission.exists_owner() {
        return Err(APIError::with(NoPermission).note("Please log in first!"));
    }
    let libraries = library_system.read().await.get_libraries().await;
    Ok(Json(libraries.into_iter().map(|m| m.into()).collect()))
}

#[get("/albums")]
pub async fn get_albums(
    library_system: LibSys,
    permission: UserPermission,
) -> Result<Json<Vec<dto::MediaSourceInfo>>, APIError> {
    if !permission.exists_owner() {
        return Err(APIError::with(NoPermission).note("Please log in first!"));
    }
    let albums = library_system.read().await.get_albums().await;
    Ok(Json(albums.into_iter().map(|m| m.into()).collect()))
}

#[get("/categories")]
pub async fn get_categories(
    library_system: LibSys,
    permission: UserPermission,
) -> Result<Json<Vec<dto::MediaSourceInfo>>, APIError> {
    if !permission.exists_owner() {
        return Err(APIError::with(NoPermission).note("Please log in first!"));
    }
    let categories = library_system.read().await.get_categories().await;
    Ok(Json(categories.into_iter().map(|m| m.into()).collect()))
}

#[get("/artists")]
pub async fn get_artists(
    library_system: LibSys,
    permission: UserPermission,
) -> Result<Json<Vec<dto::MediaSourceInfo>>, APIError> {
    if !permission.exists_owner() {
        return Err(APIError::with(NoPermission).note("Please log in first!"));
    }
    let artists = library_system.read().await.get_artists().await;
    Ok(Json(artists.into_iter().map(|m| m.into()).collect()))
}

#[get("/genres")]
pub async fn get_genres(
    library_system: LibSys,
    permission: UserPermission,
) -> Result<Json<Vec<dto::MediaSourceInfo>>, APIError> {
    if !permission.exists_owner() {
        return Err(APIError::with(NoPermission).note("Please log in first!"));
    }
    let genres = library_system.read().await.get_genres().await;
    Ok(Json(genres.into_iter().map(|m| m.into()).collect()))
}

#[get("/years")]
pub async fn get_years(
    library_system: LibSys,
    permission: UserPermission,
) -> Result<Json<Vec<dto::MediaSourceInfo>>, APIError> {
    if !permission.exists_owner() {
        return Err(APIError::with(NoPermission).note("Please log in first!"));
    }
    let years = library_system.read().await.get_years().await;
    Ok(Json(years.into_iter().map(|m| m.into()).collect()))
}

#[get("/library_info")]
pub async fn get_library_info(
    library_system: LibSys,
    query: web::Query<dto::GetLibraryQuery>,
) -> Result<Json<dto::MediaSourceInfo>, APIError> {
    match library_system.read().await.get_library(&query.title).await {
        Some(v) => Ok(Json(v.into())),
        None => Err(APIError::with(NoFound).note("No found target library with title.")),
    }
}

#[get("/album_info")]
pub async fn get_album_info(
    library_system: LibSys,
    query: web::Query<dto::GetAlbumQuery>,
) -> Result<Json<dto::MediaSourceInfo>, APIError> {
    match library_system.read().await.get_album(&query.title).await {
        Some(v) => Ok(Json(v.into())),
        None => Err(APIError::with(NoFound).note("No found target album with title.")),
    }
}

#[get("/category_info")]
pub async fn get_category_info(
    library_system: LibSys,
    query: web::Query<dto::GetCategoryQuery>,
) -> Result<Json<dto::MediaSourceInfo>, APIError> {
    match library_system.read().await.get_category(&query.title).await {
        Some(v) => Ok(Json(v.into())),
        None => Err(APIError::with(NoFound).note("No found target category with title.")),
    }
}

#[get("/artist_info")]
pub async fn get_artist_info(
    library_system: LibSys,
    query: web::Query<dto::GetCategoryQuery>,
) -> Result<Json<dto::MediaSourceInfo>, APIError> {
    match library_system.read().await.get_artist(&query.title).await {
        Some(v) => Ok(Json(v.into())),
        None => Err(APIError::with(NoFound).note("No found target artist with title.")),
    }
}

#[get("/genre_info")]
pub async fn get_genre_info(
    library_system: LibSys,
    query: web::Query<dto::GetCategoryQuery>,
) -> Result<Json<dto::MediaSourceInfo>, APIError> {
    match library_system.read().await.get_genre(&query.title).await {
        Some(v) => Ok(Json(v.into())),
        None => Err(APIError::with(NoFound).note("No found target genre with title.")),
    }
}

#[get("/year_info")]
pub async fn get_year_info(
    library_system: LibSys,
    query: web::Query<dto::GetCategoryQuery>,
) -> Result<Json<dto::MediaSourceInfo>, APIError> {
    match library_system.read().await.get_year(&query.title).await {
        Some(v) => Ok(Json(v.into())),
        None => Err(APIError::with(NoFound).note("No found target year with title.")),
    }
}

#[get("/medias")]
pub async fn get_medias(
    library_system: LibSys,
    query: web::Query<dto::GetMediasQuery>,
    permission: UserPermission,
) -> Result<Json<Vec<dto::MediaInfo>>, APIError> {
    if !permission.exists_owner() {
        return Err(APIError::with(NoPermission).note("Please log in first!"));
    }
    let libsys = library_system.read().await;
    let medias = libsys
        .get_medias_by_source_with_filter(&query.source, &query.filter)
        .await;
    match medias {
        Some(medias) => {
            if query.index >= medias.len().try_into().unwrap() {
                Err(APIError::with(Unspecified).note("Index greater than length!"))
            } else {
                let mut limit_medias: Vec<dto::MediaInfo> =
                    Vec::with_capacity(query.limit.try_into().unwrap());
                let index = query.index * query.limit;
                let max = {
                    let max = index + query.limit;
                    if max > medias.len() {
                        medias.len()
                    } else {
                        max
                    }
                };
                for m in &medias[index..max] {
                    limit_medias.push(m.into());
                }
                Ok(Json(limit_medias))
            }
        }
        None => Ok(Json(Vec::new())),
    }
}

#[get("/user/{username}")]
pub async fn get_user(
    info: web::Path<(String,)>,
    user_system: UserSys,
    permission: UserPermission,
) -> Result<Json<dto::UserInfo>, APIError> {
    if !permission.exists_owner() {
        return Err(APIError::with(NoPermission).note("Please log in first!"));
    }
    if !permission.is_admin() {
        return Err(APIError::with(NoPermission).note("Only admin can access."));
    }

    let user = user_system.get_user(&info.0).await;
    match user {
        Ok(user) => Ok(Json(user.into())),
        Err(err) => Err(APIError::with(NoFoundUser).note(err.to_string())),
    }
}

#[get("/users")]
pub async fn get_users(
    user_system: UserSys,
    permission: UserPermission,
    query: web::Query<dto::GetUsersQuery>,
) -> Result<Json<Vec<dto::UserInfo>>, APIError> {
    if !permission.exists_owner() {
        return Err(APIError::with(NoPermission).note("Please log in first!"));
    }
    if !permission.is_admin() {
        return Err(APIError::with(NoPermission).note("Only admin can access."));
    }
    let index = query.index * query.limit;
    let users = user_system.get_users(index, query.limit).await;
    match users {
        Ok(users) => {
            let users: Vec<dto::UserInfo> = users.iter().map(|u| u.to_owned().into()).collect();
            Ok(Json(users))
        }
        Err(err) => Err(APIError::with(Unexpected).note(err.to_string())),
    }
}

#[get("/users/search")]
pub async fn search_user(
    user_sys: UserSys,
    query: web::Query<dto::SearchUserQuery>,
) -> Result<Json<dto::SearchResult<dto::UserInfo>>, APIError> {
    let index = query.index * query.limit;
    match user_sys
        .get_users_by_search(&query.content, index, query.limit)
        .await
    {
        Ok((result, count)) => Ok(Json(dto::SearchResult {
            content: result.into_iter().map(|u| u.into()).collect(),
            length: count,
        })),
        Err(err) => Err(APIError::with(Unspecified).note(err)),
    }
}

#[delete("/user/{username}")]
pub async fn delete_user(
    info: web::Path<(String,)>,
    user_system: UserSys,
    permission: UserPermission,
) -> Result<HttpResponse, APIError> {
    if !permission.exists_owner() {
        return Err(APIError::with(NoPermission).note("Please log in first!"));
    }

    if let Ok(user) = user_system.get_user(&info.0).await {
        if user.is_admin {
            return Err(APIError::with(NoPermission).note("Can't delete admin!"));
        }

        if permission.have_permission_with(&info.0) {
            match user_system.delete_user(&info.0).await {
                Ok(()) => Ok(HttpResponse::Accepted().finish()),
                Err(err) => Err(APIError::with(Unexpected).note(err.to_string())),
            }
        } else {
            Err(APIError::with(NoPermission).note("No have permission!".to_owned()))
        }
    } else {
        Err(APIError::with(NoFoundUser).note("No exists username!".to_owned()))
    }
}

#[put("/user")]
pub async fn update_user(
    user_system: UserSys,
    permission: UserPermission,
    to_update: Json<dto::UserInfo>,
) -> Result<HttpResponse, APIError> {
    if !permission.exists_owner() {
        return Err(APIError::with(NoPermission).note("Please log in first!"));
    }

    if permission.is_guest() {
        return Err(APIError::with(NoPermission).note("Guest can't update information."));
    }

    let user = permission.get_owner()?;
    if user_system.exists_user(&user.username).await {
        if permission.have_permission_with(&user.username) {
            match user_system
                .update_user(&user.into(), to_update.0.into())
                .await
            {
                Ok(success) => {
                    if success {
                        Ok(HttpResponse::Ok().finish())
                    } else {
                        Err(APIError::with(Unspecified).note("No found user to update."))
                    }
                }
                Err(err) => Err(APIError::with(Unexpected).note(err.to_string())),
            }
        } else {
            Err(APIError::with(NoPermission).note("No have permission!".to_owned()))
        }
    } else {
        Err(APIError::with(NoFoundUser).note("No exists username!".to_owned()))
    }
}

#[post("/user")]
pub async fn create_user(
    permission: UserPermission,
    user_system: UserSys,
    to_create: Json<dto::UserInfo>,
) -> Result<HttpResponse, APIError> {
    if !permission.exists_owner() {
        return Err(APIError::with(NoFoundUser).note("Please login first."));
    }

    if user_system.have_user().await {
        if to_create.is_admin {
            return Err(APIError::with(NoPermission).note("Admin user only can create in setup."));
        } else if !permission.is_admin() {
            return Err(APIError::with(NoPermission).note("Only admin can create user."));
        }
    }

    if to_create.alias.len() < 4 {
        return Err(APIError::with(Unexpected).note("Alias length must more than 4."));
    }

    if to_create.username.len() < 4 {
        return Err(APIError::with(Unexpected).note("Username length must more than 4."));
    }

    if to_create.password.len() < 8 {
        return Err(APIError::with(Unexpected).note("Password length must more than 8."));
    }

    if !to_create.username.is_ascii() || !to_create.password.is_ascii() {
        return Err(APIError::with(Unexpected).note(
            "Found illegal characters in username or password. Ensure it is legal characters.",
        ));
    }

    if to_create.username == "guest" {
        return Err(APIError::with(Unexpected).note("Username can't same with `guest`."));
    }

    if !user_system.exists_user(&to_create.username).await {
        match user_system.create_user(&to_create.0.into()).await {
            Ok(()) => Ok(HttpResponse::Ok().finish()),
            Err(err) => Err(APIError::with(Unexpected).note(err.to_string())),
        }
    } else {
        Err(APIError::with(Unspecified).note("Already exists username!".to_owned()))
    }
}

#[get("/login")]
pub async fn login_user(
    query: web::Query<dto::LoginQuery>,
    user_system: UserSys,
) -> Result<Json<dto::LoginUser>, APIError> {
    match user_system.login(&query.username, &query.password).await {
        Ok(token) => {
            let user = user_system.get_user(&query.username).await;
            match user {
                Ok(user) => {
                    let current: dto::UserInfo = user.into();
                    Ok(Json(dto::LoginUser { current, token }))
                }
                Err(err) => Err(APIError::with(Unexpected).note(err.to_string())),
            }
        }
        Err(err) => Err(APIError::with(Unexpected).note(err.to_string())),
    }
}

#[get("/logout")]
pub async fn logout_user(
    query: web::Query<dto::LogoutQuery>,
    user_system: UserSys,
) -> Result<Json<bool>, APIError> {
    Ok(Json(user_system.logout(&query.token).await))
}

#[get("/current_user")]
pub async fn get_current_user(permission: UserPermission) -> Result<Json<dto::UserInfo>, APIError> {
    permission.get_owner().map(|user| Json(user))
}

#[get("/scan_libraries")]
pub async fn scan_libraries<'a>(
    permission: UserPermission,
    library_system: LibSys,
    plugin_system: PlgSys,
) -> Result<HttpResponse, APIError> {
    if !permission.is_admin() {
        Err(APIError::with(NoPermission).note("Only administrator can operate."))
    } else {
        let plgsys = plugin_system.read().await;
        library_system.write().await.scan(&plgsys).await;
        Ok(HttpResponse::Ok().finish())
    }
}

#[get("/info")]
pub async fn get_server_info(
    permission: UserPermission,
    start: web::Data<Instant>,
) -> Result<Json<dto::ServerInfo>, APIError> {
    if !permission.is_admin() {
        Err(APIError::with(NoPermission).note("Only administrator can visit."))
    } else {
        Ok(Json(dto::ServerInfo {
            author: "Jinker",
            version: env!("CARGO_PKG_VERSION"),
            time_running: start.elapsed().as_secs(),
        }))
    }
}

#[get("/scan_plugins")]
pub async fn scan_plugins(
    permission: UserPermission,
    plusys: PlgSys,
) -> Result<HttpResponse, APIError> {
    if !permission.is_admin() {
        Err(APIError::with(NoPermission).note("Only administrator can access."))
    } else {
        plusys.write().await.scan().await;
        Ok(HttpResponse::Ok().finish())
    }
}

#[get("/plugin")]
pub async fn get_plugin(
    permission: UserPermission,
    plusys: PlgSys,
    query: web::Query<dto::GetPLuginQuery>,
) -> Result<HttpResponse, APIError> {
    if !permission.is_admin() {
        Err(APIError::with(NoPermission).note("Only administrator can access."))
    } else {
        let r = plusys.read().await;
        let infos = r.get_plugins_info();
        let index = query.index * query.limit;
        let max = {
            let max = index + query.limit;
            if max > infos.len() {
                infos.len()
            } else {
                max
            }
        };

        Ok(HttpResponse::Ok().json(infos[index..max].to_vec()))
    }
}