#[macro_use]
extern crate diesel;
extern crate dotenvy;
#[macro_use]
extern crate rocket;

use chrono::Duration;
use dotenvy::dotenv;
use std::env;

mod api;
mod db;
mod models;
mod schema;
mod util;
mod worker;

#[rocket::main]
async fn main() -> Result<(), rocket::Error> {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let pool = db::ConnectionPool::new(&database_url);

    let worker = worker::Worker::new(&database_url);
    let worker_handle = worker.start(Duration::seconds(1)).unwrap();

    rocket::build()
        .manage(pool)
        .mount("/", routes![api::get_image])
        .launch()
        .await?;

    worker_handle.stop();
    Ok(())
}
