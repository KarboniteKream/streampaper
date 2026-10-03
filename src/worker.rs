use chrono::{Duration, Utc};
use clokwerk::{ScheduleHandle, Scheduler, TimeUnits};
use diesel::prelude::*;
use std::collections::HashMap;
use std::fs;
use std::ops::Sub;
use std::path::{Path, PathBuf};

use crate::config::Config;
use crate::util::Error::UnsupportedSource;
use crate::util::Result;

use super::db;
use super::models::SourceType;
use super::schema;

mod image;
mod stream;
mod youtube;

pub struct Worker {
    scheduler: Scheduler,
    pool: db::ConnectionPool,
    image_dir: PathBuf,
}

impl Worker {
    pub fn new(config: &Config) -> Worker {
        Worker {
            scheduler: Scheduler::new(),
            pool: db::ConnectionPool::new(&config.database_url),
            image_dir: config.image_dir.clone(),
        }
    }

    pub fn start(mut self, interval: Duration) -> Result<ScheduleHandle> {
        let pool = self.pool.clone();
        self.scheduler.every(15.minutes()).run(move || {
            if let Err(e) = update_sources(&mut pool.get()) {
                eprintln!("Unable to update sources: {}", e);
            }
        });

        let pool = self.pool.clone();
        let image_dir = self.image_dir.clone();
        self.scheduler.every(1.minutes()).run(move || {
            if let Err(e) = download_images(&mut pool.get(), &image_dir) {
                eprintln!("Unable to download images: {}", e);
            }
        });

        let pool = self.pool.clone();
        let image_dir = self.image_dir.clone();
        self.scheduler.every(1.hours()).run(move || {
            if let Err(e) = remove_images(&mut pool.get(), &image_dir) {
                eprintln!("Unable to remove old images: {}", e);
            }
        });

        // Initial update.
        let count = update_sources(&mut self.pool.get())?;
        println!("Updated {} source(s).", count);

        let interval = interval.to_std()?;
        Ok(self.scheduler.watch_thread(interval))
    }
}

/// Updates source playlist URLs if they don't exist or haven't been updated in 5 minutes.
fn update_sources(conn: &mut SqliteConnection) -> Result<usize> {
    use schema::sources::dsl;

    let threshold = Utc::now().sub(Duration::minutes(5)).timestamp();
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

        let timestamp = Utc::now().timestamp();
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

    let threshold = Utc::now().sub(Duration::days(7)).timestamp();
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
