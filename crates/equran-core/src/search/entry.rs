//! One ayah with its precomputed normalised search blobs.

use serde::{Deserialize, Serialize};

use super::normalize::{normalize_arabic, normalize_indonesian, normalize_latin};

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
    pub(super) fn assemble(
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
    pub(super) fn original_field(&self, field: usize) -> &str {
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
    pub(super) fn norm_field(&self, field: usize) -> &str {
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
