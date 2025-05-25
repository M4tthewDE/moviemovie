use anyhow::{bail, Result};
use tokio_postgres::{Client, NoTls};

use crate::{tmdb::MovieDetails, Config, Packet};

mod embedded {
    use refinery::embed_migrations;
    embed_migrations!("migrations");
}

pub struct DatabaseWriter {
    client: Client,
    rx: tokio::sync::mpsc::Receiver<Packet>,
}

impl DatabaseWriter {
    pub async fn new(config: &Config, rx: tokio::sync::mpsc::Receiver<Packet>) -> Result<Self> {
        let conn_string = format!(
            "host={} user={} password={}",
            config.postgres_host, config.postgres_user, config.postgres_password,
        );

        let (client, connection) = tokio_postgres::connect(&conn_string, NoTls).await?;

        tokio::spawn(async move {
            if let Err(e) = connection.await {
                eprintln!("connection error: {}", e);
            }
        });

        Ok(Self { client, rx })
    }

    pub async fn run_migrations(&mut self) -> Result<()> {
        println!("running migrations");
        embedded::migrations::runner()
            .run_async(&mut self.client)
            .await?;
        Ok(())
    }

    pub async fn run(&mut self) -> Result<()> {
        loop {
            match self.rx.recv().await {
                Some(packet) => self.insert_movie_details(packet.movie_details).await?,
                None => bail!("sender closed channel"),
            }
        }
    }

    async fn insert_movie_details(&mut self, movie_details: MovieDetails) -> Result<()> {
        self.client
            .query(
                "INSERT INTO movies (tmdb_id, title, release_date, runtime_minutes) VALUES ($1, $2, $3, $4)
                    ON CONFLICT (tmdb_id)
                    DO UPDATE SET
                    title = EXCLUDED.title,
                    release_date = EXCLUDED.release_date,
                    runtime_minutes = EXCLUDED.runtime_minutes",
                &[
                    &movie_details.id,
                    &movie_details.original_title,
                    &movie_details.release_date,
                    &movie_details.runtime,
                ],
            )
            .await?;
        Ok(())
    }
}
