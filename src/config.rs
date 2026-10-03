use std::env;
use std::fs;
use std::path::PathBuf;

#[derive(Clone, Debug)]
pub struct Config {
    pub database_url: String,
    pub image_dir: PathBuf,
}

impl Config {
    pub fn from_env() -> Self {
        let data_dir = env::var("DATA_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from("."));

        let _ = fs::create_dir_all(&data_dir);

        let database_url = data_dir
            .join("database.sqlite")
            .to_string_lossy()
            .into_owned();

        let image_dir = data_dir.join("images");

        Self {
            database_url,
            image_dir,
        }
    }
}
