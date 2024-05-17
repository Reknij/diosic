use serde::{Deserialize, Serialize};

use crate::{library_system::model::MediaInfo, user_system::model::UserInfo};

#[derive(Debug, Serialize, Clone)]
pub struct ServerInfo {
    pub version: &'static str,
    pub author: &'static str,
    pub time_running: u64,
    pub admin_required: bool,
    pub guest_enable: bool,
    pub guest_password_required: bool,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct PubMediaInfo {
    pub id: i64,
    pub title: String,
    pub album: String,
    pub artist: String,
    pub genre: String,
    pub year: u32,
    pub library: String,
    pub cover_url: Option<String>,
    pub categories: Vec<String>,
    pub sample_rate: Option<u32>,
    pub bit_depth: Option<u8>,
    pub audio_bitrate: Option<u32>,
    pub overall_bitrate: Option<u32>,
    pub channels: Option<u8>,
    pub duration_seconds: u32,
    pub file_name: String,
    pub file_type: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ListSlice<T> {
    pub items: Vec<T>,
    pub total: usize,
}

#[derive(Debug, Deserialize)]
pub struct GetSourcesQuery {
    pub limit: usize,
    pub index: usize,
    pub source: String,
    pub filter: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct GetMediasQuery {
    pub limit: usize,
    pub index: usize,
    pub source: String,
    pub filter: Option<String>,
    pub to_search: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct GetUsersQuery {
    pub limit: usize,
    pub index: usize,
    pub to_search: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ToSetup {
    pub alias: String,
    pub username: String,
    pub password: String,
    pub guest_enable: bool,
    pub guest_password: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct LoginQuery {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LoginedResult {
    pub current: UserInfo,
    pub token: String,
}

#[derive(Debug, Deserialize)]
pub struct LogoutQuery {
    pub token: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SearchUserQuery {
    pub content: String,
    pub index: usize,
    pub limit: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AuthQuery {
    pub auth: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GetPLuginQuery {
    pub index: usize,
    pub limit: usize,
}

impl From<MediaInfo> for PubMediaInfo {
    fn from(value: MediaInfo) -> Self {
        let id = value.id;
        Self {
            id,
            title: value.title.clone(),
            album: value.album,
            artist: value.artist,
            genre: value.genre,
            year: value.year,
            library: value.library,
            cover_url: value.cover_url,
            categories: value.categories,
            sample_rate: value.sample_rate,
            bit_depth: value.bit_depth,
            audio_bitrate: value.audio_bitrate,
            overall_bitrate: value.overall_bitrate,
            channels: value.channels,
            duration_seconds: value.duration_seconds,
            file_name: value.file_name,
            file_type: value.file_type,
        }
    }
}
