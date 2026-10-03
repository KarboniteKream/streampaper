use std::time::Duration;
use tiny_http::Server;

mod api;
mod config;
mod db;
mod models;
mod util;
mod worker;

use config::Config;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = Config::from_env();
    let conn = db::open(&config.database_path)?;

    let worker = worker::Worker::new(&config);
    let worker_handle = worker.start(Duration::from_secs(1))?;

    let server = Server::http("0.0.0.0:8000").map_err(|e| e as Box<dyn std::error::Error>)?;
    println!("Listening on http://0.0.0.0:8000");

    for request in server.incoming_requests() {
        api::handle_request(request, &conn, &config);
    }

    worker_handle.stop();
    Ok(())
}
