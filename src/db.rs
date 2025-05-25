use anyhow::{bail, Context, Result};
use tokio_postgres::{Client, NoTls};

use crate::{
    tmdb::{Cast, MovieDetails},
    Config, Packet,
};

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
                Some(packet) => {
                    let movie_id = self.insert_movie(&packet.movie_details).await?;
                    self.insert_cast(movie_id, &packet.cast).await?;
                }
                None => bail!("sender closed channel"),
            }
        }
    }

    async fn insert_movie(&mut self, movie_details: &MovieDetails) -> Result<i32> {
        Ok(self.client
            .query(
                "INSERT INTO movies (tmdb_id, title, release_date, runtime_minutes) VALUES ($1, $2, $3, $4)
                    ON CONFLICT (tmdb_id)
                    DO UPDATE SET
                    title = EXCLUDED.title,
                    release_date = EXCLUDED.release_date,
                    runtime_minutes = EXCLUDED.runtime_minutes,
                    created_at = EXCLUDED.created_at
                    RETURNING movie_id",
                &[
                    &movie_details.id,
                    &movie_details.original_title,
                    &movie_details.release_date,
                    &movie_details.runtime,
                ],
            )
            .await?.first().context("no movie inserted")?.get(0))
    }

    async fn insert_cast(&mut self, movie_id: i32, cast: &Cast) -> Result<()> {
        for member in &cast.cast {
            let person = self
                .client
                .query(
                    "INSERT INTO persons (tmdb_id, name) VALUES ($1, $2)
                    ON CONFLICT (tmdb_id)
                    DO UPDATE SET
                    name = EXCLUDED.name,
                    created_at = EXCLUDED.created_at
                    RETURNING person_id",
                    &[&member.id, &member.name],
                )
                .await?;

            let person_id: i32 = person.first().context("no person inserted")?.get(0);

            self.client
                .query(
                    "INSERT INTO movie_cast (movie_id, person_id, character_name) VALUES ($1, $2, $3)
                    ON CONFLICT ON CONSTRAINT movie_cast_pkey
                    DO UPDATE SET
                    movie_id = EXCLUDED.movie_id,
                    person_id = EXCLUDED.person_id,
                    character_name = EXCLUDED.character_name",
                    &[&movie_id, &person_id, &member.character],
                )
                .await?;
        }

        Ok(())
    }
}
