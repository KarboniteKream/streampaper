use diesel::prelude::*;
use std::fs::File;
use tiny_http::{Header, Method, Request, Response, StatusCode};

use super::config::Config;
use super::db;
use super::schema;

pub fn handle_request(request: Request, pool: &db::ConnectionPool, config: &Config) {
    let path = request.url().split('?').next().unwrap_or("");
    let parts: Vec<&str> = path.trim_matches('/').split('/').collect();

    if request.method() == &Method::Get
        && let ["images", source, timestamp] = parts.as_slice()
        && let Ok(timestamp) = timestamp.parse()
        && let Some(file) = get_image(pool, config, source, timestamp)
    {
        let header = "Content-Type: image/jpeg".parse::<Header>().unwrap();
        let _ = request.respond(Response::from_file(file).with_header(header));
    } else {
        let _ = request.respond(Response::empty(StatusCode(404)));
    }
}

fn get_image(
    pool: &db::ConnectionPool,
    config: &Config,
    source_name: &str,
    timestamp: i64,
) -> Option<File> {
    let conn = &mut pool.get();

    use schema::sources::dsl;

    let source = dsl::sources
        .filter(dsl::name.eq(source_name))
        .first::<db::Source>(conn)
        .ok()?;
    let image = find_closest_image(conn, source.id, timestamp)?;

    let path = config
        .image_dir
        .join(&source.name)
        .join(format!("{}.jpg", image.timestamp));

    File::open(path).ok()
}

fn find_closest_image(
    conn: &mut SqliteConnection,
    source_id: i64,
    timestamp: i64,
) -> Option<db::Image> {
    use schema::images::dsl;

    let older = dsl::images
        .filter(dsl::source_id.eq(source_id))
        .filter(dsl::timestamp.le(timestamp))
        .order(dsl::timestamp.desc())
        .first::<db::Image>(conn);

    let newer = dsl::images
        .filter(dsl::source_id.eq(source_id))
        .filter(dsl::timestamp.ge(timestamp))
        .order(dsl::timestamp.asc())
        .first::<db::Image>(conn);

    [older, newer]
        .into_iter()
        .filter_map(|image| image.ok())
        .min_by_key(|image| (image.timestamp - timestamp).abs())
}
