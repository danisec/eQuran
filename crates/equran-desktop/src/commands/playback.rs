use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use equran_core::{
    api::models::{EnglishSurah, Surah},
    audio::{cache::AudioCache, player::PlaybackControl},
    domain::{Lang, Qari},
    playback::{AyahSelection, PlaybackEngine, PlaybackEvent},
};
use serde::{Deserialize, Serialize};
use tauri::{Emitter, State, Window};
use tokio::sync::Mutex;
use tokio_util::sync::CancellationToken;

use super::tts::ensure_tts_ready;
use crate::AppState;

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlaybackProgressPayload {
    pub surah: u8,
    pub ayah: u16,
    pub phase: String,
    pub current: usize,
    pub total: usize,
}

impl From<PlaybackEvent> for PlaybackProgressPayload {
    fn from(event: PlaybackEvent) -> Self {
        Self {
            surah: event.surah,
            ayah: event.ayah,
            phase: event.phase.as_str().to_owned(),
            current: event.current,
            total: event.total,
        }
    }
}

pub struct PlaybackState {
    pub(crate) cancel_token: Mutex<CancellationToken>,
    pub(crate) tts_enabled: Arc<AtomicBool>,
    pub(crate) tafsir_voice_active: Arc<AtomicBool>,
    pub(crate) tafsir_voice_paused: Arc<AtomicBool>,
    pub(crate) tafsir_control: Arc<PlaybackControl>,
    /// The recitation's own pause/resume flag, shared with the engine.
    pub(crate) playback_control: Arc<PlaybackControl>,
}

impl Default for PlaybackState {
    fn default() -> Self {
        Self::new()
    }
}

impl PlaybackState {
    pub fn new() -> Self {
        Self {
            cancel_token: Mutex::new(CancellationToken::new()),
            tts_enabled: Arc::new(AtomicBool::new(true)),
            tafsir_voice_active: Arc::new(AtomicBool::new(false)),
            tafsir_voice_paused: Arc::new(AtomicBool::new(false)),
            tafsir_control: Arc::new(PlaybackControl::new()),
            playback_control: Arc::new(PlaybackControl::new()),
        }
    }
}

/// Arguments for [`start_playback`], grouped so the command takes one request
/// value instead of a flat list of optional fields.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StartPlaybackRequest {
    pub surah_number: u8,
    pub qari: String,
    pub lang: String,
    pub from_ayah: Option<u16>,
    pub to_ayah: Option<u16>,
    pub tts_enabled: Option<bool>,
    pub read_tafsir: Option<bool>,
}

/// The session resolved before the playback task starts.
struct PreparedSession {
    surah: Surah,
    english_surah: Option<EnglishSurah>,
    qari: Qari,
    lang: Lang,
    selection: AyahSelection,
    tts_enabled: bool,
    read_tafsir: bool,
}

#[tauri::command]
pub async fn start_playback(
    window: Window,
    state: State<'_, AppState>,
    playback_state: State<'_, PlaybackState>,
    request: StartPlaybackRequest,
) -> Result<(), String> {
    let session = prepare_session(&window, state.inner(), request).await?;
    let cancel_token = reset_cancel_token(playback_state.inner()).await;

    let cache = AudioCache::new().map_err(|e| e.to_string())?;
    let tts_flag = playback_state.tts_enabled.clone();
    tts_flag.store(session.tts_enabled, Ordering::Relaxed);

    // A previous session may have ended while paused; a new one always starts
    // sounding.
    playback_state.playback_control.resume();

    let engine = PlaybackEngine::new(
        state.api().clone(),
        cache,
        false,
        session.tts_enabled,
        false,
    )
    .with_project_roots(state.tts_roots().to_vec())
    .with_tts_enabled_flag(tts_flag)
    .with_read_tafsir(session.read_tafsir)
    .with_cancel_token(cancel_token.clone())
    .with_playback_control(playback_state.playback_control.clone());

    spawn_playback(window, engine, session, cancel_token);
    Ok(())
}

/// Resolves the request into a playable session.
///
/// Qari recitation is the core of the app; the translation voice is additive.
/// A missing TTS runtime must never stop the recitation, so it only downgrades
/// the session to qari-only and says why.
async fn prepare_session(
    window: &Window,
    state: &AppState,
    request: StartPlaybackRequest,
) -> Result<PreparedSession, String> {
    let qari: Qari = request.qari.parse()?;
    let lang: Lang = request.lang.parse()?;
    let mut tts_enabled = request.tts_enabled.unwrap_or(true);
    let mut read_tafsir = request.read_tafsir.unwrap_or(false);

    if tts_enabled || read_tafsir {
        match ensure_tts_ready(state) {
            Ok(()) => {}
            Err(reason) => {
                tts_enabled = false;
                read_tafsir = false;
                let _ = window.emit(
                    "playback-notice",
                    format!("{reason} Recitation continues without translation."),
                );
            }
        }
    }

    let api = state.api();
    let mut surah = api
        .fetch_surah(request.surah_number)
        .await
        .map_err(|e| e.to_string())?;

    let english_surah = if lang == Lang::En {
        Some(
            api.fetch_surah_english(request.surah_number)
                .await
                .map_err(|e| e.to_string())?,
        )
    } else {
        None
    };

    // Tafsir is needed for English playback (which reads it as the deeper
    // translation) and for the optional "read tafsir" mode. English uses
    // UmmahAPI commentary, Indonesian keeps quran-api-id. A failure here is not
    // fatal: English playback falls back to the verse translation and tafsir
    // narration is skipped.
    if lang == Lang::En || read_tafsir {
        match api.fetch_tafsir(request.surah_number, lang).await {
            Ok(tafsir) => {
                for ayah in &mut surah.ayat {
                    ayah.tafsir = tafsir
                        .tafsir
                        .iter()
                        .find(|entry| entry.ayat == ayah.nomor_ayat)
                        .map(|entry| entry.teks.clone());
                }
            }
            Err(error) => {
                eprintln!(
                    "Tafsir unavailable for surah {}: {error}",
                    request.surah_number
                );
            }
        }
    }

    Ok(PreparedSession {
        surah,
        english_surah,
        qari,
        lang,
        selection: AyahSelection::new(None, request.from_ayah, request.to_ayah),
        tts_enabled,
        read_tafsir,
    })
}

/// Cancels the previous session and installs a fresh token for this one.
async fn reset_cancel_token(playback_state: &PlaybackState) -> CancellationToken {
    let mut token = playback_state.cancel_token.lock().await;
    token.cancel();
    let new_token = CancellationToken::new();
    *token = new_token.clone();
    new_token
}

/// Runs the engine on its own task so the command returns as soon as playback
/// is scheduled, reporting progress and failures as window events.
fn spawn_playback(
    window: Window,
    engine: PlaybackEngine,
    session: PreparedSession,
    cancel_token: CancellationToken,
) {
    tokio::spawn(async move {
        let window_for_events = window.clone();
        let event_handler = move |event: PlaybackEvent| {
            let payload = PlaybackProgressPayload::from(event);
            let _ = window_for_events.emit("playback-progress", &payload);
        };

        let result = engine
            .play_with_events(
                &session.surah,
                session.english_surah.as_ref(),
                session.selection,
                session.qari,
                session.lang,
                Some(&event_handler),
            )
            .await;

        if let Err(error) = &result {
            let _ = window.emit("playback-error", error.to_string());
        }
        if !cancel_token.is_cancelled() {
            let _ = window.emit("playback-finished", ());
        }
    });
}

#[tauri::command]
pub async fn stop_playback(playback_state: State<'_, PlaybackState>) -> Result<(), String> {
    let token = playback_state.cancel_token.lock().await;
    // Release a paused sink first: the audio loop checks the cancel flag while
    // it waits, and the next session must not inherit the paused state.
    playback_state.playback_control.resume();
    token.cancel();
    playback_state
        .tafsir_voice_active
        .store(false, Ordering::Relaxed);
    playback_state
        .tafsir_voice_paused
        .store(false, Ordering::Relaxed);
    Ok(())
}

/// Pauses the current clip without discarding its position.
///
/// The audio thread keeps the decoded sink alive, so resuming continues from the
/// same sample instead of restarting the ayah — which matters most on the long
/// ayat (2:282 runs for minutes) and when the reader pauses mid-translation.
#[tauri::command]
pub async fn pause_playback(playback_state: State<'_, PlaybackState>) -> Result<(), String> {
    playback_state.playback_control.pause();
    Ok(())
}

#[tauri::command]
pub async fn resume_playback(playback_state: State<'_, PlaybackState>) -> Result<(), String> {
    playback_state.playback_control.resume();
    Ok(())
}

#[tauri::command]
pub async fn get_cache_status(
    surah_number: u8,
    qari: String,
) -> Result<CacheStatusPayload, String> {
    let qari: Qari = qari.parse()?;
    let cache = AudioCache::new().map_err(|e| e.to_string())?;
    let audio_dir = cache.root().join("audio");

    let mut cached_count: usize = 0;
    if audio_dir.exists() {
        let prefix = format!("{}_{:03}_", qari.key(), surah_number);
        if let Ok(entries) = std::fs::read_dir(&audio_dir) {
            for entry in entries.flatten() {
                if entry.file_name().to_string_lossy().starts_with(&prefix) {
                    cached_count += 1;
                }
            }
        }
    }

    Ok(CacheStatusPayload { cached_count })
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CacheStatusPayload {
    pub cached_count: usize,
}

#[tauri::command]
pub async fn set_tts_enabled(
    state: State<'_, AppState>,
    playback_state: State<'_, PlaybackState>,
    enabled: bool,
) -> Result<(), String> {
    if enabled {
        ensure_tts_ready(&state)?;
    }

    playback_state.tts_enabled.store(enabled, Ordering::Relaxed);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn start_request_deserializes_the_frontend_payload() {
        let request: StartPlaybackRequest = serde_json::from_str(
            r#"{
                "surahNumber": 55,
                "qari": "misyari",
                "lang": "id",
                "fromAyah": 1,
                "toAyah": 3,
                "ttsEnabled": false,
                "readTafsir": true
            }"#,
        )
        .expect("frontend payload should deserialize");

        assert_eq!(request.surah_number, 55);
        assert_eq!(request.qari, "misyari");
        assert_eq!(request.lang, "id");
        assert_eq!(request.from_ayah, Some(1));
        assert_eq!(request.to_ayah, Some(3));
        assert_eq!(request.tts_enabled, Some(false));
        assert_eq!(request.read_tafsir, Some(true));
    }

    #[test]
    fn start_request_keeps_omitted_options_empty() {
        let request: StartPlaybackRequest =
            serde_json::from_str(r#"{"surahNumber":1,"qari":"sudais","lang":"en"}"#)
                .expect("minimal payload should deserialize");

        assert_eq!(request.from_ayah, None);
        assert_eq!(request.to_ayah, None);
        assert_eq!(request.tts_enabled, None);
        assert_eq!(request.read_tafsir, None);
    }
}
