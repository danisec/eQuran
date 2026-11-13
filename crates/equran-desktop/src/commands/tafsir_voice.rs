//! Tafsir narration commands.
//!
//! The tafsir voice is a separate, cancellable stream from the qari session: it
//! shares the playback state only so stop, pause and resume can release it.

use std::collections::VecDeque;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use equran_core::audio::{
    cache::AudioCache,
    player::{PlaybackControl, play_audio_controlled},
    tts::TtsEngine,
};
use tauri::State;
use tokio::task::JoinHandle;
use tokio_util::sync::CancellationToken;

use super::playback::PlaybackState;
use super::tts::ensure_tts_ready;
use crate::AppState;

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
