use rusqlite::Connection;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::{Duration, Instant};

use crate::config::Config;
use crate::util;
use crate::util::Error::UnsupportedSource;
use crate::util::Result;

use super::db::{self, SourceType};

mod image;
mod stream;
mod youtube;

pub struct WorkerHandle {
    running: Arc<AtomicBool>,
}

impl WorkerHandle {
    pub fn stop(&self) {
        self.running.store(false, Ordering::Relaxed);
    }
}

pub struct Worker {
    database_path: PathBuf,
    image_dir: PathBuf,
}

impl Worker {
    pub fn new(config: &Config) -> Worker {
        Worker {
            database_path: config.database_path.clone(),
            image_dir: config.image_dir.clone(),
        }
    }

    pub fn start(self, interval: Duration) -> Result<WorkerHandle> {
        let running = Arc::new(AtomicBool::new(true));
        let handle = WorkerHandle {
            running: running.clone(),
        };

        // Initial update.
        let conn = db::open(&self.database_path)?;
        let count = update_sources(&conn)?;
        println!("Updated {} source(s).", count);

        thread::spawn(move || {
            if let Ok(conn) = db::open(&self.database_path) {
                self.run_loop(&conn, running, interval);
            }
        });

        Ok(handle)
    }

    fn run_loop(&self, conn: &Connection, running: Arc<AtomicBool>, interval: Duration) {
        let mut last_images = Instant::now();
        let mut last_sources = Instant::now();
        let mut last_cleanup = Instant::now();

        while running.load(Ordering::Relaxed) {
            thread::sleep(interval);

            if last_images.elapsed() >= Duration::from_mins(1) {
                last_images = Instant::now();
                if let Err(e) = download_images(conn, &self.image_dir) {
                    eprintln!("Unable to download images: {}", e);
                }
            }

            if last_sources.elapsed() >= Duration::from_mins(15) {
                last_sources = Instant::now();
                if let Err(e) = update_sources(conn) {
                    eprintln!("Unable to update sources: {}", e);
                }
            }

            if last_cleanup.elapsed() >= Duration::from_hours(1) {
                last_cleanup = Instant::now();
                if let Err(e) = remove_images(conn, &self.image_dir) {
                    eprintln!("Unable to remove old images: {}", e);
                }
            }
        }
    }
}

/// Updates source playlist URLs if they don't exist or haven't been updated in 5 minutes.
fn update_sources(conn: &Connection) -> Result<usize> {
    let threshold = util::unix_timestamp() - Duration::from_mins(5).as_secs() as i64;
    let mut stmt = conn.prepare(
        "SELECT id, name, typ, url, playlist, headers
         FROM sources
         WHERE enabled = 1 AND (playlist IS NULL OR updated_at <= ?1)",
    )?;

    let sources = stmt
        .query_map([threshold], db::Source::from_row)?
        .collect::<rusqlite::Result<Vec<_>>>()?;

    let mut count = 0;
    for source in &sources {
        let result = match SourceType::from(source.typ) {
            SourceType::YouTube => youtube::update(source, conn),
            _ => continue,
        };

        if let Err(e) = result {
            eprintln!("Unable to update source '{}': {}", source.name, e);
            continue;
        }

        count += 1;
    }

    Ok(count)
}

/// Downloads the images of all sources.
fn download_images(conn: &Connection, image_dir: &Path) -> Result<usize> {
    let mut stmt = conn.prepare(
        "SELECT id, name, typ, url, playlist, headers
         FROM sources
         WHERE enabled = 1",
    )?;

    let sources = stmt
        .query_map([], db::Source::from_row)?
        .collect::<rusqlite::Result<Vec<_>>>()?;

    let mut count = 0;
    for source in &sources {
        let directory = image_dir.join(&source.name);
        fs::create_dir_all(&directory)?;

        let timestamp = util::unix_timestamp();
        let filename = directory.join(format!("{}.jpg", timestamp));

        let result = match SourceType::from(source.typ) {
            SourceType::Url => image::download(source, &filename),
            SourceType::YouTube | SourceType::Stream => stream::download(source, &filename),
            typ => Err(UnsupportedSource(typ).into()),
        };

        if let Err(e) = result {
            eprintln!(
                "Unable to download image for source '{}': {}",
                source.name, e
            );
            continue;
        }

        conn.execute(
            "INSERT INTO images (source_id, timestamp) VALUES (?1, ?2)",
            (source.id, timestamp),
        )?;

        count += 1;
    }

    Ok(count)
}

/// Removes images older than 7 days.
fn remove_images(conn: &Connection, image_dir: &Path) -> Result<usize> {
    let threshold = util::unix_timestamp() - Duration::from_hours(7 * 24).as_secs() as i64;

    let mut stmt = conn.prepare(
        "SELECT images.timestamp, sources.name
         FROM images
         JOIN sources ON images.source_id = sources.id
         WHERE images.timestamp <= ?1",
    )?;

    let old_images = stmt
        .query_map([threshold], |row| {
            Ok((row.get::<_, i64>(0)?, row.get::<_, String>(1)?))
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?;

    conn.execute("DELETE FROM images WHERE timestamp <= ?1", [threshold])?;

    for (timestamp, source_name) in &old_images {
        let filename = image_dir
            .join(source_name)
            .join(format!("{}.jpg", timestamp));
        fs::remove_file(filename).ok();
    }

    Ok(old_images.len())
}
