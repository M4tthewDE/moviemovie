use std::path::PathBuf;

use anyhow::Result;
use db::DatabaseWriter;
use serde::Deserialize;
use tmdb::{Cast, MovieDetails, TmdbClient};

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

struct Packet {
    movie_details: MovieDetails,
    cast: Cast,
}

#[tokio::main]
async fn main() -> Result<()> {
    let config = Config::new()?;

    let (tx, rx) = tokio::sync::mpsc::channel(50);

    let mut db_writer = DatabaseWriter::new(&config, rx).await?;
    db_writer.run_migrations().await?;

    let _writer_handle = tokio::spawn(async move {
        db_writer.run().await.unwrap();
    });

    let tmdb = TmdbClient::new(&config.token)?;

    println!("loading movie ids");
    let movie_ids = tmdb.load_movie_ids().await?;

    println!("fetched {} movie ids", movie_ids.len());

    for (i, movie_id) in movie_ids.iter().enumerate() {
        let movie_details = tmdb.movie(movie_id.id).await?;
        let cast = tmdb.cast(movie_id.id).await?;

        let packet = Packet {
            movie_details,
            cast,
        };

        tx.send(packet).await?;

        println!(
            "sender progress: {:.4}%",
            (i as f64) / (movie_ids.len() as f64) * 100.0
        )
    }

    Ok(())
}
