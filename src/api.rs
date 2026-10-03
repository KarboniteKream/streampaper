use rusqlite::Connection;
use std::fs::File;
use tiny_http::{Header, Method, Request, Response, StatusCode};

use super::config::Config;

pub fn handle_request(request: Request, conn: &Connection, config: &Config) {
    let path = request.url().split('?').next().unwrap_or("");
    let parts: Vec<&str> = path.trim_matches('/').split('/').collect();

    if request.method() == &Method::Get
        && let ["images", source, timestamp] = parts.as_slice()
        && let Ok(timestamp) = timestamp.parse()
        && let Some(file) = get_image(conn, config, source, timestamp)
    {
        let header = "Content-Type: image/jpeg".parse::<Header>().unwrap();
        let _ = request.respond(Response::from_file(file).with_header(header));
    } else {
        let _ = request.respond(Response::empty(StatusCode(404)));
    }
}

fn get_image(
    conn: &Connection,
    config: &Config,
    source_name: &str,
    timestamp: i64,
) -> Option<File> {
    let image_timestamp: i64 = conn
        .query_row(
            "SELECT images.timestamp
             FROM images
             JOIN sources ON sources.id = images.source_id
             WHERE sources.name = ?1
             ORDER BY ABS(images.timestamp - ?2)
             LIMIT 1",
            (source_name, timestamp),
            |row| row.get(0),
        )
        .ok()?;

    let path = config
        .image_dir
        .join(source_name)
        .join(format!("{}.jpg", image_timestamp));

    File::open(path).ok()
}
