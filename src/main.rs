use std::path::PathBuf;

use anyhow::Result;
use serde::Deserialize;
use tmdb::TmdbClient;

mod tmdb;

#[derive(Deserialize)]
struct Config {
    token: String,
}

impl Config {
    fn new() -> Result<Config> {
        let path = PathBuf::from("config.toml");
        let data = std::fs::read_to_string(path)?;
        let config: Config = toml::from_str(&data)?;
        Ok(config)
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let config = Config::new()?;
    let tmdb = TmdbClient::new(&config.token)?;

    let movie_ids = tmdb.load_movie_ids().await?;
    println!("fetched {} movie ids", movie_ids.len());

    Ok(())
}
