use std::path::{Path, PathBuf};

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
        if is_non_empty_file(&target).await {
            return Ok(target);
        }
        let bytes = api.download_bytes(url).await?;
        fs::write(&target, bytes)
            .await
            .with_context(|| format!("failed to write {}", target.display()))?;
        Ok(target)
    }
}

/// Returns true when `path` exists and holds more than a bare file header.
///
/// Cached media can be left truncated by an interrupted download or synthesis.
/// Treating such a stub as a cache hit produces silent playback for WAV files,
/// so size is validated on every read.
pub async fn is_non_empty_file(path: &Path) -> bool {
    match fs::metadata(path).await {
        Ok(metadata) => metadata.is_file() && metadata.len() > 44,
        Err(_) => false,
    }
}

/// Validates the canonical PCM WAV header written by the TTS engine.
///
/// Older engine builds could leave a WAV container that the native decoder
/// rejects. Checking the header means such files are regenerated instead of
/// being replayed as silence.
pub async fn is_valid_pcm_wav(path: &Path) -> bool {
    use tokio::io::AsyncReadExt;

    let Ok(metadata) = fs::metadata(path).await else {
        return false;
    };
    if !metadata.is_file() || metadata.len() < 44 {
        return false;
    }

    let Ok(mut file) = fs::File::open(path).await else {
        return false;
    };
    let mut header = [0u8; 44];
    if file.read_exact(&mut header).await.is_err() {
        return false;
    }

    let riff = &header[0..4] == b"RIFF";
    let wave = &header[8..12] == b"WAVE";
    let format = u16::from_le_bytes([header[20], header[21]]);
    let channels = u16::from_le_bytes([header[22], header[23]]);
    let bits_per_sample = u16::from_le_bytes([header[34], header[35]]);

    riff && wave && format == 1 && channels > 0 && bits_per_sample == 16
}

pub fn audio_filename(qari_key: &str, surah: u8, ayah: u16) -> String {
    format!("{qari_key}_{surah:03}_{ayah:03}.mp3")
}

/// Every synthesized file is prefixed with the engine name so artifacts from a
/// previous engine are never mistaken for current output.
const TTS_PREFIX: &str = "supertonic";

pub fn tts_filename(lang: &str, surah: u8, ayah: u16) -> String {
    format!("{TTS_PREFIX}_{lang}_{surah:03}_{ayah:03}.wav")
}

/// Cache name for one spoken tafsir chunk.
///
/// The text hash is part of the name because cleaning can remove an
/// all-script paragraph and shift every following chunk index. Without it, an
/// older file for index N could be replayed for different words.
pub fn tafsir_tts_chunk_filename(
    lang: &str,
    surah: u8,
    ayah: u16,
    chunk: usize,
    text: &str,
) -> String {
    chunk_filename("tafsir", lang, surah, ayah, chunk, text)
}

/// Cache name for one spoken chunk of the ayah translation.
///
/// English uses the tafsir as the translation, and a tafsir can be thousands of
/// characters long. It is spoken chunk by chunk, so it needs the same
/// text-addressed naming as the tafsir narration instead of the single
/// `supertonic_<lang>_<surah>_<ayah>.wav` file.
pub fn translation_tts_chunk_filename(
    lang: &str,
    surah: u8,
    ayah: u16,
    chunk: usize,
    text: &str,
) -> String {
    chunk_filename("translation", lang, surah, ayah, chunk, text)
}

fn chunk_filename(
    kind: &str,
    lang: &str,
    surah: u8,
    ayah: u16,
    chunk: usize,
    text: &str,
) -> String {
    format!(
        "{kind}_{TTS_PREFIX}_{lang}_{surah:03}_{ayah:03}_{chunk:03}_{:08x}.wav",
        text_hash(text)
    )
}

fn text_hash(text: &str) -> u32 {
    use std::hash::{Hash, Hasher};

    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    text.hash(&mut hasher);
    hasher.finish() as u32
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn creates_stable_cache_filenames() {
        assert_eq!(audio_filename("05", 1, 7), "05_001_007.mp3");
        assert_eq!(tts_filename("id", 1, 7), "supertonic_id_001_007.wav");
        assert_eq!(tts_filename("en", 36, 12), "supertonic_en_036_012.wav");
        let name = tafsir_tts_chunk_filename("id", 1, 7, 2, "Bismillah");
        assert!(name.starts_with("tafsir_supertonic_id_001_007_002_"));
        assert!(name.ends_with(".wav"));
        // The text, not just the position, identifies the audio.
        assert_ne!(
            name,
            tafsir_tts_chunk_filename("id", 1, 7, 2, "Alhamdulillah")
        );

        let translation = translation_tts_chunk_filename("en", 1, 2, 1, "This Surah");
        assert!(translation.starts_with("translation_supertonic_en_001_002_001_"));
        // Translation and tafsir audio never share a file.
        assert_ne!(
            translation,
            tafsir_tts_chunk_filename("en", 1, 2, 1, "This Surah")
        );
    }

    #[tokio::test]
    async fn rejects_missing_and_truncated_files() {
        let temp_dir = tempfile::tempdir().expect("temp dir should be created");

        assert!(!is_non_empty_file(&temp_dir.path().join("missing.wav")).await);

        let truncated = temp_dir.path().join("truncated.wav");
        tokio::fs::write(&truncated, b"RIFF")
            .await
            .expect("file should be written");
        assert!(!is_non_empty_file(&truncated).await);
    }

    #[tokio::test]
    async fn accepts_complete_files() {
        let temp_dir = tempfile::tempdir().expect("temp dir should be created");
        let complete = temp_dir.path().join("complete.wav");
        tokio::fs::write(&complete, vec![0u8; 128])
            .await
            .expect("file should be written");

        assert!(is_non_empty_file(&complete).await);
    }

    fn pcm_wav_header() -> Vec<u8> {
        let mut header = vec![0u8; 44];
        header[0..4].copy_from_slice(b"RIFF");
        header[4..8].copy_from_slice(&36u32.to_le_bytes());
        header[8..12].copy_from_slice(b"WAVE");
        header[12..16].copy_from_slice(b"fmt ");
        header[16..20].copy_from_slice(&16u32.to_le_bytes());
        header[20..22].copy_from_slice(&1u16.to_le_bytes());
        header[22..24].copy_from_slice(&1u16.to_le_bytes());
        header[24..28].copy_from_slice(&44_100u32.to_le_bytes());
        header[34..36].copy_from_slice(&16u16.to_le_bytes());
        header
    }

    #[tokio::test]
    async fn accepts_pcm_wav_headers() {
        let temp_dir = tempfile::tempdir().expect("temp dir should be created");
        let path = temp_dir.path().join("valid.wav");
        tokio::fs::write(&path, pcm_wav_header())
            .await
            .expect("file should be written");

        assert!(is_valid_pcm_wav(&path).await);
    }

    #[tokio::test]
    async fn rejects_non_pcm_wav_headers() {
        let temp_dir = tempfile::tempdir().expect("temp dir should be created");
        let path = temp_dir.path().join("float.wav");
        let mut header = pcm_wav_header();
        header[20..22].copy_from_slice(&3u16.to_le_bytes());
        tokio::fs::write(&path, header)
            .await
            .expect("file should be written");

        assert!(!is_valid_pcm_wav(&path).await);
    }

    #[tokio::test]
    async fn rejects_truncated_wav_headers() {
        let temp_dir = tempfile::tempdir().expect("temp dir should be created");
        let path = temp_dir.path().join("short.wav");
        tokio::fs::write(&path, b"RIFF")
            .await
            .expect("file should be written");

        assert!(!is_valid_pcm_wav(&path).await);
    }
}
