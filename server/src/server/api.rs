use actix_files::NamedFile;
use actix_web::{
    delete, get, post, put,
    web::{self, Json},
    HttpResponse,
};
use tokio::time::Instant;
use tracing::{error, warn};

use crate::{
    library_system::model::SourceInfo,
    server::dto::{ListSlice, PubMediaInfo},
    user_system::model::{UserInfo, UserToCreate},
};

use super::error::APIErrorType::*;
use super::error::*;
use super::from_requests::*;
use super::{dto, AppState};

type State = web::Data<AppState>;

#[get("/server_info")]
pub async fn get_server_info(state: State, start: web::Data<Instant>) -> Json<dto::ServerInfo> {
    Json(dto::ServerInfo {
        admin_required: !state.user_system.contains_users().await,
        guest_enable: state.user_system.is_guest_enabled().await,
        guest_password_required: state.user_system.is_guest_password_required().await,
        author: "Jinker",
        version: env!("CARGO_PKG_VERSION"),
        time_running: start.elapsed().as_secs(),
    })
}

#[put("/setup")]
pub async fn setup(
    state: State,
    setup: Json<dto::ToSetup>,
    permission: UserPermission,
) -> Result<HttpResponse, APIError> {
    let setup = setup.0;
    let setup_fn = || async {
        if setup.guest_enable {
            if let Some(pass) = &setup.guest_password {
                if pass.len() < 8 {
                    return Err(
                        APIError::with(Unexpected).note("Password length must more than 8.")
                    );
                }
            }
            if state.user_system.exists_user("guest").await {
                let old_guest = state.user_system.get_user("guest").await;
                state
                    .user_system
                    .update_user(
                        &old_guest,
                        UserToCreate {
                            alias: old_guest.alias.clone(),
                            username: old_guest.username.clone(),
                            password: setup.guest_password.clone().unwrap_or("".to_owned()),
                        },
                    )
                    .await;
            } else {
                state
                    .user_system
                    .create_user(
                        UserToCreate {
                            alias: "Guest".to_owned(),
                            username: "guest".to_owned(),
                            password: setup.guest_password.unwrap_or("".to_owned()),
                        },
                        true,
                    )
                    .await
                    .map_err(|err| {
                        APIError::with(Unexpected)
                            .note(format!("Create guest error: {}", err.to_string()))
                    })?;
            }
        }
        state
            .user_system
            .create_user(
                UserToCreate {
                    alias: setup.alias,
                    username: setup.username,
                    password: setup.password,
                },
                true,
            )
            .await
            .map_err(|err| {
                APIError::with(Unexpected).note(format!("Create user error: {}", err.to_string()))
            })?;
        Ok(HttpResponse::Ok().finish())
    };
    if state.user_system.contains_users().await {
        if permission.is_admin() {
            match state
                .user_system
                .delete_user(&permission.get_owner().unwrap())
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
    state: State,
    info: web::Path<(i64,)>,
    permission: UserPermission,
) -> Result<NamedFile, APIError> {
    if !permission.exists_owner() {
        return Err(APIError::with(NoPermission).note("Please log in first!"));
    }
    match state.library_system.get_media_file_by_id(info.0).await {
        Some(media) => NamedFile::open_async(&media)
            .await
            .map_err(|err| APIError::with(Unexpected).note(err.to_string())),
        None => Err(APIError::with(NoFound).note("Can't get target media by hash id.")),
    }
}

#[get("/media_cover/{id}")]
pub async fn get_media_cover(
    state: State,
    info: web::Path<(i64,)>,
    permission: UserPermission,
) -> Result<NamedFile, APIError> {
    if !permission.exists_owner() {
        return Err(APIError::with(NoPermission).note("Please log in first!"));
    }

    match state
        .library_system
        .get_media_cover_file_by_id(info.0)
        .await
    {
        Some(img) => {
            if img.is_file() {
                NamedFile::open_async(&img)
                    .await
                    .map_err(|err| APIError::with(Unexpected).note(err.to_string()))
            } else {
                warn!("The media cover path `{:?}` is not exists!", img);
                Err(APIError::with(NoFound).note("The media cover is not exists."))
            }
        }
        None => Err(APIError::with(NoFound).note("Can't get target media cover.")),
    }
}

#[get("/media_info/{id}")]
pub async fn get_media_info(
    state: State,
    info: web::Path<(i64,)>,
    permission: UserPermission,
) -> Result<Json<PubMediaInfo>, APIError> {
    if !permission.exists_owner() {
        return Err(APIError::with(NoPermission).note("Please log in first!"));
    }
    match state.library_system.get_media_info_by_id(info.0).await {
        Some(info) => Ok(Json(info.into())),
        None => Err(APIError::with(NoFound).note("No found media with id!")),
    }
}

#[get("/medias")]
pub async fn get_medias(
    state: State,
    query: web::Query<dto::GetMediasQuery>,
    permission: UserPermission,
) -> Result<Json<dto::ListSlice<PubMediaInfo>>, APIError> {
    use crate::library_system::model::Source;
    if !permission.exists_owner() {
        return Err(APIError::with(NoPermission).note("Please log in first!"));
    }
    let source = Source::parse(&query.source, query.filter.as_ref().map(|f| f.as_str()));
    let to_search = query.to_search.as_ref().map(|s| s.as_str());
    let medias = state
        .library_system
        .get_medias(source, to_search, query.index, query.limit)
        .await;

    let total = state
        .library_system
        .get_total_media(source, to_search)
        .await;
    Ok(Json(dto::ListSlice {
        items: medias.into_iter().map(|v| v.into()).collect(),
        total,
    }))
}

#[get("/sources")]
pub async fn get_sources(
    state: State,
    query: web::Query<dto::GetSourcesQuery>,
    permission: UserPermission,
) -> Result<Json<ListSlice<SourceInfo>>, APIError> {
    use crate::library_system::model::Source;
    let source = Source::parse(&query.source, query.filter.as_ref().map(|f| f.as_str()));
    if !permission.exists_owner() {
        return Err(APIError::with(NoPermission).note("Please log in first!"));
    }
    let items = state
        .library_system
        .get_sources(source, query.index, query.limit)
        .await;
    let total = state.library_system.get_total_source(source).await;
    Ok(Json(ListSlice { items, total }))
}

#[get("/users/{username}")]
pub async fn get_user(
    state: State,
    info: web::Path<(String,)>,
    permission: UserPermission,
) -> Result<Json<UserInfo>, APIError> {
    if !permission.exists_owner() {
        return Err(APIError::with(NoPermission).note("Please log in first!"));
    }
    if !permission.is_admin() {
        return Err(APIError::with(NoPermission).note("Only admin can access."));
    }

    if !state.user_system.exists_user(&info.0).await {
        return Err(APIError::with(NoFoundUser));
    }
    Ok(Json(state.user_system.get_user(&info.0).await))
}

#[get("/users")]
pub async fn get_users(
    state: State,
    permission: UserPermission,
    query: web::Query<dto::GetUsersQuery>,
) -> Result<Json<ListSlice<UserInfo>>, APIError> {
    if !permission.exists_owner() {
        return Err(APIError::with(NoPermission).note("Please log in first!"));
    }
    if !permission.is_admin() {
        return Err(APIError::with(NoPermission).note("Only admin can access."));
    }
    let to_search = query.to_search.as_ref().map(|s| s.as_str());
    let items = state
        .user_system
        .get_users(query.index, query.limit, to_search)
        .await;
    let total = state.user_system.get_total_user(to_search).await;
    Ok(Json(ListSlice { items, total }))
}

#[delete("/users/{username}")]
pub async fn delete_user(
    state: State,
    info: web::Path<(String,)>,
    permission: UserPermission,
) -> Result<HttpResponse, APIError> {
    if !permission.exists_owner() {
        return Err(APIError::with(NoPermission).note("Please log in first!"));
    }

    if !state.user_system.exists_user(&info.0).await {
        return Err(APIError::with(NoFoundUser));
    }

    let user = state.user_system.get_user(&info.0).await;
    if user.is_admin {
        return Err(APIError::with(NoPermission).note("Can't delete admin!"));
    }

    if permission.have_permission_with(&info.0) {
        match state.user_system.delete_user(&user).await {
            Ok(_) => Ok(HttpResponse::Accepted().finish()),
            Err(err) => Err(APIError::with(Unexpected).note(err.to_string())),
        }
    } else {
        Err(APIError::with(NoPermission).note("No have permission!".to_owned()))
    }
}

#[put("/users")]
pub async fn update_user(
    state: State,
    permission: UserPermission,
    to_update: Json<UserToCreate>,
) -> Result<HttpResponse, APIError> {
    if !permission.exists_owner() {
        return Err(APIError::with(NoPermission).note("Please log in first!"));
    }

    if permission.is_guest() {
        return Err(APIError::with(NoPermission).note("Guest can't update information."));
    }

    let user = permission.get_owner()?;
    if state.user_system.exists_user(&user.username).await {
        if permission.have_permission_with(&user.username) {
            if state
                .user_system
                .update_user(&user.into(), to_update.0)
                .await
            {
                Ok(HttpResponse::Ok().finish())
            } else {
                Err(APIError::with(Unspecified).note("No found user to update."))
            }
        } else {
            Err(APIError::with(NoPermission).note("No have permission!".to_owned()))
        }
    } else {
        Err(APIError::with(NoFoundUser).note("No exists username!".to_owned()))
    }
}

#[post("/users")]
pub async fn create_user(
    state: State,
    permission: UserPermission,
    to_create: Json<UserToCreate>,
) -> Result<HttpResponse, APIError> {
    if !permission.exists_owner() {
        return Err(APIError::with(NoFoundUser).note("Please login first."));
    }

    if !permission.is_admin() {
        return Err(APIError::with(NoPermission).note("Only admin can create user."));
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

    if !state.user_system.exists_user(&to_create.username).await {
        match state.user_system.create_user(to_create.0, false).await {
            Ok(()) => Ok(HttpResponse::Ok().finish()),
            Err(err) => Err(APIError::with(Unexpected).note(err.to_string())),
        }
    } else {
        Err(APIError::with(Unspecified).note("Already exists username!".to_owned()))
    }
}

#[get("/login")]
pub async fn login_user(
    state: State,
    query: web::Query<dto::LoginQuery>,
) -> Result<Json<dto::LoginedResult>, APIError> {
    match state
        .user_system
        .login(&query.username, &query.password)
        .await
    {
        Some(token) => {
            let current = state.user_system.get_user(&query.username).await;
            Ok(Json(dto::LoginedResult { current, token }))
        }
        None => {
            Err(APIError::with(NoFoundUser)
                .note("Please make sure username and password is correct!"))
        }
    }
}

#[put("/logout")]
pub async fn logout_user(
    state: State,
    query: web::Query<dto::LogoutQuery>,
) -> Result<Json<bool>, APIError> {
    Ok(Json(state.user_system.logout(&query.token).await))
}

#[get("/current_user")]
pub async fn get_current_user(permission: UserPermission) -> Result<Json<UserInfo>, APIError> {
    permission.get_owner().map(|user| Json(user))
}

#[put("/actions/reload_medias")]
pub async fn reload_medias(
    state: State,
    permission: UserPermission,
) -> Result<HttpResponse, APIError> {
    if !permission.is_admin() {
        Err(APIError::with(NoPermission).note("Only administrator can operate."))
    } else {
        state.library_system.reload(&state.plugin_system).await;
        Ok(HttpResponse::Ok().finish())
    }
}

#[put("/actions/reload_plugins")]
pub async fn reload_plugins(
    state: State,
    permission: UserPermission,
) -> Result<HttpResponse, APIError> {
    if !permission.is_admin() {
        Err(APIError::with(NoPermission).note("Only administrator can operate."))
    } else {
        state.plugin_system.reload().await;
        Ok(HttpResponse::Ok().finish())
    }
}
