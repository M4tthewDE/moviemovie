use std::path::PathBuf;

use anyhow::Result;
use db::DbClient;
use serde::Deserialize;
use tmdb::TmdbClient;

mod db;
mod tmdb;

#[derive(Deserialize)]
struct Config {
    token: String,
    postgres_host: String,
    postgres_user: String,
    postgres_password: String,
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

    let mut db = DbClient::new(&config).await?;
    db.run_migrations().await?;

    let tmdb = TmdbClient::new(&config.token)?;
    let movie_ids = tmdb.load_movie_ids().await?;
    println!("fetched {} movie ids", movie_ids.len());

    let cast = tmdb.cast(movie_ids.get(0).unwrap().id).await?;
    for member in cast.cast {
        println!("{member:?}");
    }

    Ok(())
}
