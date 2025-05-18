use anyhow::Result;
use tokio_postgres::{Client, NoTls};

use crate::Config;

mod embedded {
    use refinery::embed_migrations;
    embed_migrations!("migrations");
}

pub struct DbClient {
    client: Client,
}

impl DbClient {
    pub async fn new(config: &Config) -> Result<Self> {
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

        Ok(DbClient { client })
    }

    pub async fn run_migrations(&mut self) -> Result<()> {
        println!("running migrations");
        embedded::migrations::runner()
            .run_async(&mut self.client)
            .await?;
        Ok(())
    }
}
