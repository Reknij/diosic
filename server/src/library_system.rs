use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    sync::Arc,
};

use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use tracing::{info, warn};
use walkdir::WalkDir;

use crate::{
    config::Config,
    myutil::{self, DiosicID},
    plugin_system::{self},
};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct MediaInfo {
    pub id: DiosicID,
    pub path: PathBuf,
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

pub type ArcMediaInfo = Arc<MediaInfo>;

#[derive(Debug, PartialEq, Serialize, Deserialize, Eq, Hash, Clone)]
pub struct MediaLibrary {
    pub path: PathBuf,
    pub title: String,
}

#[derive(Debug, Clone)]
pub struct MediaSourceInfo {
    pub title: String,
    pub length: usize,
}

#[derive(Debug, Clone)]
pub struct LibrarySystem {
    medias_path: HashMap<DiosicID, PathBuf>,
    medias_info: HashMap<DiosicID, ArcMediaInfo>,

    lib_medias: HashMap<String, Vec<ArcMediaInfo>>,
    album_medias: HashMap<String, Vec<ArcMediaInfo>>,
    category_medias: HashMap<String, Vec<ArcMediaInfo>>,
    artist_medias: HashMap<String, Vec<ArcMediaInfo>>,
    genre_medias: HashMap<String, Vec<ArcMediaInfo>>,
    year_medias: HashMap<String, Vec<ArcMediaInfo>>,

    libraries_info: HashMap<String, MediaSourceInfo>,
    categories_info: HashMap<String, MediaSourceInfo>,
    albums_info: HashMap<String, MediaSourceInfo>,
    artists_info: HashMap<String, MediaSourceInfo>,
    genres_info: HashMap<String, MediaSourceInfo>,
    years_info: HashMap<String, MediaSourceInfo>,

    covers: HashMap<DiosicID, PathBuf>,

    last_access_config: Arc<Config>,
    last_search_medias: Arc<RwLock<Option<(String, Vec<ArcMediaInfo>)>>>,
}

impl MediaLibrary {
    pub async fn fetch(&self) -> Vec<PathBuf> {
        info!("fetching..");
        let mut files = Vec::new();
        for entry in WalkDir::new(&self.path)
            .follow_links(false)
            .into_iter()
            .filter_map(|e| {
                e.ok()
                    .and_then(|e| if e.path().is_file() { Some(e) } else { None })
            })
        {
            files.push(entry.into_path())
        }

        files
    }
}

fn get_categories_from_directory(path: &PathBuf, lib: &MediaLibrary) -> Vec<String> {
    let mut categories = Vec::new();
    if let Some(parent) = path.parent() {
        if parent != lib.path {
            if let Some(name) = parent.file_name() {
                categories.push(name.to_str().unwrap().to_string());
                let mut grandparent = get_categories_from_directory(&parent.to_path_buf(), &lib);
                categories.append(&mut grandparent);
            }
        }
    }

    categories
}

impl LibrarySystem {
    pub async fn new<'a>(
        config: Arc<Config>,
        plgsys: &plugin_system::PluginSystem,
    ) -> LibrarySystem {
        let mut libraries_info = HashMap::with_capacity(config.libraries.len());
        let mut albums_info = HashMap::new();
        let mut categories_info = HashMap::new();
        let mut artists_info = HashMap::new();
        let mut genres_info = HashMap::new();
        let mut years_info = HashMap::new();

        let mut medias_path = HashMap::new();
        let mut medias_info = HashMap::new();

        let mut lib_medias: HashMap<String, Vec<ArcMediaInfo>> =
            HashMap::with_capacity(config.libraries.len());
        let mut album_medias: HashMap<String, Vec<ArcMediaInfo>> = HashMap::new();
        let mut category_medias: HashMap<String, Vec<ArcMediaInfo>> = HashMap::new();
        let mut artist_medias: HashMap<String, Vec<ArcMediaInfo>> = HashMap::new();
        let mut genre_medias: HashMap<String, Vec<ArcMediaInfo>> = HashMap::new();
        let mut year_medias: HashMap<String, Vec<ArcMediaInfo>> = HashMap::new();

        let mut covers = HashMap::new();
        if plgsys.exists_plugins() {
            info!("Will process media info with plugins.");
        }
        for lib in &config.libraries {
            let paths = &lib.fetch().await;
            let mut medias = Vec::with_capacity(paths.len());
            for m in paths {
                let id: DiosicID = myutil::calc_hash(&m.to_str().unwrap()).into();

                let meta = crate::metadata::read_from_path(m, &config).await;

                let cover = get_image_path_media(&m);
                let mut categories = get_categories_from_directory(&m, &lib);

                let info = {
                    let info = MediaInfo {
                        id: id.clone(),
                        path: m.clone(),
                        title: meta.title.clone(),
                        library: lib.title.clone(),
                        album: meta.album.clone(),
                        artist: meta.artist.clone(),
                        genre: meta.genre.clone(),
                        year: meta.year.clone(),
                        simple_rate: meta.simple_rate,
                        bit_depth: meta.bit_depth,
                        audio_bitrate: meta.audio_bitrate,
                        overall_bitrate: meta.overall_bitrate,
                        channels: meta.channels,
                        duration_milliseconds: meta.duration_milliseconds,
                        cover: match cover {
                            Some(path) => {
                                covers.insert(id.clone(), path);
                                Some(format!("/api/media_cover/{}", id.as_str()))
                            }
                            None => {
                                if let Some(path) = meta.cover {
                                    covers.insert(id.clone(), path);
                                    Some(format!("/api/media_cover/{}", id.as_str()))
                                } else {
                                    None
                                }
                            }
                        },
                        categories: categories.clone(),
                    };
                    if !plgsys.exists_plugins() {
                        ArcMediaInfo::new(info)
                    } else {
                        let mut json = serde_json::to_string(&info).unwrap();
                        plgsys
                            .process_media_info_json(m.to_str().unwrap(), &mut json)
                            .await;
                        match serde_json::from_str::<MediaInfo>(&json) {
                            Ok(v) => {
                                categories = v.categories.clone();
                                ArcMediaInfo::new(v)
                            }
                            Err(err) => {
                                let c = match err.classify() {
                                    serde_json::error::Category::Io => "IO",
                                    serde_json::error::Category::Syntax => "Syntax",
                                    serde_json::error::Category::Data => "Data",
                                    serde_json::error::Category::Eof => "EOF",
                                };
                                warn!(
                                    "failed to parse media info json with {} error: {}\n{}",
                                    c, err, &json
                                );
                                ArcMediaInfo::new(info)
                            }
                        }
                    }
                };

                for category in &categories {
                    let categories = category_medias.get_mut(category);
                    match categories {
                        Some(list) => list.push(info.clone()),
                        None => {
                            category_medias.insert(category.to_owned(), vec![info.clone()]);
                        }
                    }
                }

                medias.push(info.clone());
                medias_path.insert(id.clone(), m.to_owned());

                // process albums
                let albums = album_medias.get_mut(&meta.album);
                match albums {
                    Some(list) => list.push(info.clone()),
                    None => {
                        album_medias.insert(meta.album.to_owned(), vec![info.clone()]);
                    }
                }

                // process artists
                let artists = artist_medias.get_mut(&meta.artist);
                match artists {
                    Some(list) => list.push(info.clone()),
                    None => {
                        artist_medias.insert(meta.artist.to_owned(), vec![info.clone()]);
                    }
                }

                // process genres
                let genres = genre_medias.get_mut(&meta.genre);
                match genres {
                    Some(list) => list.push(info.clone()),
                    None => {
                        genre_medias.insert(meta.genre.to_owned(), vec![info.clone()]);
                    }
                }

                // process years
                let years = year_medias.get_mut(&meta.year);
                match years {
                    Some(list) => list.push(info.clone()),
                    None => {
                        year_medias.insert(meta.year.to_owned(), vec![info.clone()]);
                    }
                }

                medias_info.insert(id.clone(), info);
            }
            lib_medias.insert(lib.title.to_owned(), medias);
        }

        for (title, content) in &lib_medias {
            //process libraries information
            libraries_info.insert(
                title.clone(),
                MediaSourceInfo {
                    title: title.to_owned(),
                    length: content.len(),
                },
            );
        }

        for (title, content) in &album_medias {
            //process albums information
            albums_info.insert(
                title.clone(),
                MediaSourceInfo {
                    title: title.to_owned(),
                    length: content.len(),
                },
            );
        }

        for (title, content) in &category_medias {
            //process categories information
            categories_info.insert(
                title.clone(),
                MediaSourceInfo {
                    title: title.to_owned(),
                    length: content.len(),
                },
            );
        }

        for (title, content) in &artist_medias {
            //process categories information
            artists_info.insert(
                title.clone(),
                MediaSourceInfo {
                    title: title.to_owned(),
                    length: content.len(),
                },
            );
        }

        for (title, content) in &genre_medias {
            //process categories information
            genres_info.insert(
                title.clone(),
                MediaSourceInfo {
                    title: title.to_owned(),
                    length: content.len(),
                },
            );
        }

        for (title, content) in &year_medias {
            //process categories information
            years_info.insert(
                title.clone(),
                MediaSourceInfo {
                    title: title.to_owned(),
                    length: content.len(),
                },
            );
        }

        LibrarySystem {
            medias_path,
            covers,

            lib_medias,
            album_medias,
            category_medias,
            artist_medias,
            genre_medias,
            year_medias,

            medias_info,
            libraries_info,
            albums_info,
            categories_info,
            artists_info,
            genres_info,
            years_info,

            last_access_config: config.clone(),
            last_search_medias: Arc::new(RwLock::new(None)),
        }
    }

    pub async fn scan(&mut self, plgsys: &plugin_system::PluginSystem) {
        info!("Scanning all library..");
        let newed = LibrarySystem::new(self.last_access_config.clone(), plgsys).await;
        *self = newed;
    }

    pub async fn get_libraries(&self) -> Vec<MediaSourceInfo> {
        self.libraries_info.values().map(|v| v.clone()).collect()
    }

    pub async fn get_albums(&self) -> Vec<MediaSourceInfo> {
        self.albums_info.values().map(|v| v.clone()).collect()
    }

    pub async fn get_categories(&self) -> Vec<MediaSourceInfo> {
        self.categories_info.values().map(|v| v.clone()).collect()
    }

    pub async fn get_artists(&self) -> Vec<MediaSourceInfo> {
        self.artists_info.values().map(|v| v.clone()).collect()
    }

    pub async fn get_genres(&self) -> Vec<MediaSourceInfo> {
        self.genres_info.values().map(|v| v.clone()).collect()
    }

    pub async fn get_years(&self) -> Vec<MediaSourceInfo> {
        self.years_info.values().map(|v| v.clone()).collect()
    }

    pub async fn get_library(&self, title: &str) -> Option<MediaSourceInfo> {
        self.libraries_info.get(title).map(|o| o.clone())
    }

    pub async fn get_album(&self, title: &str) -> Option<MediaSourceInfo> {
        self.albums_info.get(title).map(|o| o.clone())
    }

    pub async fn get_category(&self, title: &str) -> Option<MediaSourceInfo> {
        self.categories_info.get(title).map(|o| o.clone())
    }

    pub async fn get_artist(&self, title: &str) -> Option<MediaSourceInfo> {
        self.artists_info.get(title).map(|o| o.clone())
    }

    pub async fn get_genre(&self, title: &str) -> Option<MediaSourceInfo> {
        self.genres_info.get(title).map(|o| o.clone())
    }

    pub async fn get_year(&self, title: &str) -> Option<MediaSourceInfo> {
        self.years_info.get(title).map(|o| o.clone())
    }

    pub async fn get_medias_by_library(&self, title: &str) -> Option<&Vec<ArcMediaInfo>> {
        let values = self.lib_medias.get(title)?;
        Some(values)
    }

    pub async fn get_medias_by_album(&self, title: &str) -> Option<&Vec<ArcMediaInfo>> {
        let values = self.album_medias.get(title)?;
        Some(values)
    }

    pub async fn get_medias_by_category(&self, title: &str) -> Option<&Vec<ArcMediaInfo>> {
        let values = self.category_medias.get(title)?;
        Some(values)
    }

    pub async fn get_medias_by_artist(&self, title: &str) -> Option<&Vec<ArcMediaInfo>> {
        let values = self.artist_medias.get(title)?;
        Some(values)
    }

    pub async fn get_medias_by_genre(&self, title: &str) -> Option<&Vec<ArcMediaInfo>> {
        let values = self.genre_medias.get(title)?;
        Some(values)
    }

    pub async fn get_medias_by_year(&self, title: &str) -> Option<&Vec<ArcMediaInfo>> {
        let values = self.year_medias.get(title)?;
        Some(values)
    }

    pub async fn get_media_file_by_id(&self, id: &DiosicID) -> Option<PathBuf> {
        match self.medias_path.get(&id) {
            Some(m) => Some(m.clone()),
            None => None,
        }
    }

    pub async fn get_media_cover_by_id(&self, id: &DiosicID) -> Option<&PathBuf> {
        self.covers.get(&id)
    }

    pub async fn get_media_info_by_id(&self, id: &DiosicID) -> Option<ArcMediaInfo> {
        if let Some(m) = self.medias_info.get(&id) {
            Some(m.clone())
        } else {
            None
        }
    }

    pub async fn get_medias_by_search(
        &self,
        search: &str,
        source: &str,
        filter: &str,
        is_filter: bool,
        index: usize,
        limit: usize,
    ) -> Vec<ArcMediaInfo> {
        let empty: Vec<ArcMediaInfo> = Vec::new();

        let all_media: Vec<ArcMediaInfo>;
        let medias: &Vec<ArcMediaInfo> = if is_filter {
            self.get_medias_by_source_with_filter(source, filter)
                .await
                .unwrap_or(&empty)
        } else {
            all_media = self.medias_info.values().map(|m| m.clone()).collect();
            &all_media
        };
        self.search_medias_core(medias, search, index, limit).await
    }

    async fn search_medias_core(
        &self,
        medias: &Vec<ArcMediaInfo>,
        search: &str,
        index: usize,
        limit: usize,
    ) -> Vec<ArcMediaInfo> {
        fn get_result(index: usize, limit: usize, result: &Vec<ArcMediaInfo>) -> Vec<ArcMediaInfo> {
            let max = {
                let m = index + limit;
                if m > result.len() {
                    result.len()
                } else {
                    m
                }
            };
            result[index..max].to_vec()
        }
        let index = index * limit;
        {
            let last = self.last_search_medias.read().await;
            if let Some((last_search, medias)) = last.as_ref() {
                if search == last_search {
                    return get_result(index, limit, medias);
                }
            }
        }

        let mut arr = Vec::with_capacity(30);
        for m in medias {
            if m.contains(search) {
                arr.push(m.clone());
            }
        }
        let result = get_result(index, limit, &arr);
        self.last_search_medias
            .write()
            .await
            .replace((search.to_owned(), arr));
        result
    }

    pub async fn get_medias_by_source_with_filter(
        &self,
        source: &str,
        filter: &str,
    ) -> Option<&Vec<ArcMediaInfo>> {
        match source {
            "library" => self.get_medias_by_library(filter).await,
            "album" => self.get_medias_by_album(filter).await,
            "category" => self.get_medias_by_category(filter).await,
            "artist" => self.get_medias_by_artist(filter).await,
            "genre" => self.get_medias_by_genre(filter).await,
            "year" => self.get_medias_by_year(filter).await,
            _ => return None,
        }
    }
}

impl MediaInfo {
    pub fn contains(&self, content: &str) -> bool {
        let content = content.to_lowercase();
        let title = self.title.to_lowercase();
        let album = self.album.to_lowercase();
        let artist = self.artist.to_lowercase();
        let year = self.year.to_lowercase();
        let genre = self.genre.to_lowercase();
        let library = self.library.to_lowercase();
        let categories: Vec<String> = self.categories.iter().map(|c| c.to_lowercase()).collect();
        if title.contains(&content)
            || album.contains(&content)
            || artist.contains(&content)
            || year.contains(&content)
            || genre.contains(&content)
            || library.contains(&content)
            || categories.contains(&content)
        {
            true
        } else {
            false
        }
    }
}

fn get_image_path_media(path: &PathBuf) -> Option<PathBuf> {
    let file_name = myutil::get_file_name_without_ext(path);
    let parent = path.parent().unwrap();
    let png = Path::new(parent).join(format!("{}.png", file_name));
    let jpg = Path::new(parent).join(format!("{}.jpg", file_name));
    let cover = if png.exists() {
        png
    } else if jpg.exists() {
        jpg
    } else {
        jpg
    };
    if cover.is_file() {
        return Some(cover);
    } else {
        None
    }
}
