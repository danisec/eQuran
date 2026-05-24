use std::path::PathBuf;

use anyhow::{Context, Result};
use tokio::fs;

use crate::api::client::ApiClient;

#[derive(Clone)]
pub struct AudioCache {
    root: PathBuf,
}

impl AudioCache {
    pub fn new() -> Result<Self> {
        let cache_root = dirs::cache_dir()
            .context("failed to resolve user cache directory")?
            .join("equran-cli");
        Ok(Self { root: cache_root })
    }

    pub fn root(&self) -> &PathBuf {
        &self.root
    }

    pub async fn get_or_download(
        &self,
        api: &ApiClient,
        url: &str,
        filename: &str,
    ) -> Result<PathBuf> {
        let audio_dir = self.root.join("audio");
        fs::create_dir_all(&audio_dir)
            .await
            .with_context(|| format!("failed to create {}", audio_dir.display()))?;
        let target = audio_dir.join(filename);
        if target.exists() {
            return Ok(target);
        }
        let bytes = api.download_bytes(url).await?;
        fs::write(&target, bytes)
            .await
            .with_context(|| format!("failed to write {}", target.display()))?;
        Ok(target)
    }
}

pub fn audio_filename(qari_key: &str, surah: u8, ayah: u16) -> String {
    format!("{qari_key}_{surah:03}_{ayah:03}.mp3")
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TtsBackend {
    Wibowo,
    EdgeTts,
    Espeak,
}

impl TtsBackend {
    pub fn prefix(self) -> &'static str {
        match self {
            Self::Wibowo => "wibowo",
            Self::EdgeTts => "edge",
            Self::Espeak => "espeak",
        }
    }

    pub fn extension(self) -> &'static str {
        match self {
            Self::EdgeTts => "mp3",
            Self::Wibowo | Self::Espeak => "wav",
        }
    }
}

pub fn tts_filename(backend: TtsBackend, lang: &str, surah: u8, ayah: u16) -> String {
    format!("{}_{lang}_{surah:03}_{ayah:03}.wav", backend.prefix())
}

pub fn tafsir_tts_filename(lang: &str, surah: u8, ayah: u16) -> String {
    format!("tafsir_edge_{lang}_{surah:03}_{ayah:03}.mp3")
}

pub fn tafsir_tts_chunk_filename(
    backend: TtsBackend,
    lang: &str,
    surah: u8,
    ayah: u16,
    chunk: usize,
) -> String {
    format!(
        "tafsir_{}_{lang}_{surah:03}_{ayah:03}_{chunk:03}.{}",
        backend.prefix(),
        backend.extension()
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn creates_stable_cache_filenames() {
        assert_eq!(audio_filename("05", 1, 7), "05_001_007.mp3");
        assert_eq!(
            tts_filename(TtsBackend::Wibowo, "id", 1, 7),
            "wibowo_id_001_007.wav"
        );
        assert_eq!(
            tts_filename(TtsBackend::Espeak, "en", 36, 12),
            "espeak_en_036_012.wav"
        );
        assert_eq!(
            tafsir_tts_filename("id", 1, 7),
            "tafsir_edge_id_001_007.mp3"
        );
        assert_eq!(
            tafsir_tts_chunk_filename(TtsBackend::Wibowo, "id", 1, 7, 2),
            "tafsir_wibowo_id_001_007_002.wav"
        );
    }
}
