use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct ApiResponse<T> {
    pub code: u16,
    pub message: String,
    pub data: T,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct SurahListItem {
    pub nomor: u8,
    pub nama: String,
    #[serde(rename = "namaLatin")]
    pub nama_latin: String,
    #[serde(rename = "jumlahAyat")]
    pub jumlah_ayat: u16,
    #[serde(rename = "tempatTurun")]
    pub tempat_turun: String,
    pub arti: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Surah {
    pub nomor: u8,
    pub nama: String,
    #[serde(rename = "namaLatin")]
    pub nama_latin: String,
    #[serde(rename = "jumlahAyat")]
    pub jumlah_ayat: u16,
    #[serde(rename = "tempatTurun")]
    pub tempat_turun: String,
    pub arti: String,
    pub deskripsi: String,
    #[serde(rename = "audioFull")]
    pub audio_full: HashMap<String, String>,
    pub ayat: Vec<Ayat>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Ayat {
    #[serde(rename = "nomorAyat")]
    pub nomor_ayat: u16,
    #[serde(rename = "teksArab")]
    pub teks_arab: String,
    #[serde(rename = "teksLatin")]
    pub teks_latin: String,
    #[serde(rename = "teksIndonesia")]
    pub teks_indonesia: String,
    pub audio: HashMap<String, String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct EnglishSurah {
    pub ayahs: Vec<EnglishAyah>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct EnglishAyah {
    #[serde(rename = "numberInSurah")]
    pub number_in_surah: u16,
    #[serde(rename = "textEnglish")]
    pub text_english: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct TafsirSurah {
    pub nomor: u8,
    pub nama: String,
    #[serde(rename = "namaLatin")]
    pub nama_latin: String,
    #[serde(rename = "jumlahAyat")]
    pub jumlah_ayat: u16,
    #[serde(rename = "tempatTurun")]
    pub tempat_turun: String,
    pub arti: String,
    pub deskripsi: String,
    pub tafsir: Vec<TafsirAyah>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct TafsirAyah {
    pub ayat: u16,
    pub teks: String,
}

#[derive(Debug, Deserialize)]
pub struct QuranApiIdSurah {
    pub number: u8,
    #[serde(rename = "numberOfVerses")]
    pub number_of_verses: u16,
    pub name: QuranApiIdSurahName,
    pub revelation: QuranApiIdRevelation,
    pub tafsir: QuranApiIdSurahTafsir,
    pub verses: Vec<QuranApiIdVerse>,
}

#[derive(Debug, Deserialize)]
pub struct QuranApiIdSurahName {
    pub short: String,
    pub transliteration: QuranApiIdLocalizedText,
    pub translation: QuranApiIdLocalizedText,
}

#[derive(Debug, Deserialize)]
pub struct QuranApiIdLocalizedText {
    pub id: String,
}

#[derive(Debug, Deserialize)]
pub struct QuranApiIdRevelation {
    pub id: String,
}

#[derive(Debug, Deserialize)]
pub struct QuranApiIdSurahTafsir {
    pub id: String,
}

#[derive(Debug, Deserialize)]
pub struct QuranApiIdVerse {
    pub number: QuranApiIdVerseNumber,
    pub tafsir: QuranApiIdVerseTafsir,
}

#[derive(Debug, Deserialize)]
pub struct QuranApiIdVerseNumber {
    #[serde(rename = "inSurah")]
    pub in_surah: u16,
}

#[derive(Debug, Deserialize)]
pub struct QuranApiIdVerseTafsir {
    pub id: QuranApiIdVerseTafsirId,
}

#[derive(Debug, Deserialize)]
pub struct QuranApiIdVerseTafsirId {
    pub short: String,
    pub long: String,
}

impl From<QuranApiIdSurah> for TafsirSurah {
    fn from(value: QuranApiIdSurah) -> Self {
        Self {
            nomor: value.number,
            nama: value.name.short,
            nama_latin: value.name.transliteration.id,
            jumlah_ayat: value.number_of_verses,
            tempat_turun: value.revelation.id,
            arti: value.name.translation.id,
            deskripsi: value.tafsir.id,
            tafsir: value
                .verses
                .into_iter()
                .map(|verse| {
                    let long = verse.tafsir.id.long.trim();
                    let short = verse.tafsir.id.short.trim();
                    TafsirAyah {
                        ayat: verse.number.in_surah,
                        teks: if long.is_empty() {
                            short.to_owned()
                        } else {
                            long.to_owned()
                        },
                    }
                })
                .collect(),
        }
    }
}

impl EnglishSurah {
    pub fn translation_for_ayah(&self, ayah_number: u16) -> Option<&str> {
        self.ayahs
            .iter()
            .find(|ayah| ayah.number_in_surah == ayah_number)
            .map(|ayah| ayah.text_english.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::{ApiResponse, QuranApiIdSurah, TafsirSurah};

    #[test]
    fn parses_tafsir_response() {
        let json = r#"{
            "code": 200,
            "message": "Data retrieved successfully",
            "data": {
                "nomor": 1,
                "nama": "الفاتحة",
                "namaLatin": "Al-Fatihah",
                "jumlahAyat": 7,
                "tempatTurun": "Mekah",
                "arti": "Pembukaan",
                "deskripsi": "Surat pembuka",
                "tafsir": [{ "ayat": 1, "teks": "Tafsir ayat pertama" }]
            }
        }"#;

        let parsed: ApiResponse<TafsirSurah> =
            serde_json::from_str(json).expect("tafsir response should parse");

        assert_eq!(parsed.data.nomor, 1);
        assert_eq!(parsed.data.tafsir[0].ayat, 1);
        assert_eq!(parsed.data.tafsir[0].teks, "Tafsir ayat pertama");
    }

    #[test]
    fn maps_quran_api_id_tafsir_response() {
        let json = r#"{
            "code": 200,
            "status": "OK.",
            "message": "Success fetching surah.",
            "data": {
                "number": 112,
                "numberOfVerses": 2,
                "name": {
                    "short": "الإخلاص",
                    "long": "سورة الإخلاص",
                    "transliteration": { "en": "Al-Ikhlaas", "id": "Al-Ikhlas" },
                    "translation": { "en": "Sincerity", "id": "Ikhlas" }
                },
                "revelation": { "arab": "مكة", "en": "Meccan", "id": "Makkiyyah" },
                "tafsir": { "id": "Deskripsi surah" },
                "verses": [
                    {
                        "number": { "inQuran": 6222, "inSurah": 1 },
                        "tafsir": { "id": { "short": "Tafsir pendek", "long": "Tafsir panjang" } }
                    },
                    {
                        "number": { "inQuran": 6223, "inSurah": 2 },
                        "tafsir": { "id": { "short": "Fallback pendek", "long": "" } }
                    }
                ]
            }
        }"#;

        let parsed: ApiResponse<QuranApiIdSurah> =
            serde_json::from_str(json).expect("quran-api-id response should parse");
        let mapped: TafsirSurah = parsed.data.into();

        assert_eq!(mapped.nomor, 112);
        assert_eq!(mapped.nama_latin, "Al-Ikhlas");
        assert_eq!(mapped.jumlah_ayat, 2);
        assert_eq!(mapped.tafsir[0].ayat, 1);
        assert_eq!(mapped.tafsir[0].teks, "Tafsir panjang");
        assert_eq!(mapped.tafsir[1].teks, "Fallback pendek");
    }
}
