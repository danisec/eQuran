use std::time::Duration;

use anyhow::{Context, Result, bail};
use reqwest::Client;
use serde::de::DeserializeOwned;

use crate::api::models::{
    ApiResponse, EnglishSurah, QuranApiIdSurah, Surah, SurahListItem, TafsirSurah,
};

const API_BASE_ID: &str = "https://equran.id/api/v2";
const API_BASE_EN: &str = "https://equran.id/api/en";
const QURAN_API_ID_BASE: &str = "https://quran-api-id.vercel.app";

#[derive(Clone)]
pub struct ApiClient {
    client: Client,
}

impl ApiClient {
    pub fn new() -> Result<Self> {
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .user_agent(concat!("equran-cli/", env!("CARGO_PKG_VERSION")))
            .build()
            .context("failed to create HTTP client")?;
        Ok(Self { client })
    }

    pub async fn fetch_surah_list(&self) -> Result<Vec<SurahListItem>> {
        self.get_wrapped(&format!("{API_BASE_ID}/surat")).await
    }

    pub async fn fetch_surah(&self, number: u8) -> Result<Surah> {
        self.get_wrapped(&format!("{API_BASE_ID}/surat/{number}"))
            .await
    }

    pub async fn fetch_surah_english(&self, number: u8) -> Result<EnglishSurah> {
        self.get_wrapped(&format!("{API_BASE_EN}/surah/{number}"))
            .await
    }

    pub async fn fetch_tafsir(&self, number: u8) -> Result<TafsirSurah> {
        let surah: QuranApiIdSurah = self
            .get_wrapped(&format!("{QURAN_API_ID_BASE}/surah/{number}"))
            .await?;
        Ok(surah.into())
    }

    pub async fn download_bytes(&self, url: &str) -> Result<Vec<u8>> {
        let response = self
            .client
            .get(url)
            .send()
            .await
            .with_context(|| format!("failed to request {url}"))?;
        let status = response.status();
        if !status.is_success() {
            bail!("download failed with HTTP {status} for {url}");
        }
        let bytes = response
            .bytes()
            .await
            .with_context(|| format!("failed to read response body from {url}"))?;
        Ok(bytes.to_vec())
    }

    async fn get_wrapped<T>(&self, url: &str) -> Result<T>
    where
        T: DeserializeOwned,
    {
        let response = self
            .client
            .get(url)
            .send()
            .await
            .with_context(|| format!("failed to request {url}"))?;
        let status = response.status();
        if !status.is_success() {
            bail!("API request failed with HTTP {status} for {url}");
        }
        let body: ApiResponse<T> = response
            .json()
            .await
            .with_context(|| format!("failed to parse JSON from {url}"))?;
        if body.code != 200 {
            bail!("API returned code {}: {}", body.code, body.message);
        }
        Ok(body.data)
    }
}
