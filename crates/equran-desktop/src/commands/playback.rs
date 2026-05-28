use std::collections::VecDeque;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use equran_core::{
    audio::{
        cache::AudioCache,
        player::play_audio_cancellable,
        tts::{NaturalIndonesianStatus, TtsEngine},
    },
    domain::{Lang, Qari},
    playback::{AyahSelection, PlaybackEngine, PlaybackEvent},
};
use serde::Serialize;
use tauri::{Emitter, State, Window};
use tokio::sync::Mutex;
use tokio::task::JoinHandle;
use tokio_util::sync::CancellationToken;

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
    cancel_token: Mutex<CancellationToken>,
    tts_enabled: Arc<AtomicBool>,
}

impl PlaybackState {
    pub fn new() -> Self {
        Self {
            cancel_token: Mutex::new(CancellationToken::new()),
            tts_enabled: Arc::new(AtomicBool::new(true)),
        }
    }
}

#[tauri::command]
pub async fn start_playback(
    window: Window,
    state: State<'_, AppState>,
    playback_state: State<'_, PlaybackState>,
    surah_number: u8,
    qari: String,
    lang: String,
    from_ayah: Option<u16>,
    to_ayah: Option<u16>,
    tts_enabled: Option<bool>,
) -> Result<(), String> {
    let qari: Qari = qari.parse().map_err(|e| e)?;
    let lang: Lang = lang.parse().map_err(|e| e)?;
    let tts_enabled = tts_enabled.unwrap_or(true);

    if tts_enabled && lang == Lang::Id {
        ensure_natural_indonesian_ready(&state)?;
    }

    let cancel_token = {
        let mut token = playback_state.cancel_token.lock().await;
        token.cancel();
        let new_token = CancellationToken::new();
        *token = new_token.clone();
        new_token
    };

    let api = state.api().clone();
    let surah = api
        .fetch_surah(surah_number)
        .await
        .map_err(|e| e.to_string())?;

    let english_surah = if lang == Lang::En {
        Some(
            api.fetch_surah_english(surah_number)
                .await
                .map_err(|e| e.to_string())?,
        )
    } else {
        None
    };

    let cache = AudioCache::new().map_err(|e| e.to_string())?;
    let cancel_for_check = cancel_token.clone();
    let tts_flag = playback_state.tts_enabled.clone();
    tts_flag.store(tts_enabled, Ordering::Relaxed);
    let engine = PlaybackEngine::new(api, cache, false, tts_enabled, false)
        .with_project_roots(state.tts_roots().to_vec())
        .with_tts_enabled_flag(tts_flag)
        .with_cancel_token(cancel_token);

    let selection = AyahSelection::new(None, from_ayah, to_ayah);

    tokio::spawn(async move {
        let window_clone = window.clone();
        let event_handler = move |event: PlaybackEvent| {
            let payload = PlaybackProgressPayload::from(event);
            let _ = window_clone.emit("playback-progress", &payload);
        };

        let result = engine
            .play_with_events(
                &surah,
                english_surah.as_ref(),
                selection,
                qari,
                lang,
                Some(&event_handler),
            )
            .await;

        if let Err(e) = &result {
            let _ = window.emit("playback-error", e.to_string());
        }
        if !cancel_for_check.is_cancelled() {
            let _ = window.emit("playback-finished", ());
        }
    });

    Ok(())
}

#[tauri::command]
pub async fn stop_playback(playback_state: State<'_, PlaybackState>) -> Result<(), String> {
    let token = playback_state.cancel_token.lock().await;
    token.cancel();
    Ok(())
}

#[tauri::command]
pub async fn get_cache_status(
    surah_number: u8,
    qari: String,
) -> Result<CacheStatusPayload, String> {
    let qari: Qari = qari.parse().map_err(|e| e)?;
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
        ensure_natural_indonesian_ready(&state)?;
    }

    playback_state.tts_enabled.store(enabled, Ordering::Relaxed);
    Ok(())
}

#[tauri::command]
pub async fn play_tafsir_voice(
    playback_state: State<'_, PlaybackState>,
    state: State<'_, AppState>,
    surah_number: u8,
    ayah_number: u16,
    text: String,
) -> Result<(), String> {
    let text = text.trim().to_owned();
    if text.is_empty() {
        return Err("Tafsir text is empty".to_owned());
    }

    let cancel_token = {
        let mut token = playback_state.cancel_token.lock().await;
        token.cancel();
        let new_token = CancellationToken::new();
        *token = new_token.clone();
        new_token
    };

    let cache = AudioCache::new().map_err(|e| e.to_string())?;
    let tts = TtsEngine::with_project_roots(cache.root().clone(), state.tts_roots().to_vec());
    let chunks = TtsEngine::split_tafsir_text(&text);
    if chunks.is_empty() {
        return Err("Tafsir text is empty".to_owned());
    }

    let current_audio = tts
        .synthesize_tafsir_chunk_cached(&chunks[0], surah_number, ayah_number, 1)
        .await
        .map_err(|e| e.to_string())?;

    const TAFSIR_PREFETCH_AHEAD: usize = 3;
    let mut next_chunk_index = 1;
    let mut queued_audio: VecDeque<JoinHandle<anyhow::Result<PathBuf>>> = VecDeque::new();

    while next_chunk_index < chunks.len() && queued_audio.len() < TAFSIR_PREFETCH_AHEAD {
        queued_audio.push_back(spawn_tafsir_chunk_synthesis(
            tts.clone(),
            chunks[next_chunk_index].clone(),
            surah_number,
            ayah_number,
            next_chunk_index + 1,
        ));
        next_chunk_index += 1;
    }

    let mut current_audio = Some(current_audio);

    while let Some(audio_path) = current_audio.take() {
        while next_chunk_index < chunks.len() && queued_audio.len() < TAFSIR_PREFETCH_AHEAD {
            queued_audio.push_back(spawn_tafsir_chunk_synthesis(
                tts.clone(),
                chunks[next_chunk_index].clone(),
                surah_number,
                ayah_number,
                next_chunk_index + 1,
            ));
            next_chunk_index += 1;
        }

        play_audio_cancellable(&audio_path, &cancel_token)
            .await
            .map_err(|e| e.to_string())?;

        if cancel_token.is_cancelled() {
            return Ok(());
        }

        if let Some(next_audio) = queued_audio.pop_front() {
            current_audio = Some(
                next_audio
                    .await
                    .map_err(|e| format!("failed to join tafsir TTS task: {e}"))?
                    .map_err(|e| e.to_string())?,
            );
        }
    }

    Ok(())
}

fn spawn_tafsir_chunk_synthesis(
    tts: TtsEngine,
    chunk: String,
    surah_number: u8,
    ayah_number: u16,
    chunk_number: usize,
) -> JoinHandle<anyhow::Result<PathBuf>> {
    tokio::spawn(async move {
        tts.synthesize_tafsir_chunk_cached(&chunk, surah_number, ayah_number, chunk_number)
            .await
    })
}

fn ensure_natural_indonesian_ready(state: &AppState) -> Result<(), String> {
    let cache = AudioCache::new().map_err(|e| e.to_string())?;
    let tts = TtsEngine::with_project_roots(cache.root().clone(), state.tts_roots().to_vec());

    if matches!(
        tts.natural_indonesian_status(),
        NaturalIndonesianStatus::Ready
    ) {
        return Ok(());
    }

    Err("Download Natural Indonesian Voice before enabling TTS Translation.".to_owned())
}
