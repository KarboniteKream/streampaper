#[macro_use]
extern crate diesel;
extern crate dotenvy;
#[macro_use]
extern crate rocket;

use dotenvy::dotenv;
use std::time::Duration;

mod api;
mod config;
mod db;
mod models;
mod schema;
mod util;
mod worker;

use config::Config;

#[rocket::main]
#[allow(clippy::result_large_err)]
async fn main() -> Result<(), rocket::Error> {
    dotenv().ok();

    let config = Config::from_env();
    let pool = db::ConnectionPool::new(&config.database_url);

    let worker = worker::Worker::new(&config);
    let worker_handle = worker.start(Duration::from_secs(1)).unwrap();

    rocket::build()
        .manage(pool)
        .manage(config)
        .mount("/", routes![api::get_image])
        .launch()
        .await?;

    worker_handle.stop();
    Ok(())
}
