#[macro_use]
extern crate diesel;
extern crate dotenv;
#[macro_use]
extern crate rocket;

use chrono::Duration;
use dotenv::dotenv;
use std::env;

mod api;
mod db;
mod models;
mod schema;
mod worker;

#[rocket::main]
async fn main() -> Result<(), rocket::Error> {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let pool = db::ConnectionPool::new(&database_url);

    let worker = worker::Worker::new(&database_url);
    let worker_handle = worker.start(Duration::seconds(1));

    let rocket_result = rocket::build()
        .manage(pool)
        .mount("/", routes![api::get_image])
        .launch()
        .await;

    worker_handle.stop();
    rocket_result
}
