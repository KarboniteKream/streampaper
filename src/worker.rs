use chrono::{Duration, Utc};
use clokwerk::{ScheduleHandle, Scheduler, TimeUnits};
use diesel::prelude::*;
use std::collections::HashMap;
use std::fs;
use std::ops::Sub;

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

    pub fn start(mut self, interval: Duration) -> ScheduleHandle {
        let pool = self.pool.clone();
        self.scheduler.every(15.minutes()).run(move || {
            update_sources(&pool.get());
        });

        let pool = self.pool.clone();
        self.scheduler.every(1.minutes()).run(move || {
            download_images(&pool.get());
        });

        let pool = self.pool.clone();
        self.scheduler.every(1.hours()).run(move || {
            cleanup_images(&pool.get());
        });

        // Initial update.
        update_sources(&self.pool.get());

        self.scheduler.watch_thread(interval.to_std().unwrap())
    }
}

/// Updates source playlist URLs if they don't exist or haven't been updated in 5 minutes.
fn update_sources(conn: &SqliteConnection) {
    use schema::sources::dsl;

    let threshold = Utc::now().sub(Duration::minutes(5)).timestamp();

    let sources = dsl::sources
        .filter(dsl::playlist.is_null())
        .or_filter(dsl::updated_at.le(threshold))
        .load::<db::Source>(conn)
        .expect("Error loading sources");

    for source in sources {
        if !source.enabled {
            continue;
        }

        match SourceType::from(source.typ) {
            SourceType::YouTube => youtube::update_source(&source, conn),
            _ => continue,
        }
    }
}

/// Downloads the images of all sources.
fn download_images(conn: &SqliteConnection) {
    use schema::sources::dsl;

    let sources = dsl::sources
        .load::<db::Source>(conn)
        .expect("Error loading sources");

    for source in sources {
        use schema::images::{dsl, table};

        if !source.enabled {
            continue;
        }

        // Create the target directory, if necessary.
        let directory = format!("images/{}", source.name);
        fs::create_dir_all(&directory).unwrap();

        let timestamp = Utc::now().timestamp();
        let filename = format!("{}/{}.jpg", directory, timestamp);

        match SourceType::from(source.typ) {
            SourceType::Url => image::download(&source, &filename),
            SourceType::YouTube => youtube::download_image(&source, &filename),
            _ => continue,
        }

        let result = diesel::insert_into(table)
            .values((dsl::source_id.eq(source.id), dsl::timestamp.eq(timestamp)))
            .execute(conn);

        if let Err(e) = result {
            eprintln!("Unable to update '{}' at {}: {}", source.name, timestamp, e);
        }
    }
}

// Cleanup old images.
fn cleanup_images(conn: &SqliteConnection) {
    use schema::images::{dsl, table};

    let threshold = Utc::now().sub(Duration::days(7)).timestamp();

    let sources = schema::sources::dsl::sources
        .load::<db::Source>(conn)
        .expect("Error loading sources")
        .into_iter()
        .map(|source| (source.id, source.name))
        .collect::<HashMap<i64, String>>();

    let images = dsl::images
        .filter(dsl::timestamp.le(threshold))
        .load::<db::Image>(conn)
        .expect("Error loading images");

    for image in images {
        if !sources.contains_key(&image.source_id) {
            continue;
        }

        let source = sources.get(&image.source_id).unwrap();
        let filename = format!("images/{}/{}.jpg", source, image.timestamp);
        fs::remove_file(filename).unwrap();
    }

    let result = diesel::delete(table)
        .filter(dsl::timestamp.le(threshold))
        .execute(conn);

    if let Err(e) = result {
        eprintln!("Unable to delete old images: {}", e);
    }
}
