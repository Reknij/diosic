use std::path::{Path, PathBuf};

use lofty::{Accessor, MimeType, AudioFile, TaggedFileExt};
use serde::{Serialize, Deserialize};
use tokio::fs;
use tracing::info;

use crate::{config::Config, myutil};

#[derive(Serialize, Deserialize)]
pub struct Metadata {
    pub title: String,
    pub album: String,
    pub artist: String,
    pub genre: String,
    pub year: String,
    pub cover: Option<PathBuf>,
    pub simple_rate: Option<u32>,
    pub bit_depth: Option<u8>,
    pub audio_bitrate: Option<u32>,
    pub overall_bitrate: Option<u32>,
    pub channels: Option<u8>,
    pub duration_milliseconds: u128,
}

pub async fn read_from_path<P>(path: P, config: &Config) -> Metadata
where
    P: AsRef<Path>,
{
    let file_name_without_ext = myutil::get_file_name_without_ext(&path.as_ref().to_path_buf());
    let mut meta = Metadata {
        title: file_name_without_ext.to_owned(),
        album: "Unknown".to_owned(),
        artist: "Unknown".to_owned(),
        genre: "Unknown".to_owned(),
        year: "Unknown".to_owned(),
        cover: None,
        simple_rate: None,
        bit_depth: None,
        audio_bitrate: None,
        overall_bitrate: None,
        channels: None,
        duration_milliseconds: 0
    };

    let tagged_file = lofty::read_from_path(&path);
    match tagged_file {
        Ok(tagged_file) => {
            let id3v2 = tagged_file.primary_tag();
            let properties = tagged_file.properties();

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
                    meta.year = year.to_string();
                }
                let pics = id3.pictures();
                if pics.len() > 0 {
                    let pic = &pics[0];
                    if let Some(data_path) = &config.data_path {
                        if data_path.is_dir() {
                            let cache_cover_directory = data_path.join("cache_covers");
                            if !cache_cover_directory.exists() {
                                fs::create_dir(&cache_cover_directory).await.unwrap();
                            }
                            let pic_name = match pic.mime_type() {
                                MimeType::Bmp => Some(format!("{}.bmp", file_name_without_ext)),
                                MimeType::Gif => Some(format!("{}.gif", file_name_without_ext)),
                                MimeType::Jpeg => {
                                    Some(format!("{}.jpg", file_name_without_ext))
                                }
                                MimeType::Png => Some(format!("{}.png", file_name_without_ext)),
                                MimeType::Tiff => {
                                    Some(format!("{}.tiff", file_name_without_ext))
                                }
                                _ => None,
                            };
                            if let Some(pic_name) = pic_name {
                                let save_path = cache_cover_directory.join(pic_name);
                                fs::write(&save_path, pic.data()).await.unwrap();
                                meta.cover = Some(save_path);
                            }
                        } else {
                            info!("`data_path` is empty so can't save media cover.")
                        }
                    }
                }
                meta.audio_bitrate = properties.audio_bitrate();
                meta.overall_bitrate = properties.overall_bitrate();
                meta.simple_rate = properties.sample_rate();
                meta.bit_depth = properties.bit_depth();
                meta.channels = properties.channels();
                meta.duration_milliseconds = properties.duration().as_millis();
            }
        }
        Err(err) => {
            let p = path.as_ref().to_str().unwrap_or("Failed path");
            tracing::error!("{:?} from reading `{}`", err.to_string(), p);
        }
    }
    // Get the primary tag (ID3v2 in this case)

    meta
}
