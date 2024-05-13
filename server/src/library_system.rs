use std::{
    path::{Path, PathBuf},
    sync::Arc,
    time,
};

use futures_util::future::join_all;
use sqlx::{sqlite::SqliteRow, Pool, QueryBuilder, Row, Sqlite};
use tracing::{error, info};

pub mod model;

use crate::{
    config::Config,
    myutil::{self},
    plugin_system::{self},
};

use self::model::{LibraryInfo, MediaInfo, MediaMetaInfo, Source, SourceInfo};

const SQLITE_LIMIT: usize = 999;

#[derive(Debug, Clone)]
pub struct LibrarySystem {
    db: Pool<Sqlite>,
    config: Arc<Config>,
}

impl LibrarySystem {
    pub async fn recreate_tables(db: Pool<Sqlite>) {
        sqlx::query(
            "DROP TABLE IF EXISTS medias;
            DROP TABLE IF EXISTS media_categories;",
        )
        .execute(&db)
        .await
        .expect("LibrarySystem drop tables failed!");
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS medias(
            id INTEGER PRIMARY KEY,
            path TEXT NOT NULL,
            title TEXT NOT NULL,
            album TEXT NOT NULL,
            artist TEXT NOT NULL,
            genre TEXT NOT NULL,
            year INT NOT NULL,
            library TEXT NOT NULL,
            cover_path TEXT NULL,
            cover_url TEXT NULL,
            sample_rate INT NULL,
            bit_depth INT NULL,
            audio_bitrate INT NULL,
            overall_bitrate INT NULL,
            channels INT NULL,
            duration_seconds INT NOT NULL,
            file_name TEXT NOT NULL,
            file_type TEXT NOT NULL
        );
        CREATE INDEX IF NOT EXISTS mi_library ON medias (library);
        CREATE INDEX IF NOT EXISTS mi_album ON medias (album);
        CREATE INDEX IF NOT EXISTS mi_artist ON medias (artist);
        CREATE INDEX IF NOT EXISTS mi_genre ON medias (genre);
        CREATE INDEX IF NOT EXISTS mi_year ON medias (year);",
        )
        .execute(&db)
        .await
        .expect("Create `medias` table failed!");

        sqlx::query(
            "CREATE TABLE IF NOT EXISTS media_categories(
            category_title TEXT NOT NULL,
            media_id INT NOT NULL
        );
        CREATE INDEX IF NOT EXISTS mci_category_title ON media_categories (category_title);
        CREATE INDEX IF NOT EXISTS mci_media_id ON media_categories (media_id);",
        )
        .execute(&db)
        .await
        .expect("Create `media_categories` table failed!");
    }

    pub async fn new<'a>(db: Pool<Sqlite>, config: Arc<Config>) -> LibrarySystem {
        Self::recreate_tables(db.clone()).await;

        LibrarySystem { config, db }
    }

    pub async fn scan(&self) -> (Vec<(LibraryInfo, Vec<PathBuf>)>, usize) {
        let mut library_paths: Vec<(LibraryInfo, Vec<PathBuf>)> =
            Vec::with_capacity(self.config.libraries.len());
        let mut total_path = 0;
        for lib in &self.config.libraries {
            let files = lib.fetch().await;
            if files.is_empty() {
                continue;
            } else {
                total_path += files.len();
                library_paths.push((lib.clone(), files));
            }
        }
        (library_paths, total_path)
    }

    pub async fn perform_medias(
        &self,
        plgsys: &plugin_system::PluginSystem,
        library_paths: Vec<(LibraryInfo, Vec<PathBuf>)>,
        total_path: usize,
    ) {
        let start = time::Instant::now();
        Self::recreate_tables(self.db.clone()).await;
        let mut plugins_context = if plgsys.exists_plugins().await {
            Some(plgsys.init_plugins_context().await)
        } else {
            None
        };

        let mut media_start_id = 0;
        let mut medias_handlers = Vec::with_capacity(total_path);
        info!("Performing the medias..");
        for (library, paths) in &library_paths {
            for path in paths {
                let config = self.config.clone();
                let path = path.clone();
                let library = library.clone();
                media_start_id += 1;
                let handler = tokio::spawn(async move {
                    let mut meta = match MediaMetaInfo::read_from_path(&path, &config).await {
                        Ok(v) => v,
                        Err(err) => {
                            error!("Read media meta info failed: {:?}", err);
                            return None;
                        }
                    };
                    if let Some(path) = get_image_path_media(&path) {
                        meta.cover = Some(path);
                    };

                    let info = MediaInfo::from_meta(
                        meta,
                        config.clone(),
                        &library,
                        media_start_id,
                        path.clone(),
                    );

                    Some(info)
                });
                medias_handlers.push(handler);
            }
        }

        let mut have_category = false;
        let mut medias: Vec<MediaInfo> = join_all(medias_handlers)
            .await
            .into_iter()
            .filter(|item| item.as_ref().is_ok_and(|inner| inner.is_some()))
            .map(|item| {
                let media = item.unwrap().unwrap();
                if !have_category && !media.categories.is_empty() {
                    have_category = true;
                }
                media
            })
            .collect();
        if medias.is_empty() {
            return;
        }

        if let Some(ctx) = plugins_context.as_mut() {
            for media in &mut medias {
                ctx.process_media_info_json(media).await;
            }
        }

        if have_category {
            for medias in medias.chunks(SQLITE_LIMIT) {
                let mut b = QueryBuilder::new(
                    "INSERT INTO media_categories(category_title, media_id) VALUES ",
                );
                let mut s = b.separated(",");
                for media in medias {
                    for category in &media.categories {
                        s.push("(")
                            .push_bind_unseparated(category)
                            .push_unseparated(",")
                            .push_bind_unseparated(media.id)
                            .push_unseparated(")");
                    }
                }
                b.build()
                    .execute(&self.db)
                    .await
                    .expect("Insert categories failed!");
            }
        }

        let mut total_media = 0;
        for medias in medias.chunks(SQLITE_LIMIT) {
            let r = QueryBuilder::new("INSERT INTO medias(id, path, cover_path, cover_url, title, library, album, artist, genre, year, sample_rate, bit_depth, audio_bitrate, overall_bitrate, channels, duration_seconds, file_name, file_type) ").push_values(medias, |mut b, media| {
                b.push_bind(media.id)
                    .push_bind(media.path.to_string_lossy())
                    .push_bind(media.cover_path.as_ref().and_then(|p|p.to_str()))
                    .push_bind(&media.cover_url)
                    .push_bind(&media.title)
                    .push_bind(&media.library)
                    .push_bind(&media.album)
                    .push_bind(&media.artist)
                    .push_bind(&media.genre)
                    .push_bind(media.year)
                    .push_bind(media.sample_rate)
                    .push_bind(media.bit_depth)
                    .push_bind(media.audio_bitrate)
                    .push_bind(media.overall_bitrate)
                    .push_bind(media.channels)
                    .push_bind(media.duration_seconds as u32)
                .push_bind(&media.file_name)
            .push_bind(&media.file_type);
            }).build().execute(&self.db).await.expect("Insert medias failed!");
            total_media += r.rows_affected();
        }

        info!(
            "{total_media} medias saved in {:.2}s",
            start.elapsed().as_secs_f32()
        )
    }

    pub async fn reload(&self, plgsys: &plugin_system::PluginSystem) {
        info!("Scanning all library..");
        let (library_paths, total_path) = self.scan().await;
        self.perform_medias(plgsys, library_paths, total_path).await;
    }

    pub async fn get_media_file_by_id(&self, id: i64) -> Option<PathBuf> {
        sqlx::query("SELECT path FROM medias WHERE id=? LIMIT 1")
            .bind(id)
            .fetch_optional(&self.db)
            .await
            .expect("Get media file by id failed!")
            .map(|row| {
                let str: String = row.get("path");
                Path::new(&str).to_path_buf()
            })
    }

    pub async fn get_media_cover_file_by_id(&self, id: i64) -> Option<PathBuf> {
        sqlx::query("SELECT cover_path FROM medias WHERE id=? LIMIT 1")
            .bind(id)
            .fetch_optional(&self.db)
            .await
            .expect("Get media cover file by id failed!")
            .map(|row| {
                let str: String = row.get("cover_path");
                Path::new(&str).to_path_buf()
            })
    }

    pub async fn get_media_info_by_id(&self, id: i64) -> Option<MediaInfo> {
        if let Some(row) = sqlx::query("SELECT * FROM medias WHERE id = ? LIMIT 1")
            .bind(id)
            .fetch_optional(&self.db)
            .await
            .expect("Get media by id failed!")
        {
            Some(self.from_row(row).await)
        } else {
            None
        }
    }

    pub fn get_sources_core_query<'a>(
        &self,
        main: &str,
        source: Source<'a>,
    ) -> QueryBuilder<'a, Sqlite> {
        let mut builder = QueryBuilder::new(main);
        match source {
            Source::Any => (),
            Source::Library(_) => {
                builder.push(" GROUP BY library");
            }
            Source::Category(_) => {
                builder.push(" GROUP BY category_title");
            }
            Source::Album(_) => {
                builder.push(" GROUP BY album");
            }
            Source::Artist(_) => {
                builder.push(" GROUP BY artist");
            }
            Source::Genre(_) => {
                builder.push(" GROUP BY genre");
            }
            Source::Year(_) => {
                builder.push(" WHERE year > 0 GROUP BY year");
            }
        };
        builder
    }

    pub async fn get_sources<'a>(
        &self,
        source: Source<'a>,
        index: usize,
        limit: usize,
    ) -> Vec<SourceInfo> {
        let col = match source {
            Source::Category(_) => "category_title",
            Source::Album(_) => "album",
            Source::Artist(_) => "artist",
            Source::Genre(_) => "genre",
            Source::Year(_) => "CAST(year AS TEXT)",
            _ => "library",
        };
        let table = match source {
            Source::Category(_) => "media_categories",
            _ => "medias",
        };
        let main = format!("SELECT COUNT(1) AS count, {col} AS label FROM {table}");
        let mut builder = self.get_sources_core_query(&main, source);
        let rows = builder
            .push(" LIMIT ")
            .push_bind(limit as i64)
            .push(" OFFSET ")
            .push_bind((index * limit) as i64)
            .build()
            .fetch_all(&self.db)
            .await
            .expect("Get all source failed!");
        let mut result = Vec::with_capacity(rows.len());
        for row in rows {
            result.push(SourceInfo {
                title: row.get("label"),
                total_media: row.get("count"),
            })
        }
        result
    }

    pub async fn get_total_source<'a>(&self, source: Source<'a>) -> usize {
        let col = match source {
            Source::Category(_) => "category_title",
            Source::Album(_) => "album",
            Source::Artist(_) => "artist",
            Source::Genre(_) => "genre",
            Source::Year(_) => "year",
            _ => "library",
        };
        let table = match source {
            Source::Category(_) => "media_categories",
            _ => "medias",
        };
        let wq = match source {
            Source::Year(_) => "WHERE year > 0",
            _ => "",
        };
        let main = format!("SELECT COUNT(DISTINCT {col}) AS count FROM {table} {wq}");
        QueryBuilder::new(&main)
            .build()
            .fetch_optional(&self.db)
            .await
            .expect("Get total media failed!")
            .map(|row| row.get::<u32, _>("count") as usize)
            .unwrap_or(0)
    }

    pub fn get_medias_core_query<'a>(
        &self,
        main: &str,
        source: Source<'a>,
        to_search: Option<&str>,
    ) -> QueryBuilder<'a, Sqlite> {
        let mut builder = QueryBuilder::new(main);

        let exists_source = match source {
            Source::Any => false,
            _ => true,
        };
        let is_search = to_search.is_some() && !to_search.as_ref().unwrap().trim().is_empty();
        if exists_source || is_search {
            builder.push(" WHERE");
        }
        match source {
            Source::Any => (),
            Source::Library(v) => {
                if let Some(v) = v {
                    builder.push(" library = ").push_bind(v);
                }
            }
            Source::Category(v) => {
                if let Some(v) = v {
                    builder.push(" category_title = ").push_bind(v);
                }
            }
            Source::Album(v) => {
                if let Some(v) = v {
                    builder.push(" album = ").push_bind(v);
                }
            }
            Source::Artist(v) => {
                if let Some(v) = v {
                    builder.push(" artist = ").push_bind(v);
                }
            }
            Source::Genre(v) => {
                if let Some(v) = v {
                    builder.push(" genre = ").push_bind(v);
                }
            }
            Source::Year(v) => {
                if v > 0 {
                    builder.push(" year = ").push_bind(v);
                }
            }
        };
        if is_search {
            let search = to_search.unwrap();
            if exists_source {
                builder.push(" AND");
            }
            let v = format!("%{search}%");
            builder
                .push(" (title LIKE ")
                .push_bind(v.clone())
                .push(" OR album LIKE ")
                .push_bind(v.clone())
                .push(" OR artist LIKE ")
                .push_bind(v.clone())
                .push(" OR genre LIKE ")
                .push_bind(v.clone())
                .push(")");
        }
        builder
    }

    pub async fn get_medias<'a>(
        &self,
        source: Source<'a>,
        to_search: Option<&str>,
        index: usize,
        limit: usize,
    ) -> Vec<MediaInfo> {
        let main = match source {
            Source::Category(_) => {
                "SELECT * FROM medias INNER JOIN media_categories ON media_categories.media_id = id"
            }
            _ => "SELECT * FROM medias",
        };
        let mut builder = self.get_medias_core_query(main, source, to_search);
        let rows = builder
            .push(" LIMIT ")
            .push_bind(limit as i64)
            .push(" OFFSET ")
            .push_bind((index * limit) as i64)
            .build()
            .fetch_all(&self.db)
            .await
            .expect("Get all medias failed!");
        let mut result = Vec::with_capacity(rows.len());
        for row in rows {
            result.push(self.from_row(row).await);
        }
        result
    }

    pub async fn get_total_media<'a>(&self, source: Source<'a>, to_search: Option<&str>) -> usize {
        let main = match source {
            Source::Category(_) => "SELECT COUNT(1) AS count FROM medias INNER JOIN media_categories ON media_categories.media_id = id",
            _ => "SELECT COUNT(1) AS count FROM medias"
        };
        self.get_medias_core_query(main, source, to_search)
            .build()
            .fetch_optional(&self.db)
            .await
            .expect("Get total media failed!")
            .map(|row| row.get::<u32, _>("count") as usize)
            .unwrap_or(0)
    }

    pub async fn from_row(&self, row: SqliteRow) -> MediaInfo {
        let id = row.get("id");
        MediaInfo {
            id,
            title: row.get("title"),
            album: row.get("album"),
            artist: row.get("artist"),
            genre: row.get("genre"),
            year: row.get("year"),
            library: row.get("library"),
            path: Path::new(&row.get::<String, _>("path")).to_path_buf(),
            cover_path: {
                let str: Option<String> = row.get("cover_path");
                str.map(|p| Path::new(&p).to_path_buf())
            },
            cover_url: row.get("cover_url"),
            categories: sqlx::query("SELECT category_title FROM media_categories WHERE media_id=?")
                .bind(id)
                .fetch_all(&self.db)
                .await
                .expect("Fetch all categories failed!")
                .into_iter()
                .map(|row| row.get("category_title"))
                .collect(),
            sample_rate: row.get("sample_rate"),
            bit_depth: row.get("bit_depth"),
            audio_bitrate: row.get("audio_bitrate"),
            overall_bitrate: row.get("overall_bitrate"),
            channels: row.get("channels"),
            duration_seconds: row.get("duration_seconds"),
            file_name: row.get("file_name"),
            file_type: row.get("file_type"),
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
