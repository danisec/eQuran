use super::normalize::*;
use super::persist::{load_from, save_to};
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

#[test]
fn english_search_ignores_indonesian_fields() {
    let index = index_from(vec![entry(
        1,
        1,
        "Dengan nama Allah",
        "بسم الله",
        "Bismillah",
        Some("In the name of Allah"),
        Some("Tafsir menyebut sabar dan syukur"),
    )]);

    // "sabar" only exists in the Indonesian tafsir: invisible in English mode.
    assert!(
        index
            .search_for("sabar", 10, crate::domain::Lang::En)
            .is_empty()
    );
    assert_eq!(
        index.search_for("sabar", 10, crate::domain::Lang::Id).len(),
        1
    );

    // "dengan" only exists in the Indonesian translation.
    assert!(
        index
            .search_for("dengan", 10, crate::domain::Lang::En)
            .is_empty()
    );
    assert_eq!(
        index
            .search_for("dengan", 10, crate::domain::Lang::Id)
            .len(),
        1
    );

    // An English query quotes the English translation, never the tafsir.
    let hits = index.search_for("name", 10, crate::domain::Lang::En);
    assert_eq!(hits.len(), 1);
    assert_eq!(hits[0].matched_field, "english");
    assert!(hits[0].snippet.to_lowercase().contains("name"));
}
