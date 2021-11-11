use chrono::{Duration, Utc};
use clokwerk::{ScheduleHandle, Scheduler, TimeUnits};
use diesel::prelude::*;
use std::collections::HashMap;
use std::fs;
use std::ops::Sub;

use crate::util::Error::UnsupportedSource;
use crate::util::Result;

use super::db;
use super::models::SourceType;
use super::schema;

mod image;
mod youtube;

pub struct Worker {
    scheduler: Scheduler,
    pool: db::ConnectionPool,
}

impl Worker {
    pub fn new(database_url: &str) -> Worker {
        Worker {
            scheduler: Scheduler::new(),
            pool: db::ConnectionPool::new(database_url),
        }
    }

    pub fn start(mut self, interval: Duration) -> Result<ScheduleHandle> {
        let pool = self.pool.clone();
        self.scheduler.every(15.minutes()).run(move || {
            if let Err(e) = update_sources(&pool.get()) {
                eprintln!("Unable to update sources: {}", e);
            }
        });

        let pool = self.pool.clone();
        self.scheduler.every(1.minutes()).run(move || {
            if let Err(e) = download_images(&pool.get()) {
                eprintln!("Unable to download images: {}", e);
            }
        });

        let pool = self.pool.clone();
        self.scheduler.every(1.hours()).run(move || {
            if let Err(e) = remove_images(&pool.get()) {
                eprintln!("Unable to remove old images: {}", e);
            }
        });

        // Initial update.
        let count = update_sources(&self.pool.get())?;
        println!("Updated {} sources.", count);

        let interval = interval.to_std()?;
        Ok(self.scheduler.watch_thread(interval))
    }
}

/// Updates source playlist URLs if they don't exist or haven't been updated in 5 minutes.
fn update_sources(conn: &SqliteConnection) -> Result<usize> {
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
fn download_images(conn: &SqliteConnection) -> Result<usize> {
    use schema::sources::dsl;

    let sources = dsl::sources.load::<db::Source>(conn)?;
    let mut count = 0;

    for source in &sources {
        use schema::images::{dsl, table};

        if !source.enabled {
            continue;
        }

        // Create the target directory, if necessary.
        let directory = format!("images/{}", source.name);
        fs::create_dir_all(&directory)?;

        let timestamp = Utc::now().timestamp();
        let filename = format!("{}/{}.jpg", directory, timestamp);

        let result = match SourceType::from(source.typ) {
            SourceType::Url => image::download(source, &filename),
            SourceType::YouTube => youtube::download(source, &filename),
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
fn remove_images(conn: &SqliteConnection) -> Result<usize> {
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
            let filename = format!("images/{}/{}.jpg", source, image.timestamp);
            fs::remove_file(filename).ok();
        }
    }

    Ok(images.len())
}
