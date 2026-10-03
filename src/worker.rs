use diesel::prelude::*;
use std::collections::HashMap;
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

use super::db;
use super::models::SourceType;
use super::schema;

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
    pool: db::ConnectionPool,
    image_dir: PathBuf,
}

impl Worker {
    pub fn new(config: &Config) -> Worker {
        Worker {
            pool: db::ConnectionPool::new(&config.database_url),
            image_dir: config.image_dir.clone(),
        }
    }

    pub fn start(self, interval: Duration) -> Result<WorkerHandle> {
        let running = Arc::new(AtomicBool::new(true));
        let handle = WorkerHandle {
            running: running.clone(),
        };

        // Initial update.
        let count = update_sources(&mut self.pool.get())?;
        println!("Updated {} source(s).", count);

        thread::spawn(move || self.run_loop(running, interval));

        Ok(handle)
    }

    fn run_loop(&self, running: Arc<AtomicBool>, interval: Duration) {
        let mut last_images = Instant::now();
        let mut last_sources = Instant::now();
        let mut last_cleanup = Instant::now();

        while running.load(Ordering::Relaxed) {
            thread::sleep(interval);

            if last_images.elapsed() >= Duration::from_mins(1) {
                last_images = Instant::now();
                if let Err(e) = download_images(&mut self.pool.get(), &self.image_dir) {
                    eprintln!("Unable to download images: {}", e);
                }
            }

            if last_sources.elapsed() >= Duration::from_mins(15) {
                last_sources = Instant::now();
                if let Err(e) = update_sources(&mut self.pool.get()) {
                    eprintln!("Unable to update sources: {}", e);
                }
            }

            if last_cleanup.elapsed() >= Duration::from_hours(1) {
                last_cleanup = Instant::now();
                if let Err(e) = remove_images(&mut self.pool.get(), &self.image_dir) {
                    eprintln!("Unable to remove old images: {}", e);
                }
            }
        }
    }
}

/// Updates source playlist URLs if they don't exist or haven't been updated in 5 minutes.
fn update_sources(conn: &mut SqliteConnection) -> Result<usize> {
    use schema::sources::dsl;

    let threshold = util::unix_timestamp() - Duration::from_mins(5).as_secs() as i64;
    let mut count = 0;

    let sources = dsl::sources
        .filter(dsl::playlist.is_null())
        .or_filter(dsl::updated_at.le(threshold))
        .load::<db::Source>(conn)?;

    for source in &sources {
        if !source.enabled {
            continue;
        }

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
fn download_images(conn: &mut SqliteConnection, image_dir: &Path) -> Result<usize> {
    use schema::sources::dsl;

    let sources = dsl::sources.load::<db::Source>(conn)?;
    let mut count = 0;

    for source in &sources {
        use schema::images::{dsl, table};

        if !source.enabled {
            continue;
        }

        // Create the target directory, if necessary.
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

        diesel::insert_into(table)
            .values((dsl::source_id.eq(source.id), dsl::timestamp.eq(timestamp)))
            .execute(conn)?;

        count += 1;
    }

    Ok(count)
}

/// Removes images older than 7 days.
fn remove_images(conn: &mut SqliteConnection, image_dir: &Path) -> Result<usize> {
    use schema::images::{dsl, table};

    let sources = schema::sources::dsl::sources
        .load::<db::Source>(conn)?
        .into_iter()
        .map(|source| (source.id, source.name))
        .collect::<HashMap<i64, String>>();

    let threshold = util::unix_timestamp() - Duration::from_hours(7 * 24).as_secs() as i64;
    let predicate = dsl::timestamp.le(threshold);

    let images = dsl::images.filter(&predicate).load::<db::Image>(conn)?;
    diesel::delete(table).filter(&predicate).execute(conn)?;

    for image in &images {
        if let Some(source) = sources.get(&image.source_id) {
            let filename = image_dir
                .join(source)
                .join(format!("{}.jpg", image.timestamp));
            fs::remove_file(filename).ok();
        }
    }

    Ok(images.len())
}
