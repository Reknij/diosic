use serde::{Deserialize, Serialize};

use crate::{
    library_system,
    myutil::{self, DiosicID},
    user_system,
};

#[derive(Debug, Serialize, Clone)]
pub struct ServerInfo {
    pub version: &'static str,
    pub author: &'static str,
    pub time_running: u64,
}

#[derive(Debug, Serialize, Clone)]
pub struct SetupInfo {
    pub admin_required: bool,
    pub guest_enable: bool,
    pub guest_password_required: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UserInfo {
    pub username: String,
    pub alias: String,
    pub password: String,
    pub is_admin: bool,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct MediaInfo {
    pub id: myutil::DiosicID,
    pub title: String,
    pub album: String,
    pub artist: String,
    pub genre: String,
    pub year: String,
    pub library: String,
    pub cover: Option<String>,
    pub categories: Vec<String>,
    pub simple_rate: Option<u32>,
    pub bit_depth: Option<u8>,
    pub audio_bitrate: Option<u32>,
    pub overall_bitrate: Option<u32>,
    pub channels: Option<u8>,
    pub duration_milliseconds: u128,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct MediaSourceInfo {
    pub title: String,
    pub length: usize,
}

#[derive(Debug, Deserialize)]
pub struct GetMediasQuery {
    pub limit: usize,
    pub index: usize,
    pub filter: String,
    pub source: String,
}

#[derive(Debug, Deserialize)]
pub struct GetUsersQuery {
    pub limit: usize,
    pub index: usize,
}

#[derive(Debug, Deserialize)]
pub struct LoginQuery {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Deserialize)]
pub struct LogoutQuery {
    pub token: myutil::DiosicID,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LoginUser {
    pub current: UserInfo,
    pub token: DiosicID,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ToSetup {
    pub admin: UserInfo,
    pub guest_enable: bool,
    pub guest_password: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GetLibraryQuery {
    pub title: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GetAlbumQuery {
    pub title: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GetCategoryQuery {
    pub title: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SearchMediaQuery {
    pub content: String,
    pub source: Option<String>,
    pub filter: Option<String>,
    pub index: usize,
    pub limit: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SearchResult<T> {
    pub content: Vec<T>,
    pub length: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SearchUserQuery {
    pub content: String,
    pub index: usize,
    pub limit: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AuthQuery {
    pub auth: DiosicID,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GetPLuginQuery {
    pub index: usize,
    pub limit: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AddPLuginQuery {
    pub address: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RemovePluginQuery {
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UndoAddPluginQuery {
    pub id: String,
}

impl ToSetup {
    pub fn process(&mut self) {
        self.admin.is_admin = true;
        if let Some(pass) = &self.guest_password {
            if pass.is_empty() {
                self.guest_password = None;
            }
        }
    }
}

impl Into<user_system::UserInfo> for LoginQuery {
    fn into(self) -> user_system::UserInfo {
        user_system::UserInfo {
            username: self.username.to_owned(),
            password: self.password,
            alias: self.username.to_owned(),
            is_admin: false,
        }
    }
}

impl From<library_system::MediaSourceInfo> for MediaSourceInfo {
    fn from(m: library_system::MediaSourceInfo) -> Self {
        Self {
            title: m.title,
            length: m.length,
        }
    }
}

impl From<user_system::UserInfo> for UserInfo {
    fn from(u: user_system::UserInfo) -> Self {
        Self {
            username: u.username,
            password: u.password,
            alias: u.alias,
            is_admin: u.is_admin,
        }
    }
}

impl Into<user_system::UserInfo> for UserInfo {
    fn into(self) -> user_system::UserInfo {
        user_system::UserInfo {
            username: self.username,
            password: self.password,
            alias: self.alias,
            is_admin: self.is_admin,
        }
    }
}

impl<T> From<T> for MediaInfo
where
    T: AsRef<library_system::MediaInfo>,
{
    fn from(m: T) -> Self {
        let m: &library_system::MediaInfo = m.as_ref();
        Self {
            id: m.id.clone(),
            title: m.title.clone(),
            album: m.album.clone(),
            artist: m.artist.clone(),
            genre: m.genre.clone(),
            year: m.year.clone(),
            library: m.library.clone(),
            cover: m.cover.clone(),
            categories: m.categories.clone(),
            simple_rate: m.simple_rate.clone(),
            bit_depth: m.bit_depth.clone(),
            audio_bitrate: m.audio_bitrate.clone(),
            overall_bitrate: m.overall_bitrate.clone(),
            channels: m.channels.clone(),
            duration_milliseconds: m.duration_milliseconds.clone(),
        }
    }
}
