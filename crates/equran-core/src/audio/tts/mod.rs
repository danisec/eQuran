//! On-device Supertonic 3 runtime used for translation and tafsir narration.
//!
//! The engine stays cache-first: a request renders into a temporary file and
//! only publishes it once the WAV is valid, so an interrupted synthesis never
//! becomes a false cache hit. When the runtime is missing, callers get an
//! actionable diagnosis instead of a generic failure.

mod config;
mod engine;
mod runtime;
mod service;
mod text;

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

pub use config::forget_supertonic_probes;
pub use engine::TtsEngine;
pub use runtime::RuntimeSetup;

const DEFAULT_VOICE: &str = "M2";
const CACHE_SUBDIR: &str = "tts";
const SERVICE_REQUEST_TIMEOUT: Duration = Duration::from_secs(300);
/// Characters per synthesis request.
///
/// Supertonic's diffusion loop costs roughly 0.12 s per character per step on
/// CPU, so a long request blocks the single service process for minutes. Short
/// requests keep the first chunk audible quickly and let the prefetch queue
/// overlap synthesis with playback.
const TAFSIR_CHUNK_CHARS: usize = 320;
/// Diffusion steps handed to Supertonic.
///
/// The library default is 8. Measured on a Ryzen 7 5700U, 5 steps cut synthesis
/// time by ~43% with no audible quality loss for narration, and 4 steps only
/// saves another ~25% while starting to sound flat. Override with
/// `EQURAN_TTS_SUPERTONIC_STEPS`.
const DEFAULT_TOTAL_STEPS: u32 = 5;
/// Device preference handed to the Python runtime.
///
/// `auto` uses a GPU execution provider when one is installed and otherwise
/// falls back to CPU; `cpu` keeps the GPU free for games; `gpu` is the same as
/// `auto` but makes a missing GPU explicit in the logs. Override with
/// `EQURAN_TTS_DEVICE`.
const DEFAULT_DEVICE: &str = "auto";
/// Seconds the Python service may sit idle before it exits.
///
/// The loaded model holds ~1 GB of RSS and a thread pool; releasing it while
/// nobody is listening keeps the desktop responsive. The next request respawns
/// the service and pays the model load again. Override with
/// `EQURAN_TTS_IDLE_TIMEOUT` (0 keeps it alive forever).
const DEFAULT_IDLE_TIMEOUT_SECS: u32 = 600;

/// Readiness of the local Supertonic runtime, surfaced to the UI so the
/// translation toggle can be disabled with an actionable message.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TtsVoiceStatus {
    Ready,
    MissingRuntime,
    MissingScript,
}

impl TtsVoiceStatus {
    pub fn is_ready(self) -> bool {
        matches!(self, Self::Ready)
    }
}

/// Builds a unique sibling path so concurrent synthesis never shares a temp file.
fn temporary_path(target: &Path) -> PathBuf {
    static COUNTER: AtomicU64 = AtomicU64::new(0);

    let sequence = COUNTER.fetch_add(1, Ordering::Relaxed);
    let stem = target
        .file_stem()
        .and_then(|stem| stem.to_str())
        .unwrap_or("tts");
    target.with_file_name(format!("{stem}.{}.{sequence}.part", std::process::id()))
}

#[cfg(test)]
mod tests;
