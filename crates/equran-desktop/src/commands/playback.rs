use std::collections::VecDeque;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use equran_core::{
    audio::{
        cache::AudioCache,
        player::{PlaybackControl, play_audio_controlled},
        tts::{TtsEngine, TtsVoiceStatus},
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
    tafsir_voice_active: Arc<AtomicBool>,
    tafsir_voice_paused: Arc<AtomicBool>,
    tafsir_control: Arc<PlaybackControl>,
    /// The recitation's own pause/resume flag, shared with the engine.
    playback_control: Arc<PlaybackControl>,
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
    read_tafsir: Option<bool>,
) -> Result<(), String> {
    let qari: Qari = qari.parse()?;
    let lang: Lang = lang.parse()?;
    let mut tts_enabled = tts_enabled.unwrap_or(true);
    let mut read_tafsir = read_tafsir.unwrap_or(false);

    // Qari recitation is the core of the app; the translation voice is additive.
    // A missing TTS runtime must never stop the recitation, so it only downgrades
    // the session to qari-only and says why.
    if tts_enabled || read_tafsir {
        match ensure_tts_ready(&state) {
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

    // The tafsir is served by the Indonesian API. It is needed for English
    // playback (which reads it as the translation) and for the optional
    // "read tafsir" mode. A failure here is not fatal: English playback falls
    // back to the verse translation and tafsir narration is skipped.
    let mut surah = surah;
    if lang == Lang::En || read_tafsir {
        match api.fetch_tafsir(surah_number).await {
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
                eprintln!("Tafsir unavailable for surah {surah_number}: {error}");
            }
        }
    }

    let cache = AudioCache::new().map_err(|e| e.to_string())?;
    let cancel_for_check = cancel_token.clone();
    let tts_flag = playback_state.tts_enabled.clone();
    tts_flag.store(tts_enabled, Ordering::Relaxed);

    // A previous session may have ended while paused; a new one always starts
    // sounding.
    playback_state.playback_control.resume();

    let engine = PlaybackEngine::new(api, cache, false, tts_enabled, false)
        .with_project_roots(state.tts_roots().to_vec())
        .with_tts_enabled_flag(tts_flag)
        .with_read_tafsir(read_tafsir)
        .with_cancel_token(cancel_token)
        .with_playback_control(playback_state.playback_control.clone());

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

#[tauri::command]
pub async fn is_tafsir_voice_active(
    playback_state: State<'_, PlaybackState>,
) -> Result<bool, String> {
    Ok(playback_state.tafsir_voice_active.load(Ordering::Relaxed))
}

#[tauri::command]
pub async fn is_tafsir_voice_paused(
    playback_state: State<'_, PlaybackState>,
) -> Result<bool, String> {
    Ok(playback_state.tafsir_voice_paused.load(Ordering::Relaxed))
}

/// Pauses the tafsir narration without discarding its position.
///
/// The audio thread keeps the decoded sink alive, so resuming continues from
/// the same sample instead of restarting the chunk.
#[tauri::command]
pub async fn pause_tafsir_voice(playback_state: State<'_, PlaybackState>) -> Result<(), String> {
    if !playback_state.tafsir_voice_active.load(Ordering::Relaxed) {
        return Ok(());
    }

    playback_state.tafsir_control.pause();
    playback_state
        .tafsir_voice_paused
        .store(true, Ordering::Relaxed);
    Ok(())
}

#[tauri::command]
pub async fn resume_tafsir_voice(playback_state: State<'_, PlaybackState>) -> Result<(), String> {
    if !playback_state.tafsir_voice_active.load(Ordering::Relaxed) {
        return Ok(());
    }

    playback_state.tafsir_control.resume();
    playback_state
        .tafsir_voice_paused
        .store(false, Ordering::Relaxed);
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

    ensure_tts_ready(&state)?;

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

    playback_state
        .tafsir_voice_active
        .store(true, Ordering::Relaxed);
    playback_state
        .tafsir_voice_paused
        .store(false, Ordering::Relaxed);
    playback_state.tafsir_control.resume();
    let active_flag = playback_state.tafsir_voice_active.clone();
    let paused_flag = playback_state.tafsir_voice_paused.clone();
    let control = playback_state.tafsir_control.clone();
    let result = play_tafsir_chunks(
        tts,
        chunks,
        surah_number,
        ayah_number,
        cancel_token,
        active_flag,
        paused_flag,
        control,
    )
    .await;
    playback_state
        .tafsir_voice_active
        .store(false, Ordering::Relaxed);
    playback_state
        .tafsir_voice_paused
        .store(false, Ordering::Relaxed);

    result
}

#[allow(clippy::too_many_arguments)]
async fn play_tafsir_chunks(
    tts: TtsEngine,
    chunks: Vec<String>,
    surah_number: u8,
    ayah_number: u16,
    cancel_token: CancellationToken,
    active_flag: Arc<AtomicBool>,
    paused_flag: Arc<AtomicBool>,
    control: Arc<PlaybackControl>,
) -> Result<(), String> {
    let current_audio = tts
        .synthesize_tafsir_chunk_cached(&chunks[0], surah_number, ayah_number, 1)
        .await
        .map_err(|e| format!("Failed to generate tafsir voice: {e}"))?;

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

        play_audio_controlled(&audio_path, &cancel_token, &control)
            .await
            .map_err(|e| format!("Failed to play tafsir audio: {e}"))?;

        if cancel_token.is_cancelled() {
            break;
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

    active_flag.store(false, Ordering::Relaxed);
    paused_flag.store(false, Ordering::Relaxed);
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

fn ensure_tts_ready(state: &AppState) -> Result<(), String> {
    let cache = AudioCache::new().map_err(|e| e.to_string())?;
    let tts = TtsEngine::with_project_roots(cache.root().clone(), state.tts_roots().to_vec());

    match tts.voice_status() {
        TtsVoiceStatus::Ready => Ok(()),
        TtsVoiceStatus::MissingScript => Err(
            "TTS engine files are missing. Reinstall eQuran before enabling TTS Translation."
                .to_owned(),
        ),
        TtsVoiceStatus::MissingRuntime => Err(
            "TTS runtime is not ready. Install the supertonic Python package, then restart eQuran."
                .to_owned(),
        ),
    }
}
