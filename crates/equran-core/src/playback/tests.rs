//! Selection and event tests for the playback engine.

use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use super::*;
use crate::api::models::EnglishAyah;

fn ayah(number: u16) -> Ayat {
    Ayat {
        nomor_ayat: number,
        teks_arab: format!("arab {number}"),
        teks_latin: format!("latin {number}"),
        teks_indonesia: format!("id {number}"),
        tafsir: None,
        audio: HashMap::from([(
            "05".to_owned(),
            format!("https://example.test/{number}.mp3"),
        )]),
    }
}

fn surah() -> Surah {
    Surah {
        nomor: 2,
        nama: "البقرة".into(),
        nama_latin: "Al-Baqarah".into(),
        jumlah_ayat: 3,
        tempat_turun: "Madinah".into(),
        arti: "Sapi".into(),
        deskripsi: "Test surah".into(),
        audio_full: HashMap::new(),
        ayat: vec![ayah(1), ayah(2), ayah(3)],
    }
}

#[tokio::test]
async fn emits_recitation_events_for_selected_dry_run_ayat() {
    let cache = AudioCache::new().unwrap();
    let engine = PlaybackEngine::new(ApiClient::new().unwrap(), cache, true, false, false);
    let events = Arc::new(Mutex::new(Vec::new()));
    let captured_events = Arc::clone(&events);
    let handler = move |event: PlaybackEvent| {
        captured_events
            .lock()
            .unwrap()
            .push((event.ayah, event.phase, event.current, event.total));
    };

    engine
        .play_with_events(
            &surah(),
            None,
            AyahSelection::new(None, Some(2), Some(3)),
            Qari::Misyari,
            Lang::Id,
            Some(&handler),
        )
        .await
        .unwrap();

    assert_eq!(
        events.lock().unwrap().clone(),
        vec![
            (2, PlaybackPhase::Recitation, 2, 3),
            (3, PlaybackPhase::Recitation, 3, 3),
        ]
    );
}

#[test]
fn tafsir_phase_has_a_stable_wire_name() {
    assert_eq!(PlaybackPhase::Tafsir.as_str(), "tafsir");
}

#[test]
fn read_tafsir_is_opt_in() {
    let engine = PlaybackEngine::new(
        ApiClient::new().unwrap(),
        AudioCache::new().unwrap(),
        true,
        false,
        false,
    );

    assert!(!engine.read_tafsir);
    assert!(engine.with_read_tafsir(true).read_tafsir);
}

#[test]
fn plans_current_tts_generation_during_recitation_when_translation_enabled() {
    let plan = TtsGenerationPlan::new(true, true, true);

    assert!(plan.current_ayah);
    assert!(plan.next_ayah);
}

#[test]
fn english_translation_prefers_tafsir_over_verse_translation() {
    let engine = PlaybackEngine::new(
        ApiClient::new().unwrap(),
        AudioCache::new().unwrap(),
        true,
        false,
        false,
    );
    let mut ayah = ayah(1);
    ayah.tafsir = Some("Tafsir panjang untuk ayat satu".to_owned());
    let english = EnglishSurah {
        ayahs: vec![EnglishAyah {
            number_in_surah: 1,
            text_english: "Short verse translation".to_owned(),
        }],
    };

    let text = engine
        .translation_for(&ayah, Some(&english), Lang::En)
        .expect("English text should resolve");

    assert_eq!(text, "Tafsir panjang untuk ayat satu");
}

#[test]
fn english_translation_falls_back_to_verse_translation_without_tafsir() {
    let engine = PlaybackEngine::new(
        ApiClient::new().unwrap(),
        AudioCache::new().unwrap(),
        true,
        false,
        false,
    );
    let ayah = ayah(1);
    let english = EnglishSurah {
        ayahs: vec![EnglishAyah {
            number_in_surah: 1,
            text_english: "Short verse translation".to_owned(),
        }],
    };

    let text = engine
        .translation_for(&ayah, Some(&english), Lang::En)
        .expect("English text should resolve");

    assert_eq!(text, "Short verse translation");
}
