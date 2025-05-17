use std::io::{Cursor, Read};

use anyhow::Result;
use chrono::{Datelike, Local};
use flate2::read::GzDecoder;
use reqwest::{
    header::{self, HeaderMap, HeaderValue},
    Client,
};
use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct MovieIdEntry {
    pub id: u64,
    pub original_title: String,
    pub popularity: f64,
    pub adult: bool,
}

pub struct TmdbClient {
    client: Client,
}

impl TmdbClient {
    pub fn new(token: &str) -> Result<Self> {
        let mut headers = HeaderMap::new();

        let mut auth_header = HeaderValue::from_str(&format!("Bearer {token}"))?;
        auth_header.set_sensitive(true);
        headers.insert(header::AUTHORIZATION, auth_header);

        let client = Client::builder().default_headers(headers).build()?;

        Ok(Self { client })
    }

    pub async fn load_movie_ids(&self) -> Result<Vec<MovieIdEntry>> {
        let now = Local::now();
        let day = now.day().checked_sub(1).unwrap_or(1);
        let month = now.month();
        let year = now.year();
        let url =
            format!("http://files.tmdb.org/p/exports/movie_ids_{month:02}_{day}_{year}.json.gz");

        let response = self.client.get(url).send().await?.error_for_status()?;
        let bytes = response.bytes().await?;
        let cursor = Cursor::new(bytes);
        let mut decoder = GzDecoder::new(cursor);

        let mut json_text = String::new();
        decoder.read_to_string(&mut json_text)?;

        let mut movie_ids = Vec::new();
        for line in json_text.lines() {
            let movie_id: MovieIdEntry = serde_json::from_str(line)?;
            movie_ids.push(movie_id);
        }

        Ok(movie_ids)
    }
}
