use std::time::Duration;

use anyhow::{Context, Result, bail};
use reqwest::Client;
use serde::de::DeserializeOwned;

use crate::api::models::{
    ApiResponse, EnglishSurah, QuranApiIdSurah, Surah, SurahListItem, TafsirSurah,
    UmmahApiResponse, UmmahTafsirSurah,
};
use crate::domain::Lang;

const API_BASE_ID: &str = "https://equran.id/api/v2";
const API_BASE_EN: &str = "https://equran.id/api/en";
const QURAN_API_ID_BASE: &str = "https://quran-api-id.vercel.app";
const UMMAH_API_BASE: &str = "https://ummahapi.com";
/// English commentary with the best ayah coverage among UmmahAPI's sources.
const UMMAH_TAFSIR_EN: &str = "maarif";

#[derive(Clone)]
pub struct ApiClient {
    client: Client,
    /// Optional UmmahAPI key. The public endpoint works without one; the
    /// variable keeps higher-quota keys out of the repository.
    ummah_api_key: Option<String>,
}

impl ApiClient {
    pub fn new() -> Result<Self> {
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .user_agent(concat!("equran-cli/", env!("CARGO_PKG_VERSION")))
            .build()
            .context("failed to create HTTP client")?;
        let ummah_api_key = std::env::var("UMMAH_API_KEY")
            .ok()
            .map(|key| key.trim().to_owned())
            .filter(|key| !key.is_empty());
        Ok(Self {
            client,
            ummah_api_key,
        })
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

    pub async fn fetch_tafsir(&self, number: u8, lang: Lang) -> Result<TafsirSurah> {
        match lang {
            Lang::Id => {
                let surah: QuranApiIdSurah = self
                    .get_wrapped(&format!("{QURAN_API_ID_BASE}/surah/{number}"))
                    .await?;
                Ok(surah.into())
            }
            Lang::En => self.fetch_tafsir_english(number).await,
        }
    }

    async fn fetch_tafsir_english(&self, number: u8) -> Result<TafsirSurah> {
        let url = format!("{UMMAH_API_BASE}/api/tafsir/{UMMAH_TAFSIR_EN}/surah/{number}");
        let mut request = self.client.get(&url);
        if let Some(key) = &self.ummah_api_key {
            request = request.query(&[("apikey", key)]);
        }

        let response = request
            .send()
            .await
            .with_context(|| format!("failed to request English tafsir from {url}"))?;
        let status = response.status();
        if !status.is_success() {
            bail!("English tafsir request failed with HTTP {status} for {url}");
        }

        let body: UmmahApiResponse<UmmahTafsirSurah> = response
            .json()
            .await
            .with_context(|| format!("failed to parse English tafsir JSON from {url}"))?;
        if !body.success {
            bail!("UmmahAPI rejected the English tafsir request for surah {number}");
        }
        Ok(body.data.into())
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
