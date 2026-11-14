use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

use anyhow::{Context, Result};
use tokio::fs;

use crate::api::client::ApiClient;

/// Default cap for cached media (qari recitation and synthesized narration), in
/// megabytes.
///
/// Narration is uncompressed PCM WAV at roughly 88 KB per second, and one ayah's
/// tafsir can run to tens of megabytes, so an unbounded cache fills a disk after
/// a single pass through a long surah. The cache keeps a working set instead and
/// discards the least recently used files past this cap. Override with
/// `EQURAN_TTS_CACHE_LIMIT_MB`; `0` turns the cap off.
pub const DEFAULT_MEDIA_LIMIT_MB: u64 = 2048;

/// Age past which a `.part` file cannot belong to a live synthesis.
///
/// Synthesis renders into a `.part` sibling and renames it into place, so a
/// leftover is always a crash artifact. The worker request itself times out
/// after 300 s, hence any file older than this grace period is safe to delete.
pub const ORPHAN_PART_MAX_AGE: Duration = Duration::from_secs(600);

/// What the media cache currently occupies on disk.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct CacheUsage {
    pub total_bytes: u64,
    pub file_count: usize,
    pub qari_bytes: u64,
    pub translation_bytes: u64,
    pub tafsir_bytes: u64,
}

/// Result of one maintenance pass, so the caller can report what happened.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct CacheMaintenance {
    pub orphan_parts_removed: usize,
    pub files_removed: usize,
    pub bytes_freed: u64,
}

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

    /// Builds a cache rooted at `root` instead of the user cache directory.
    /// Used by tests; production code always goes through [`AudioCache::new`].
    #[cfg(test)]
    fn with_root(root: PathBuf) -> Self {
        Self { root }
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

    /// Sums what the media cache occupies, split by kind.
    ///
    /// Only the two media directories are counted (`audio/` for downloaded
    /// recitation, `tts/` for synthesized narration); the search index has its
    /// own lifecycle and is left out so "clear cache" never rebuilds it.
    pub async fn usage(&self) -> CacheUsage {
        let mut usage = CacheUsage::default();
        for entry in self.media_entries().await {
            usage.total_bytes += entry.bytes;
            usage.file_count += 1;
            match entry.kind {
                MediaKind::Qari => usage.qari_bytes += entry.bytes,
                MediaKind::Translation => usage.translation_bytes += entry.bytes,
                MediaKind::Tafsir => usage.tafsir_bytes += entry.bytes,
            }
        }
        usage
    }

    /// Keeps the media cache inside its cap and removes crash leftovers.
    ///
    /// Narration is the only part that grows without bound, so this runs at
    /// startup and never while a clip is playing: it deletes the least recently
    /// used files (oldest mtime first) until the total is back under the cap.
    /// A zeroed report means there was nothing to do; failures to remove an
    /// individual file are ignored, since a file that will not delete is not
    /// worth failing startup over.
    pub async fn prune(&self) -> CacheMaintenance {
        self.prune_to(media_limit_bytes()).await
    }

    /// [`prune`](Self::prune) with an explicit cap, so tests never depend on
    /// process environment.
    async fn prune_to(&self, limit: Option<u64>) -> CacheMaintenance {
        let mut report = CacheMaintenance::default();

        // A `.part` file is a synthesis that never finished. Delete the ones old
        // enough that no live request can still own them.
        let cutoff = SystemTime::now()
            .checked_sub(ORPHAN_PART_MAX_AGE)
            .unwrap_or(SystemTime::UNIX_EPOCH);
        for entry in self.all_dir_entries().await {
            if !entry.is_part || entry.modified > cutoff {
                continue;
            }
            if fs::remove_file(&entry.path).await.is_ok() {
                report.orphan_parts_removed += 1;
                report.bytes_freed += entry.bytes;
            }
        }

        let Some(limit) = limit else {
            return report;
        };

        // Leftovers are gone, so the media list is accurate for the cap check.
        let mut entries = self.media_entries().await;
        let mut total: u64 = entries.iter().map(|entry| entry.bytes).sum();
        if total <= limit {
            return report;
        }

        // Oldest first: the file nobody touched for the longest is the safest
        // to drop, and re-synthesising it is a few seconds of CPU.
        entries.sort_by_key(|entry| entry.modified);
        for entry in entries {
            if total <= limit {
                break;
            }
            if fs::remove_file(&entry.path).await.is_ok() {
                total = total.saturating_sub(entry.bytes);
                report.files_removed += 1;
                report.bytes_freed += entry.bytes;
            }
        }
        report
    }

    /// Removes every cached media file, leaving the search index alone.
    pub async fn clear_media(&self) -> CacheMaintenance {
        let mut report = CacheMaintenance::default();
        for entry in self.all_dir_entries().await {
            if fs::remove_file(&entry.path).await.is_ok() {
                report.files_removed += 1;
                report.bytes_freed += entry.bytes;
            }
        }
        report
    }

    /// Media files that count toward the cap, with the metadata a prune needs.
    async fn media_entries(&self) -> Vec<MediaEntry> {
        let mut entries = Vec::new();
        for (dir, default_kind) in [
            (self.root.join("audio"), MediaKind::Qari),
            (self.root.join("tts"), MediaKind::Translation),
        ] {
            for entry in read_dir_entries(&dir).await {
                if entry.is_part {
                    continue;
                }
                entries.push(MediaEntry {
                    kind: media_kind(&entry.name, default_kind),
                    ..entry
                });
            }
        }
        entries
    }

    /// Every file in the media directories, including `.part` leftovers.
    async fn all_dir_entries(&self) -> Vec<MediaEntry> {
        let mut entries = Vec::new();
        for dir in [self.root.join("audio"), self.root.join("tts")] {
            entries.extend(read_dir_entries(&dir).await);
        }
        entries
    }
}

/// Which narration a cached file holds, for the usage breakdown.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum MediaKind {
    Qari,
    Translation,
    Tafsir,
}

struct MediaEntry {
    path: PathBuf,
    name: String,
    bytes: u64,
    modified: SystemTime,
    is_part: bool,
    kind: MediaKind,
}

/// Classifies a cache filename by the prefix the engine writes.
fn media_kind(name: &str, default: MediaKind) -> MediaKind {
    if name.starts_with("translation_") {
        MediaKind::Translation
    } else if name.starts_with("tafsir_") {
        MediaKind::Tafsir
    } else {
        default
    }
}

async fn read_dir_entries(dir: &Path) -> Vec<MediaEntry> {
    let Ok(mut reader) = fs::read_dir(dir).await else {
        return Vec::new();
    };
    let mut entries = Vec::new();
    while let Ok(Some(entry)) = reader.next_entry().await {
        let name = entry.file_name().to_string_lossy().into_owned();
        let Ok(metadata) = entry.metadata().await else {
            continue;
        };
        if !metadata.is_file() {
            continue;
        }
        entries.push(MediaEntry {
            path: entry.path(),
            is_part: name.ends_with(".part"),
            bytes: metadata.len(),
            modified: metadata.modified().unwrap_or(SystemTime::UNIX_EPOCH),
            kind: MediaKind::Qari,
            name,
        });
    }
    entries
}

/// The media cap in bytes, from `EQURAN_TTS_CACHE_LIMIT_MB`.
///
/// `None` disables pruning; an unparseable value falls back to the default, so a
/// typo never silently turns the cap off. `0` is a deliberate "no cap".
pub fn media_limit_bytes() -> Option<u64> {
    if let Ok(raw) = std::env::var("EQURAN_TTS_CACHE_LIMIT_MB")
        && let Ok(mb) = raw.trim().parse::<u64>()
    {
        return (mb > 0).then(|| mb.saturating_mul(1024 * 1024));
    }
    Some(DEFAULT_MEDIA_LIMIT_MB * 1024 * 1024)
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

    async fn write_media(cache: &AudioCache, dir: &str, name: &str, bytes: usize) {
        let path = cache.root.join(dir).join(name);
        tokio::fs::create_dir_all(path.parent().unwrap())
            .await
            .expect("media dir should be created");
        tokio::fs::write(&path, vec![0u8; bytes])
            .await
            .expect("media file should be written");
    }

    #[tokio::test]
    async fn usage_counts_media_by_kind_and_skips_part_files() {
        let temp_dir = tempfile::tempdir().expect("temp dir should be created");
        let cache = AudioCache::with_root(temp_dir.path().to_path_buf());

        write_media(&cache, "audio", "05_001_001.mp3", 100).await;
        write_media(
            &cache,
            "tts",
            "translation_supertonic_en_001_002_001_x.wav",
            200,
        )
        .await;
        write_media(&cache, "tts", "tafsir_supertonic_en_001_002_001_x.wav", 300).await;
        write_media(&cache, "tts", "supertonic_id_001_002.wav", 400).await;
        // A leftover render must not count as stored media.
        write_media(&cache, "tts", "supertonic_en_001_001.123.4.part", 900).await;

        let usage = cache.usage().await;

        assert_eq!(usage.file_count, 4);
        assert_eq!(usage.total_bytes, 1000);
        assert_eq!(usage.qari_bytes, 100);
        assert_eq!(usage.translation_bytes, 600);
        assert_eq!(usage.tafsir_bytes, 300);
    }

    #[tokio::test]
    async fn prunes_least_recently_used_files_down_to_the_cap() {
        let temp_dir = tempfile::tempdir().expect("temp dir should be created");
        let cache = AudioCache::with_root(temp_dir.path().to_path_buf());

        write_media(&cache, "tts", "translation_a_001_001_001_x.wav", 400).await;
        write_media(&cache, "tts", "translation_b_001_002_001_x.wav", 400).await;
        write_media(&cache, "tts", "translation_c_001_003_001_x.wav", 400).await;

        // Age the three files so the eviction order is deterministic: `a` is
        // oldest and `c` is newest.
        let set_age = |name: &str, secs: u64| {
            let path = cache.root.join("tts").join(name);
            let mtime = SystemTime::now() - Duration::from_secs(secs);
            let file = std::fs::File::options()
                .write(true)
                .open(&path)
                .expect("file should open");
            file.set_modified(mtime).expect("mtime should be set");
        };
        set_age("translation_a_001_001_001_x.wav", 300);
        set_age("translation_b_001_002_001_x.wav", 200);
        set_age("translation_c_001_003_001_x.wav", 100);

        // Cap of 800 B keeps two of the three 400 B files: the oldest goes.
        let report = cache.prune_to(Some(800)).await;

        assert_eq!(report.files_removed, 1);
        assert_eq!(report.bytes_freed, 400);
        let usage = cache.usage().await;
        assert_eq!(usage.total_bytes, 800);
        assert!(
            !cache
                .root
                .join("tts/translation_a_001_001_001_x.wav")
                .exists()
        );
        assert!(
            cache
                .root
                .join("tts/translation_b_001_002_001_x.wav")
                .exists()
        );
        assert!(
            cache
                .root
                .join("tts/translation_c_001_003_001_x.wav")
                .exists()
        );
    }

    #[tokio::test]
    async fn prune_removes_only_old_orphan_part_files() {
        let temp_dir = tempfile::tempdir().expect("temp dir should be created");
        let cache = AudioCache::with_root(temp_dir.path().to_path_buf());

        write_media(&cache, "tts", "supertonic_en_001_001.111.0.part", 500).await;
        write_media(&cache, "tts", "supertonic_en_001_002.222.1.part", 500).await;

        // The old leftover is removed; a fresh one may belong to a live render.
        let old = cache.root.join("tts/supertonic_en_001_001.111.0.part");
        std::fs::File::options()
            .write(true)
            .open(&old)
            .expect("file should open")
            .set_modified(SystemTime::now() - Duration::from_secs(3600))
            .expect("mtime should be set");

        // No cap, so only the orphan sweep runs.
        let report = cache.prune_to(None).await;

        assert_eq!(report.orphan_parts_removed, 1);
        assert_eq!(report.bytes_freed, 500);
        assert!(!old.exists());
        assert!(
            cache
                .root
                .join("tts/supertonic_en_001_002.222.1.part")
                .exists()
        );
    }

    #[tokio::test]
    async fn clear_media_removes_everything_but_keeps_other_directories() {
        let temp_dir = tempfile::tempdir().expect("temp dir should be created");
        let cache = AudioCache::with_root(temp_dir.path().to_path_buf());

        write_media(&cache, "audio", "05_001_001.mp3", 100).await;
        write_media(&cache, "tts", "tafsir_supertonic_en_001_002_001_x.wav", 200).await;
        write_media(&cache, "tts", "supertonic_en_001_002.222.1.part", 50).await;
        // The search index shares the cache root but not the media lifecycle.
        write_media(&cache, "index", "quran-index.json", 1024).await;

        let report = cache.clear_media().await;

        assert_eq!(report.files_removed, 3);
        assert_eq!(report.bytes_freed, 350);
        assert_eq!(cache.usage().await.total_bytes, 0);
        assert!(cache.root.join("index/quran-index.json").exists());
    }
}
