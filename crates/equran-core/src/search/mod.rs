//! Local full-text search index for the Quran.
//!
//! The index merges the Arabic text, latin transliteration, Indonesian
//! translation, English translation and Indonesian tafsir of every ayah into a
//! single JSON document stored in the user cache directory. Searches run fully
//! offline using a hand rolled BM25 scorer; no extra dependencies are required.

use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::OnceLock;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use tokio::fs;
use tokio::sync::Semaphore;
use tokio::task::JoinSet;

use crate::api::client::ApiClient;

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

/// Counter used to make atomic-save temporary files unique within a process.
static PART_COUNTER: AtomicU64 = AtomicU64::new(0);

// ---------------------------------------------------------------------------
// Normalisation
// ---------------------------------------------------------------------------

/// Returns the current unix timestamp, in seconds.
fn now_unix() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or(0)
}

/// Folds whitespace runs into single spaces and trims the result.
fn collapse_whitespace(input: &str) -> String {
    input.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Returns true when the character is an Arabic combining mark (harakat or a
/// Quranic annotation) that should be ignored for matching.
fn is_arabic_mark(ch: char) -> bool {
    matches!(ch,
        '\u{064B}'..='\u{0655}' // fathatan..hamza above (includes madda above)
        | '\u{0670}'            // superscript alef
        | '\u{06D6}'..='\u{06ED}' // Quranic annotation signs
    )
}

/// Returns true when the string contains at least one Arabic codepoint.
pub fn contains_arabic(input: &str) -> bool {
    input.chars().any(|ch| {
        matches!(ch,
            '\u{0600}'..='\u{06FF}'
            | '\u{0750}'..='\u{077F}'
            | '\u{08A0}'..='\u{08FF}'
            | '\u{FB50}'..='\u{FDFF}'
            | '\u{FE70}'..='\u{FEFF}'
        )
    })
}

/// Normalises Arabic text for matching.
///
/// Harakat/diacritics are stripped, tatweel is removed, common letter variants
/// are unified (`أ إ آ ٱ → ا`, `ى → ي`, `ة → ه`, `ؤ → و`, `ئ → ي`) and
/// whitespace is collapsed. The result keeps letters and digits only.
pub fn normalize_arabic(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    for ch in input.chars() {
        if is_arabic_mark(ch) || ch == '\u{0640}' {
            continue;
        }
        let unified = match ch {
            '\u{0623}' | '\u{0625}' | '\u{0622}' | '\u{0671}' => '\u{0627}', // أ إ آ ٱ → ا
            '\u{0649}' => '\u{064A}',                                        // ى → ي
            '\u{0629}' => '\u{0647}',                                        // ة → ه
            '\u{0624}' => '\u{0648}',                                        // ؤ → و
            '\u{0626}' => '\u{064A}',                                        // ئ → ي
            other => other,
        };
        out.push(unified);
    }
    collapse_whitespace(&out)
}

/// Folds a single latin character to its ascii-ish base, if it has one.
fn fold_latin(ch: char) -> Option<&'static str> {
    let folded = match ch {
        'à' | 'á' | 'â' | 'ã' | 'ä' | 'å' | 'ā' | 'ă' | 'ą' | 'ǎ' | 'ǟ' | 'ǡ' | 'ǻ' | 'ȁ' | 'ȃ'
        | 'ȧ' | 'ḁ' | 'ạ' | 'ả' | 'ấ' | 'ầ' | 'ẩ' | 'ẫ' | 'ậ' | 'ắ' | 'ằ' | 'ẳ' | 'ẵ' | 'ặ' => {
            "a"
        }
        'æ' => "ae",
        'ƀ' | 'ƃ' | 'ƅ' | 'ḃ' | 'ḅ' | 'ḇ' => "b",
        'ç' | 'ć' | 'ĉ' | 'ċ' | 'č' | 'ƈ' | 'ḉ' => "c",
        'ð' | 'ď' | 'đ' | 'ƌ' | 'ǆ' | 'ǳ' | 'ḋ' | 'ḍ' | 'ḏ' | 'ḑ' | 'ḓ' => "d",
        'è' | 'é' | 'ê' | 'ë' | 'ē' | 'ĕ' | 'ė' | 'ę' | 'ě' | 'ȅ' | 'ȇ' | 'ȩ' | 'ḕ' | 'ḗ' | 'ḙ'
        | 'ḛ' | 'ḝ' | 'ẹ' | 'ẻ' | 'ẽ' | 'ế' | 'ề' | 'ể' | 'ễ' | 'ệ' => "e",
        'ƒ' | 'ḟ' => "f",
        'ĝ' | 'ğ' | 'ġ' | 'ģ' | 'ǧ' | 'ǵ' | 'ḡ' => "g",
        'ĥ' | 'ħ' | 'ƕ' | 'ȟ' | 'ḣ' | 'ḥ' | 'ḧ' | 'ḩ' | 'ḫ' | 'ẖ' => "h",
        'ì' | 'í' | 'î' | 'ï' | 'ĩ' | 'ī' | 'ĭ' | 'į' | 'ı' | 'ǐ' | 'ȉ' | 'ȋ' | 'ḭ' | 'ḯ' | 'ỉ'
        | 'ị' => "i",
        'ĵ' | 'ǰ' => "j",
        'ķ' | 'ĸ' | 'ƙ' | 'ǩ' | 'ḱ' | 'ḳ' | 'ḵ' => "k",
        'ĺ' | 'ļ' | 'ľ' | 'ł' | 'ƚ' | 'ƛ' | 'ǉ' | 'ḷ' | 'ḹ' | 'ḻ' | 'ḽ' => "l",
        'ḿ' | 'ṁ' | 'ṃ' => "m",
        'ñ' | 'ń' | 'ņ' | 'ň' | 'ŋ' | 'ƞ' | 'ǌ' | 'ǹ' | 'ṅ' | 'ṇ' | 'ṉ' | 'ṋ' => {
            "n"
        }
        'ò' | 'ó' | 'ô' | 'õ' | 'ö' | 'ō' | 'ŏ' | 'ő' | 'ơ' | 'ǒ' | 'ǫ' | 'ǭ' | 'ǿ' | 'ȍ' | 'ȏ'
        | 'ȫ' | 'ȭ' | 'ȯ' | 'ȱ' | 'ṍ' | 'ṏ' | 'ṑ' | 'ṓ' | 'ọ' | 'ỏ' | 'ố' | 'ồ' | 'ổ' | 'ỗ'
        | 'ộ' | 'ớ' | 'ờ' | 'ở' | 'ỡ' | 'ợ' => "o",
        'œ' => "oe",
        'ƥ' | 'ṕ' | 'ṗ' => "p",
        'ƣ' => "q",
        'ŕ' | 'ŗ' | 'ř' | 'ȑ' | 'ȓ' | 'ṙ' | 'ṛ' | 'ṝ' | 'ṟ' => "r",
        'ś' | 'ŝ' | 'ş' | 'š' | 'ƨ' | 'ƪ' | 'ș' | 'ṡ' | 'ṣ' | 'ṥ' | 'ṧ' | 'ṩ' => {
            "s"
        }
        'ß' => "ss",
        'ţ' | 'ť' | 'ŧ' | 'ƫ' | 'ƭ' | 'ț' | 'ṫ' | 'ṭ' | 'ṯ' | 'ṱ' | 'ẗ' => "t",
        'þ' => "th",
        'ù' | 'ú' | 'û' | 'ü' | 'ũ' | 'ū' | 'ŭ' | 'ů' | 'ű' | 'ų' | 'ư' | 'ǔ' | 'ǖ' | 'ǘ' | 'ǚ'
        | 'ǜ' | 'ȕ' | 'ȗ' | 'ṳ' | 'ṵ' | 'ṷ' | 'ṹ' | 'ṻ' | 'ụ' | 'ủ' | 'ứ' | 'ừ' | 'ử' | 'ữ'
        | 'ự' => "u",
        'ṽ' | 'ṿ' => "v",
        'ŵ' | 'ẁ' | 'ẃ' | 'ẅ' | 'ẇ' | 'ẉ' | 'ẘ' => "w",
        'ẋ' | 'ẍ' => "x",
        'ý' | 'ÿ' | 'ŷ' | 'ƴ' | 'ȳ' | 'ẏ' | 'ẙ' | 'ỳ' | 'ỵ' | 'ỷ' | 'ỹ' => "y",
        'ź' | 'ż' | 'ž' | 'ƶ' | 'ƹ' | 'ƺ' | 'ƽ' | 'ẑ' | 'ẓ' | 'ẕ' => "z",
        _ => return None,
    };
    Some(folded)
}

/// Returns true for the apostrophe-like marks that are dropped entirely so a
/// word such as `Qur'an` matches `quran`.
fn is_apostrophe(ch: char) -> bool {
    matches!(
        ch,
        '\'' | '\u{02BC}' | '\u{02BE}' | '\u{02BF}' | '\u{2018}' | '\u{2019}'
    )
}

/// Normalises latin (transliteration/translation) text for matching.
///
/// Text is lowercased, combining diacritics in `U+0300..=U+036F` are dropped,
/// precomposed accents are folded to their ascii base (`ā → a`, `ḥ → h`,
/// `ṣ → s`, ...), apostrophes are removed and punctuation becomes whitespace.
pub fn normalize_latin(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    for ch in input.chars() {
        if ('\u{0300}'..='\u{036F}').contains(&ch) || is_apostrophe(ch) {
            continue;
        }
        if let Some(folded) = fold_latin(ch) {
            out.push_str(folded);
            continue;
        }
        if ch.is_ascii_alphanumeric() {
            out.push(ch.to_ascii_lowercase());
            continue;
        }
        if ch.is_alphabetic() {
            for lowered in ch.to_lowercase() {
                if ('\u{0300}'..='\u{036F}').contains(&lowered) {
                    continue;
                }
                if let Some(folded) = fold_latin(lowered) {
                    out.push_str(folded);
                } else if lowered.is_alphanumeric() {
                    out.push(lowered);
                } else {
                    out.push(' ');
                }
            }
            continue;
        }
        out.push(' ');
    }
    collapse_whitespace(&out)
}

/// Alias of [`normalize_latin`] for Indonesian text.
pub fn normalize_indonesian(input: &str) -> String {
    normalize_latin(input)
}

/// Tokenises free text into normalised terms.
///
/// Each whitespace-delimited chunk is routed to the Arabic or latin normaliser
/// based on its script, then split again so punctuation-joined fragments become
/// separate terms.
pub fn tokenize(input: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    for raw in input.split_whitespace() {
        let normalized = if contains_arabic(raw) {
            normalize_arabic(raw)
        } else {
            normalize_latin(raw)
        };
        for piece in normalized.split_whitespace() {
            tokens.push(piece.to_owned());
        }
    }
    tokens
}

/// Maps the API revelation label to the canonical Arabic-derived spelling.
fn revelation_label(raw: &str) -> String {
    if raw.to_lowercase().contains("madin") {
        "Madaniyah".to_owned()
    } else {
        "Makkiyah".to_owned()
    }
}

// ---------------------------------------------------------------------------
// Index data
// ---------------------------------------------------------------------------

/// A single ayah together with its precomputed normalised search blobs.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SearchEntry {
    /// Surah number, 1..=114.
    pub surah: u8,
    /// Ayah number within the surah.
    pub ayah: u16,
    /// Latin name of the surah (for example `Al-Fatihah`).
    pub surah_name_latin: String,
    /// `Makkiyah` or `Madaniyah`.
    pub revelation: String,
    /// Original Arabic text.
    pub teks_arab: String,
    /// Original latin transliteration.
    pub teks_latin: String,
    /// Original Indonesian translation.
    pub teks_indonesia: String,
    /// English translation, when available.
    pub teks_english: Option<String>,
    /// Indonesian tafsir, when available.
    pub tafsir: Option<String>,
    /// Normalised Arabic blob.
    pub arab_norm: String,
    /// Normalised latin transliteration blob.
    pub latin_norm: String,
    /// Normalised Indonesian blob.
    pub indo_norm: String,
    /// Normalised English blob (empty when unavailable).
    pub eng_norm: String,
    /// Normalised tafsir blob (empty when unavailable).
    pub tafsir_norm: String,
}

impl SearchEntry {
    /// Builds an entry, precomputing every normalised blob from its source text.
    #[allow(clippy::too_many_arguments)]
    fn assemble(
        surah: u8,
        ayah: u16,
        surah_name_latin: String,
        revelation: String,
        teks_arab: String,
        teks_latin: String,
        teks_indonesia: String,
        teks_english: Option<String>,
        tafsir: Option<String>,
    ) -> Self {
        let arab_norm = normalize_arabic(&teks_arab);
        let latin_norm = normalize_latin(&teks_latin);
        let indo_norm = normalize_indonesian(&teks_indonesia);
        let eng_norm = teks_english
            .as_deref()
            .map(normalize_latin)
            .unwrap_or_default();
        let tafsir_norm = tafsir
            .as_deref()
            .map(normalize_indonesian)
            .unwrap_or_default();
        Self {
            surah,
            ayah,
            surah_name_latin,
            revelation,
            teks_arab,
            teks_latin,
            teks_indonesia,
            teks_english,
            tafsir,
            arab_norm,
            latin_norm,
            indo_norm,
            eng_norm,
            tafsir_norm,
        }
    }

    /// Original (unnormalised) text of the field at position `field`.
    fn original_field(&self, field: usize) -> &str {
        match field {
            0 => &self.teks_arab,
            1 => &self.teks_latin,
            2 => &self.teks_indonesia,
            3 => self.teks_english.as_deref().unwrap_or(""),
            4 => self.tafsir.as_deref().unwrap_or(""),
            _ => &self.teks_indonesia,
        }
    }

    /// Normalised blob of the field at position `field`.
    fn norm_field(&self, field: usize) -> &str {
        match field {
            0 => &self.arab_norm,
            1 => &self.latin_norm,
            2 => &self.indo_norm,
            3 => &self.eng_norm,
            4 => &self.tafsir_norm,
            _ => "",
        }
    }
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
// ---------------------------------------------------------------------------

/// Fetches and assembles every entry for a single surah.
///
/// The Indonesian text is mandatory; English and tafsir failures are tolerated
/// and leave those fields empty so the index stays usable offline.
async fn fetch_surah_entries(api: &ApiClient, number: u8) -> Result<Vec<SearchEntry>> {
    let surah = api
        .fetch_surah(number)
        .await
        .with_context(|| format!("failed to fetch surah {number} for the search index"))?;
    let surah_name_latin = surah.nama_latin.clone();
    let revelation = revelation_label(&surah.tempat_turun);

    let english = api.fetch_surah_english(number).await.ok();
    let tafsir = api.fetch_tafsir(number).await.ok();

    let english_map: HashMap<u16, String> = english
        .map(|surah| {
            surah
                .ayahs
                .into_iter()
                .map(|ayah| (ayah.number_in_surah, ayah.text_english))
                .collect()
        })
        .unwrap_or_default();
    let tafsir_map: HashMap<u16, String> = tafsir
        .map(|surah| {
            surah
                .tafsir
                .into_iter()
                .map(|ayah| (ayah.ayat, ayah.teks))
                .collect()
        })
        .unwrap_or_default();

    let mut entries = Vec::with_capacity(surah.ayat.len());
    for ayat in surah.ayat {
        let teks_english = english_map.get(&ayat.nomor_ayat).cloned();
        let tafsir_text = tafsir_map.get(&ayat.nomor_ayat).cloned();
        entries.push(SearchEntry::assemble(
            number,
            ayat.nomor_ayat,
            surah_name_latin.clone(),
            revelation.clone(),
            ayat.teks_arab,
            ayat.teks_latin,
            ayat.teks_indonesia,
            teks_english,
            tafsir_text,
        ));
    }
    Ok(entries)
}

/// Builds the whole index from the network.
///
/// All 114 surahs are fetched with at most [`MAX_CONCURRENCY`] requests in
/// flight. `progress(done, total)` is called once per completed surah.
pub async fn build(
    api: &ApiClient,
    progress: impl Fn(usize, usize) + Send + Sync,
) -> Result<SearchIndex> {
    let semaphore = Arc::new(Semaphore::new(MAX_CONCURRENCY));
    let mut tasks: JoinSet<Result<Vec<SearchEntry>>> = JoinSet::new();
    for number in 1..=SURAH_COUNT {
        let api = api.clone();
        let semaphore = Arc::clone(&semaphore);
        tasks.spawn(async move {
            let permit = semaphore
                .acquire_owned()
                .await
                .context("search index build semaphore closed")?;
            let entries = fetch_surah_entries(&api, number).await?;
            drop(permit);
            Ok(entries)
        });
    }

    let total = SURAH_COUNT as usize;
    let mut done = 0usize;
    let mut entries = Vec::new();
    while let Some(joined) = tasks.join_next().await {
        let surah_entries = joined.context("search index worker task panicked")??;
        entries.extend(surah_entries);
        done += 1;
        progress(done, total);
    }
    entries.sort_by_key(|entry| (entry.surah, entry.ayah));
    Ok(SearchIndex {
        version: INDEX_VERSION,
        built_at: now_unix(),
        entries,
        fields: field_index_slot(),
    })
}

// ---------------------------------------------------------------------------
// Persistence
// ---------------------------------------------------------------------------

/// Returns the path of the on-disk index:
/// `<cache_dir>/equran-cli/index/quran-index.json`.
pub fn index_path() -> Result<PathBuf> {
    let cache_root = dirs::cache_dir()
        .context("failed to resolve user cache directory")?
        .join("equran-cli")
        .join("index");
    Ok(cache_root.join("quran-index.json"))
}

/// Atomically writes the index to `path` via a unique `.part` file and rename.
async fn save_to(path: &Path, index: &SearchIndex) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .await
            .with_context(|| format!("failed to create {}", parent.display()))?;
    }
    let json = serde_json::to_string(index).context("failed to serialise the search index")?;

    let counter = PART_COUNTER.fetch_add(1, Ordering::Relaxed);
    let file_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("quran-index.json");
    let part_name = format!("{file_name}.part.{}.{}", std::process::id(), counter);
    let mut part_path = path.to_path_buf();
    part_path.set_file_name(part_name);

    fs::write(&part_path, json.as_bytes())
        .await
        .with_context(|| format!("failed to write {}", part_path.display()))?;
    fs::rename(&part_path, path)
        .await
        .with_context(|| format!("failed to move {} into place", part_path.display()))?;
    Ok(())
}

/// Loads an index from `path`.
///
/// A missing, unreadable or corrupt file yields `Ok(None)` so the caller can
/// rebuild instead of failing outright.
async fn load_from(path: &Path) -> Result<Option<SearchIndex>> {
    let bytes = match fs::read(path).await {
        Ok(bytes) => bytes,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(error) => {
            return Err(error).with_context(|| format!("failed to read {}", path.display()));
        }
    };
    match serde_json::from_slice::<SearchIndex>(&bytes) {
        Ok(index) => Ok(Some(index)),
        Err(_) => Ok(None),
    }
}

/// Loads the persisted index from disk when it exists and matches
/// [`INDEX_VERSION`].
///
/// Returns `Ok(None)` when no valid cached index is available; callers can then
/// fall back to [`load_or_build`] or [`build`].
pub async fn load_cached() -> Result<Option<SearchIndex>> {
    let path = index_path()?;
    match load_from(&path).await? {
        Some(index) if index.version == INDEX_VERSION => Ok(Some(index)),
        _ => Ok(None),
    }
}

/// Loads the cached index when it is valid, otherwise builds and saves it.
///
/// `rebuild` forces a fresh build even when a valid file is present.
pub async fn load_or_build(
    api: &ApiClient,
    rebuild: bool,
    progress: impl Fn(usize, usize) + Send + Sync,
) -> Result<SearchIndex> {
    let path = index_path()?;
    if !rebuild
        && let Some(index) = load_from(&path).await?
        && index.version == INDEX_VERSION
    {
        return Ok(index);
    }
    let index = build(api, progress).await?;
    save_to(&path, &index).await?;
    Ok(index)
}

// ---------------------------------------------------------------------------
// Query and scoring
// ---------------------------------------------------------------------------

/// An inverted index and statistics for a single scored field.
#[derive(Debug)]
struct FieldIndex {
    /// term -> (entry index, term frequency)
    postings: HashMap<String, Vec<(usize, u32)>>,
    /// Token count of each entry's field.
    doc_len: Vec<usize>,
    /// Average token count across entries that have this field.
    avg_len: f32,
    /// Number of entries with a non-empty value for this field.
    total_docs: usize,
    /// BM25 field weight.
    weight: f32,
}

impl FieldIndex {
    /// Returns every indexed term that starts with `token`.
    ///
    /// Exact matches come first so an exact hit is never diluted by a longer
    /// one. Terms shorter than the token are skipped: a reader typing
    /// "bismillah" should not match a document that only contains "bis".
    fn terms_with_prefix(&self, token: &str) -> Vec<&str> {
        let mut exact = None;
        let mut prefixed = Vec::new();
        for term in self.postings.keys() {
            if term == token {
                exact = Some(term.as_str());
                break;
            }
            if term.len() > token.len() && term.starts_with(token) {
                prefixed.push(term.as_str());
            }
        }
        let mut terms = Vec::with_capacity(prefixed.len() + 1);
        terms.extend(exact);
        terms.extend(prefixed);
        terms
    }
}

/// Builds the inverted index for one field.
fn build_field_index(entries: &[SearchEntry], field: usize, weight: f32) -> FieldIndex {
    let mut postings: HashMap<String, Vec<(usize, u32)>> = HashMap::new();
    let mut doc_len = vec![0usize; entries.len()];
    let mut total_len = 0usize;
    let mut total_docs = 0usize;

    for (idx, entry) in entries.iter().enumerate() {
        let text = entry.norm_field(field);
        let mut term_freq: HashMap<&str, u32> = HashMap::new();
        let mut count = 0usize;
        for term in text.split_whitespace() {
            *term_freq.entry(term).or_insert(0) += 1;
            count += 1;
        }
        doc_len[idx] = count;
        if count > 0 {
            total_docs += 1;
            total_len += count;
        }
        for (term, freq) in term_freq {
            postings
                .entry(term.to_owned())
                .or_default()
                .push((idx, freq));
        }
    }

    let avg_len = if total_docs > 0 {
        total_len as f32 / total_docs as f32
    } else {
        0.0
    };
    FieldIndex {
        postings,
        doc_len,
        avg_len,
        total_docs,
        weight,
    }
}

/// Returns the unique normalised query tokens, preserving order.
fn unique_tokens(query: &str) -> Vec<String> {
    let mut seen = HashSet::new();
    let mut tokens = Vec::new();
    for token in tokenize(query) {
        if seen.insert(token.clone()) {
            tokens.push(token);
        }
    }
    tokens
}

/// Normalises a single word using the script-appropriate normaliser.
fn normalize_word(word: &str) -> String {
    if contains_arabic(word) {
        normalize_arabic(word)
    } else {
        normalize_latin(word)
    }
}

/// Returns true when the normalised `word` matches any query token.
fn word_matches_token(word: &str, tokens: &[String]) -> bool {
    let normalized = normalize_word(word);
    if normalized.is_empty() {
        return false;
    }
    tokens.iter().any(|token| {
        normalized == *token
            || normalized.starts_with(token.as_str())
            || token.starts_with(normalized.as_str())
    })
}

/// Finds the character index of the first word that matches a query token.
fn find_match_center(text: &str, tokens: &[String]) -> Option<usize> {
    let mut word = String::new();
    let mut word_start = 0usize;
    for (char_pos, ch) in text.chars().enumerate() {
        if ch.is_whitespace() {
            if !word.is_empty() {
                if word_matches_token(&word, tokens) {
                    return Some(word_start);
                }
                word.clear();
            }
        } else {
            if word.is_empty() {
                word_start = char_pos;
            }
            word.push(ch);
        }
    }
    if !word.is_empty() && word_matches_token(&word, tokens) {
        return Some(word_start);
    }
    None
}

/// Builds a snippet of at most [`SNIPPET_MAX_CHARS`] characters, roughly
/// centred on the first matching word.
fn make_snippet(text: &str, tokens: &[String]) -> String {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return String::new();
    }
    let chars: Vec<char> = trimmed.chars().collect();
    if chars.len() <= SNIPPET_MAX_CHARS {
        return trimmed.to_owned();
    }

    let center = find_match_center(trimmed, tokens).unwrap_or(0);
    let half = SNIPPET_MAX_CHARS / 2;
    let start = center
        .saturating_sub(half)
        .min(chars.len().saturating_sub(SNIPPET_MAX_CHARS));
    let end = (start + SNIPPET_MAX_CHARS).min(chars.len());

    let mut budget = SNIPPET_MAX_CHARS;
    let mut snippet = String::new();
    if start > 0 {
        snippet.push('…');
        budget -= 1;
    }
    let take = (end - start).min(budget);
    snippet.extend(chars[start..start + take].iter());
    if end < chars.len() && snippet.chars().count() < SNIPPET_MAX_CHARS {
        snippet.push('…');
    }
    snippet.trim().to_owned()
}

/// A single search result.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchHit {
    /// Surah number, 1..=114.
    pub surah: u8,
    /// Ayah number within the surah.
    pub ayah: u16,
    /// Latin name of the surah.
    pub surah_name_latin: String,
    /// Weighted BM25 score.
    pub score: f32,
    /// Short excerpt from the highest-scoring field.
    pub snippet: String,
    /// Name of the field that produced the snippet.
    pub matched_field: String,
}

impl SearchIndex {
    /// Searches the index for `query` and returns at most `limit` hits.
    ///
    /// Every query token must match the entry (AND semantics); results are
    /// ranked by the total weighted BM25 score across all fields.
    pub fn search(&self, query: &str, limit: usize) -> Vec<SearchHit> {
        if limit == 0 {
            return Vec::new();
        }
        let tokens = unique_tokens(query);
        if tokens.is_empty() {
            return Vec::new();
        }

        let count = self.entries.len();
        let fields = self.fields.get_or_init(|| {
            (0..FIELD_NAMES.len())
                .map(|field| build_field_index(&self.entries, field, FIELD_WEIGHTS[field]))
                .collect()
        });

        let mut scores = vec![0f32; count];
        let mut allowed = vec![true; count];
        // (best per-token weighted score, field index) for snippet selection.
        let mut best: Vec<(f32, usize)> = vec![(0.0, 2); count];

        for token in &tokens {
            let mut matched = vec![false; count];
            let mut any = false;
            for (field_pos, field) in fields.iter().enumerate() {
                if field.total_docs == 0 || field.avg_len <= 0.0 {
                    continue;
                }
                // Exact postings first, then prefix postings. A reader types
                // "bismillah" or "rahman" and expects to reach "bismillāhir"
                // and "raḥmān"; requiring the full transliteration would make
                // the most obvious queries miss.
                let mut expanded: Vec<&(usize, u32)> = Vec::new();
                for term in field.terms_with_prefix(token) {
                    if let Some(postings) = field.postings.get(term) {
                        expanded.extend(postings.iter());
                    }
                }
                if expanded.is_empty() {
                    continue;
                }
                any = true;
                let df = expanded.len() as f32;
                let total_docs = field.total_docs as f32;
                let idf = (1.0 + (total_docs - df + 0.5) / (df + 0.5)).ln();
                for &(idx, tf) in expanded {
                    let tf = tf as f32;
                    let doc_len = field.doc_len[idx] as f32;
                    let denom = tf + K1 * (1.0 - B + B * doc_len / field.avg_len);
                    let score = idf * (tf * (K1 + 1.0)) / denom * field.weight;
                    scores[idx] += score;
                    matched[idx] = true;
                    if score > best[idx].0 {
                        best[idx] = (score, field_pos);
                    }
                }
            }
            if !any {
                return Vec::new();
            }
            for (idx, did_match) in matched.iter().enumerate() {
                if !did_match {
                    allowed[idx] = false;
                }
            }
        }

        let mut ranked: Vec<(usize, f32)> = (0..count)
            .filter(|&idx| allowed[idx])
            .map(|idx| (idx, scores[idx]))
            .collect();
        ranked.sort_by(|a, b| b.1.total_cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
        ranked.truncate(limit);

        ranked
            .into_iter()
            .map(|(idx, score)| {
                let entry = &self.entries[idx];
                let field_pos = best[idx].1;
                SearchHit {
                    surah: entry.surah,
                    ayah: entry.ayah,
                    surah_name_latin: entry.surah_name_latin.clone(),
                    score,
                    snippet: make_snippet(entry.original_field(field_pos), &tokens),
                    matched_field: FIELD_NAMES[field_pos].to_owned(),
                }
            })
            .collect()
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn entry(
        surah: u8,
        ayah: u16,
        indo: &str,
        arab: &str,
        latin: &str,
        english: Option<&str>,
        tafsir: Option<&str>,
    ) -> SearchEntry {
        SearchEntry::assemble(
            surah,
            ayah,
            format!("Surah {surah}"),
            "Makkiyah".to_owned(),
            arab.to_owned(),
            latin.to_owned(),
            indo.to_owned(),
            english.map(str::to_owned),
            tafsir.map(str::to_owned),
        )
    }

    fn index_from(entries: Vec<SearchEntry>) -> SearchIndex {
        SearchIndex {
            version: INDEX_VERSION,
            built_at: now_unix(),
            entries,
            fields: field_index_slot(),
        }
    }

    #[test]
    fn arabic_with_and_without_harakat_normalise_equal() {
        assert_eq!(normalize_arabic("الرَّحْمٰنِ"), normalize_arabic("الرحمن"));
        assert_eq!(normalize_arabic("الرَّحْمٰنِ"), "الرحمن");
    }

    #[test]
    fn arabic_unifies_letter_variants_and_tatweel() {
        assert_eq!(normalize_arabic("أإآٱ"), "اااا");
        assert_eq!(normalize_arabic("ىةؤئ"), "يهوي");
        assert_eq!(normalize_arabic("مـحـمـد"), "محمد");
        assert_eq!(normalize_arabic("  a  b  "), "a b");
    }

    #[test]
    fn latin_diacritics_fold_to_ascii() {
        assert_eq!(normalize_latin("aḥmad"), "ahmad");
        assert_eq!(normalize_latin("ahmad"), "ahmad");
        assert_eq!(normalize_latin("ā ī ū ṣ ḍ ṭ ẓ"), "a i u s d t z");
    }

    #[test]
    fn prefix_token_reaches_longer_word() {
        // "bismillah" must find the transliteration "bismillāhir", and
        // "rahman" must find "raḥmān", or the obvious queries return nothing.
        let index = index_from(vec![entry(
            1,
            1,
            "",
            "",
            "Bismillāhir-raḥmānir-raḥīm",
            None,
            None,
        )]);

        let hits = index.search("bismillah", 10);
        assert_eq!(hits.len(), 1, "prefix should match a longer indexed term");

        let hits = index.search("rahman", 10);
        assert_eq!(hits.len(), 1);

        // A token longer than the indexed term must not match it.
        let hits = index.search("bismillahirahmanirrahim", 10);
        assert!(
            hits.is_empty(),
            "a longer token must not match a shorter term"
        );
    }

    #[test]
    fn fields_are_built_once_per_index() {
        let index = index_from(vec![entry(1, 1, "allah maha pengasih", "", "", None, None)]);
        assert!(index.fields.get().is_none(), "indexes are built lazily");

        let _ = index.search("allah", 5);
        assert!(index.fields.get().is_some(), "the first query builds them");

        // A second query reuses the cached indexes rather than rebuilding.
        let first = index.fields.get().map(Vec::len);
        let _ = index.search("pengasih", 5);
        assert_eq!(index.fields.get().map(Vec::len), first);
    }

    #[test]
    fn latin_strips_punctuation_and_apostrophes() {
        assert_eq!(normalize_latin("Qur'an!"), "quran");
        assert_eq!(normalize_latin("al-ḥamdu, li-llāh"), "al hamdu li llah");
    }

    #[test]
    fn indonesian_is_alias_of_latin() {
        assert_eq!(
            normalize_indonesian("Muḥammad"),
            normalize_latin("Muḥammad")
        );
    }

    #[test]
    fn tokenize_routes_by_script_and_splits_punctuation() {
        assert_eq!(
            tokenize("Bismillah, al-Rahman!"),
            vec!["bismillah", "al", "rahman"]
        );
        assert_eq!(tokenize("الْحَمْدُ لِلَّهِ"), vec!["الحمد", "لله"]);
        assert!(contains_arabic("مرحبا"));
        assert!(!contains_arabic("hello"));
    }

    #[test]
    fn search_requires_all_query_tokens() {
        let index = index_from(vec![
            entry(1, 1, "allah maha pengasih", "", "", None, None),
            entry(2, 1, "allah maha penyayang", "", "", None, None),
        ]);

        let hits = index.search("allah", 10);
        assert_eq!(hits.len(), 2);

        let hits = index.search("allah pengasih", 10);
        assert_eq!(hits.len(), 1);
        assert_eq!((hits[0].surah, hits[0].ayah), (1, 1));

        assert!(index.search("allah pengasih tidakada", 10).is_empty());
        assert!(index.search("tidakada", 10).is_empty());
        assert!(index.search("", 10).is_empty());
    }

    #[test]
    fn bm25_ranks_repeated_term_first() {
        let long = "allah ".to_owned() + &"firman ".repeat(60);
        let index = index_from(vec![
            entry(1, 1, "allah allah allah maha pengasih", "", "", None, None),
            entry(2, 1, &long, "", "", None, None),
        ]);
        let hits = index.search("allah", 10);
        assert_eq!(hits.len(), 2);
        assert_eq!(hits[0].surah, 1, "higher term frequency should rank first");
        assert!(hits[0].score >= hits[1].score);
    }

    #[test]
    fn snippet_is_truncated_and_centred() {
        let indo = format!("{}allah{}", "kata ".repeat(80), " akhir".repeat(80));
        let index = index_from(vec![entry(1, 1, &indo, "", "", None, None)]);
        let hits = index.search("allah", 10);
        assert_eq!(hits.len(), 1);
        assert!(
            hits[0].snippet.chars().count() <= SNIPPET_MAX_CHARS,
            "snippet must fit the limit"
        );
        assert!(hits[0].snippet.contains("allah"));
    }

    #[test]
    fn matched_field_reports_tafsir() {
        let index = index_from(vec![entry(
            1,
            1,
            "terjemahan biasa",
            "",
            "",
            None,
            Some("penjelasan panjang tentang rahmat dan kasih sayang"),
        )]);
        let hits = index.search("rahmat", 10);
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].matched_field, "tafsir");
        assert!(hits[0].snippet.contains("rahmat"));
    }

    #[test]
    fn index_json_round_trip() {
        let index = index_from(vec![
            entry(
                1,
                1,
                "allah maha pengasih",
                "بسم الله",
                "bismillah",
                Some("In the name of Allah"),
                Some("tafsir pertama"),
            ),
            entry(
                1,
                2,
                "allah maha penyayang",
                "الحمد لله",
                "alhamdulillah",
                None,
                None,
            ),
        ]);
        let json = serde_json::to_string(&index).expect("serialise");
        let restored: SearchIndex = serde_json::from_str(&json).expect("deserialise");
        assert_eq!(index, restored);
    }

    #[test]
    fn is_stale_respects_timestamp() {
        let mut index = index_from(Vec::new());
        index.built_at = now_unix();
        assert!(!index.is_stale(3600));
        index.built_at = now_unix().saturating_sub(7200);
        assert!(index.is_stale(3600));
    }

    #[test]
    fn index_path_follows_cache_convention() {
        if let Ok(path) = index_path() {
            assert!(path.ends_with("equran-cli/index/quran-index.json"));
        }
    }

    #[tokio::test]
    async fn save_and_load_round_trip_on_disk() {
        let dir = tempfile::tempdir().expect("temp dir");
        let path = dir.path().join("nested").join("quran-index.json");
        let index = index_from(vec![entry(1, 1, "allah", "", "", None, None)]);
        save_to(&path, &index).await.expect("save");
        let loaded = load_from(&path).await.expect("load").expect("present");
        assert_eq!(index, loaded);
        assert!(
            load_from(&dir.path().join("missing.json"))
                .await
                .expect("load")
                .is_none()
        );
    }
}
