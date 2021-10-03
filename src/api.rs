use diesel::prelude::*;
use rocket::http::ContentType;
use rocket::tokio::task::spawn_blocking;
use rocket::State;
use std::fs;

use super::db;
use super::schema;

#[get("/images/<source_name>/<timestamp>")]
pub async fn get_image(
    pool: &State<db::ConnectionPool>,
    source_name: &str,
    timestamp: i64,
) -> Option<(ContentType, Option<Vec<u8>>)> {
    let conn = pool.get();

    use schema::sources::dsl;

    let source = dsl::sources
        .filter(dsl::name.eq(source_name))
        .first::<db::Source>(&conn)
        .ok()?;
    let image = get_closest_image(&conn, source.id, timestamp)?;

    let path = format!("images/{}/{}.jpg", source.name, image.timestamp);
    let data = spawn_blocking(|| fs::read(path)).await.unwrap().ok();

    Some((ContentType::JPEG, data))
}

fn get_closest_image(conn: &SqliteConnection, source_id: i64, timestamp: i64) -> Option<db::Image> {
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

    vec![older, newer]
        .into_iter()
        .filter_map(|image| image.ok())
        .map(|image| {
            let diff = (image.timestamp - timestamp).abs();
            (image, diff)
        })
        .min_by_key(|(_, diff)| *diff)
        .map(|(image, _)| image)
}
