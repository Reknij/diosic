use std::path::{Path, PathBuf};

use lofty::{Accessor, MimeType};
use tokio::fs;
use tracing::info;

use crate::{config::Config, myutil};

pub struct Metadata {
    pub title: String,
    pub album: String,
    pub artist: String,
    pub genre: String,
    pub year: String,
    pub cover: Option<PathBuf>,
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
    };

    let tagged_file = lofty::read_from_path(&path, false);
    match tagged_file {
        Ok(tagged_file) => {
            let id3v2 = tagged_file.primary_tag();

            match id3v2 {
                Some(id3) => {
                    if let Some(title) = id3.title() {
                        meta.title = title.to_owned();
                    }
                    if let Some(artist) = id3.artist() {
                        meta.artist = artist.to_owned();
                    }
                    if let Some(album) = id3.album() {
                        meta.album = album.to_owned();
                    }
                    if let Some(genre) = id3.genre() {
                        meta.genre = genre.to_owned();
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
                }
                None => {}
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
