use std::{
    path::{Path, PathBuf},
    sync::Arc,
};

use crate::{config::Config, myutil};
use anyhow::{Context, Result};
use lofty::{
    file::FileType,
    picture::{MimeType, Picture, PictureType},
    prelude::{Accessor, AudioFile, TaggedFileExt},
};
use serde::{Deserialize, Serialize};
use tokio::{fs, io::AsyncWriteExt};
use tracing::{info, warn};
use walkdir::WalkDir;

#[derive(Debug, PartialEq, Serialize, Deserialize, Eq, Hash, Clone)]
pub struct LibraryInfo {
    pub path: PathBuf,
    pub title: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct SourceInfo {
    pub title: String,
    pub total_media: u32,
}

impl LibraryInfo {
    pub async fn fetch(&self) -> Vec<PathBuf> {
        info!("Library `{}` with path `{:?}` fetching..", self.title, self.path);
        let mut files = Vec::new();
        for entry in WalkDir::new(&self.path)
            .follow_links(false)
            .into_iter()
            .filter_map(|e| e.ok().and_then(|e| if e.path().is_file() { Some(e) } else { None }))
        {
            files.push(entry.into_path())
        }

        info!("Fetched {} medias.", files.len());

        files
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Source<'a> {
    Any,
    Library(&'a str),
    Category(&'a str),
    Album(&'a str),
    Artist(&'a str),
    Genre(&'a str),
    Year(u32),
}

impl<'a> Source<'a> {
    pub fn parse(source: &'a str, filter: Option<&'a str>) -> Self {
        match source.to_lowercase().as_str() {
            "library" if filter.is_some() => Self::Library(filter.as_ref().unwrap()),
            "category" if filter.is_some() => Self::Category(filter.as_ref().unwrap()),
            "album" if filter.is_some() => Self::Album(filter.as_ref().unwrap()),
            "artist" if filter.is_some() => Self::Artist(filter.as_ref().unwrap()),
            "genre" if filter.is_some() => Self::Genre(filter.as_ref().unwrap()),
            "year" if filter.is_some() => Self::Year(filter.map(|f| f.parse().unwrap_or(0)).unwrap_or(0)),
            _ => Self::Any,
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct MediaMetaInfo {
    pub title: String,
    pub album: String,
    pub artist: String,
    pub genre: String,
    pub year: u32,
    pub cover: Option<PathBuf>,
    pub sample_rate: Option<u32>,
    pub bit_depth: Option<u8>,
    pub audio_bitrate: Option<u32>,
    pub overall_bitrate: Option<u32>,
    pub channels: Option<u8>,
    pub duration_seconds: u64,
    pub file_name: String,
    pub file_type: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct MediaInfo {
    pub id: i64,
    pub path: PathBuf,
    pub title: String,
    pub album: String,
    pub artist: String,
    pub genre: String,
    pub year: u32,
    pub library: String,
    pub cover_path: Option<PathBuf>,
    pub cover_url: Option<String>,
    pub sample_rate: Option<u32>,
    pub bit_depth: Option<u8>,
    pub audio_bitrate: Option<u32>,
    pub overall_bitrate: Option<u32>,
    pub channels: Option<u8>,
    pub duration_seconds: u32,
    pub categories: Vec<String>,
    pub file_name: String,
    pub file_type: String,
}

impl MediaInfo {
    pub fn from_meta(meta: MediaMetaInfo, config: Arc<Config>, library: &LibraryInfo, id: i64, path: PathBuf) -> Self {
        let public_url = config.public_url.as_str();
        let categories = Self::get_categories_from_directory(&path, &library);
        Self {
            id,
            path,
            title: meta.title,
            album: meta.album,
            artist: meta.artist,
            genre: meta.genre,
            year: meta.year,
            library: library.title.to_owned(),
            cover_path: meta.cover.clone().and_then(|p| if p.to_str().is_some() { Some(p) } else { None }),
            cover_url: meta.cover.and_then(|p| {
                if p.is_file() && p.to_str().is_some() {
                    Some(format!("{public_url}/api/media_cover/{id}"))
                } else {
                    None
                }
            }),
            sample_rate: meta.sample_rate,
            bit_depth: meta.bit_depth,
            audio_bitrate: meta.audio_bitrate,
            overall_bitrate: meta.overall_bitrate,
            channels: meta.channels,
            duration_seconds: meta.duration_seconds as u32,
            categories,
            file_name: meta.file_name,
            file_type: meta.file_type,
        }
    }

    fn get_categories_from_directory(path: &PathBuf, lib: &LibraryInfo) -> Vec<String> {
        let mut categories = Vec::new();
        if let Some(parent) = path.parent() {
            if parent != lib.path {
                if let Some(name) = parent.file_name() {
                    categories.push(name.to_str().unwrap().to_string());
                    let mut grandparent = Self::get_categories_from_directory(&parent.to_path_buf(), &lib);
                    categories.append(&mut grandparent);
                }
            }
        }

        categories
    }
}

impl MediaMetaInfo {
    fn cover_picture_level(pic: PictureType) -> usize {
        let mut pics = [PictureType::CoverFront, PictureType::LeadArtist, PictureType::Illustration, PictureType::Media, PictureType::CoverBack];
        pics.reverse();
        for (i, p) in pics.iter().enumerate() {
            if p == &pic {
                return i;
            }
        }

        0
    }

    pub async fn read_from_path<P>(path: P, config: &Config) -> Result<Self>
    where
        P: AsRef<Path>,
    {
        match lofty::read_from_path(&path) {
            Ok(tagged_file) => {
                let id3v2 = tagged_file.primary_tag();
                let properties = tagged_file.properties();
                let file_name_without_ext = myutil::get_file_name_without_ext(&path);
                let mut meta = MediaMetaInfo {
                    title: file_name_without_ext.to_owned(),
                    album: "Unknown".to_owned(),
                    artist: "Unknown".to_owned(),
                    genre: "Unknown".to_owned(),
                    year: 0,
                    cover: None,
                    sample_rate: properties.sample_rate(),
                    bit_depth: properties.bit_depth(),
                    audio_bitrate: properties.audio_bitrate(),
                    overall_bitrate: properties.overall_bitrate(),
                    channels: properties.channels(),
                    duration_seconds: properties.duration().as_secs(),
                    file_name: file_name_without_ext.to_owned(),
                    file_type: match tagged_file.file_type() {
                        FileType::Aac => "aac",
                        FileType::Aiff => "aiff",
                        FileType::Ape => "ape",
                        FileType::Flac => "flac",
                        FileType::Mpeg => "mpeg",
                        FileType::Mp4 => "mp4",
                        FileType::Mpc => "mpc",
                        FileType::Opus => "opus",
                        FileType::Vorbis => "vorbis",
                        FileType::Speex => "speex",
                        FileType::Wav => "wav",
                        FileType::WavPack => "wav_pack",
                        FileType::Custom(v) => v,
                        _ => "unknown",
                    }
                    .to_owned(),
                };

                if let Some(id3) = id3v2 {
                    if let Some(title) = id3.title() {
                        meta.title = title.to_string();
                    }
                    if let Some(artist) = id3.artist() {
                        meta.artist = artist.to_string();
                    }
                    if let Some(album) = id3.album() {
                        meta.album = album.to_string();
                    }
                    if let Some(genre) = id3.genre() {
                        meta.genre = genre.to_string();
                    }
                    if let Some(year) = id3.year() {
                        meta.year = year;
                    }
                    let pics = id3.pictures();
                    if pics.is_empty() {
                        return Ok(meta);
                    }
                    if let Some(cache_cover_directory) = &config.covers_cached_path {
                        let mut cover: Option<&Picture> = None;
                        for pic in pics {
                            let new_level = Self::cover_picture_level(pic.pic_type());
                            if new_level == 0 {
                                continue;
                            }
                            match cover {
                                Some(current) => {
                                    let current_level = Self::cover_picture_level(current.pic_type());
                                    if new_level > current_level {
                                        cover = Some(pic);
                                    }
                                }
                                None => {
                                    cover = Some(pic);
                                }
                            }
                        }
                        if let Some(cover) = cover {
                            if let Some(mime) = cover.mime_type() {
                                let cover_name = match mime {
                                    MimeType::Bmp => Some(format!("{}.bmp", file_name_without_ext)),
                                    MimeType::Gif => Some(format!("{}.gif", file_name_without_ext)),
                                    MimeType::Jpeg => Some(format!("{}.jpg", file_name_without_ext)),
                                    MimeType::Png => Some(format!("{}.png", file_name_without_ext)),
                                    MimeType::Tiff => Some(format!("{}.tiff", file_name_without_ext)),
                                    MimeType::Unknown(unk) => {
                                        warn!("Get the cover of unknown extension `{unk}` from media.");
                                        None
                                    }
                                    _ => None,
                                };
                                if let Some(cover_name) = cover_name {
                                    let save_path = cache_cover_directory.join(cover_name);
                                    let mut fs = fs::OpenOptions::new()
                                        .write(true)
                                        .create(true)
                                        .open(&save_path)
                                        .await
                                        .with_context(|| "Open to write the cover file failed!")?;
                                    fs.write(cover.data()).await.with_context(|| "Write the picture data into thee file failed!")?;
                                    meta.cover = Some(save_path);
                                }
                            }
                        }
                    } else {
                        info!("`data_path` is empty so can't save media cover.")
                    }
                }
                Ok(meta)
            }
            Err(err) => {
                let p = path.as_ref().to_str().unwrap_or("Failed path");
                tracing::error!("{:?} from reading `{}`", err.to_string(), p);
                anyhow::bail!(err)
            }
        }
    }
}
