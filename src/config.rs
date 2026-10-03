use std::env;
use std::fs;
use std::path::PathBuf;

#[derive(Clone, Debug)]
pub struct Config {
    pub database_path: PathBuf,
    pub image_dir: PathBuf,
}

impl Config {
    pub fn from_env() -> Self {
        let data_dir = env::var("DATA_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from("."));

        let _ = fs::create_dir_all(&data_dir);

        let database_path = data_dir.join("database.sqlite");
        let image_dir = data_dir.join("images");

        Self {
            database_path,
            image_dir,
        }
    }
}
