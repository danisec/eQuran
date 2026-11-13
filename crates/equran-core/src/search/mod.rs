//! Local full-text search index for the Quran.
//!
//! The index merges the Arabic text, latin transliteration, Indonesian
//! translation, English translation and Indonesian tafsir of every ayah into a
//! single JSON document stored in the user cache directory. Searches run fully
//! offline using a hand rolled BM25 scorer; no extra dependencies are required.

mod build;
mod entry;
mod field;
mod normalize;
mod persist;
mod query;
mod score;

use std::sync::{Arc, OnceLock};
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

use field::FieldIndex;

pub use build::build;
pub use entry::SearchEntry;
pub use persist::{index_path, load_cached, load_or_build};
pub use score::SearchHit;

/// Schema version of the persisted index. Bump this whenever [`SearchEntry`] or
/// [`SearchIndex`] change shape so stale files are rebuilt instead of loading.
pub const INDEX_VERSION: u32 = 1;

/// Number of surahs in the Quran; every build fetches exactly this many.
const SURAH_COUNT: u8 = 114;

/// Maximum number of HTTP requests kept in flight while building the index.
const MAX_CONCURRENCY: usize = 4;

/// BM25 term frequency saturation parameter.
const K1: f32 = 1.2;
/// BM25 document length normalisation parameter.
const B: f32 = 0.75;

/// Maximum length, in characters, of a result snippet.
const SNIPPET_MAX_CHARS: usize = 220;

/// Names of the scored fields, indexed by their position in the scorer.
const FIELD_NAMES: [&str; 5] = ["arab", "latin", "indonesian", "english", "tafsir"];
/// Relative importance of each field. Tafsir is the richest topic source.
const FIELD_WEIGHTS: [f32; 5] = [1.0, 0.9, 1.0, 0.8, 1.2];

/// Returns the current unix timestamp, in seconds.
fn now_unix() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or(0)
}

/// The complete searchable index for the Quran.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SearchIndex {
    /// Schema version; compared against [`INDEX_VERSION`] when loading.
    pub version: u32,
    /// Unix timestamp, in seconds, at which the index was built.
    pub built_at: u64,
    /// Every ayah, sorted by surah then ayah.
    pub entries: Vec<SearchEntry>,
    /// Inverted indexes, built on first query and reused afterwards.
    ///
    /// Rebuilding these for every keystroke would walk all 6236 entries five
    /// times; the search box is debounced but still fires often enough to matter.
    #[serde(skip, default = "field_index_slot")]
    fields: Arc<OnceLock<Vec<FieldIndex>>>,
}

fn field_index_slot() -> Arc<OnceLock<Vec<FieldIndex>>> {
    Arc::new(OnceLock::new())
}

impl PartialEq for SearchIndex {
    fn eq(&self, other: &Self) -> bool {
        self.version == other.version
            && self.built_at == other.built_at
            && self.entries == other.entries
    }
}

impl SearchIndex {
    /// Returns true when the index is older than `max_age_secs` seconds.
    pub fn is_stale(&self, max_age_secs: u64) -> bool {
        now_unix().saturating_sub(self.built_at) > max_age_secs
    }

    /// Number of indexed ayahs.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Returns true when the index holds no ayahs.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

// ---------------------------------------------------------------------------
// Building

#[cfg(test)]
mod tests;
