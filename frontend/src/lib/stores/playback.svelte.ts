//! Playback state, transport, settings and voice status.
//!
//! The store is split by concern so a change to one does not touch the others:
//!
//! - `state`   — the shared `playbackState` and the id-to-label maps
//! - `session` — listeners plus play/pause/resume/end/skip
//! - `settings`— persisted qari, language, prefetch, Read Tafsir, last surah
//! - `status`  — TTS voice readiness and the cache progress panel
//! - `errors`  — rejection-to-message helper
//!
//! Consumers keep importing from this file; the barrel below preserves the
//! original public names.

import { startTafsirVoicePolling, stopTafsirVoicePolling } from './tafsir.svelte';
import {
  destroyPlaybackSessionListeners,
  initPlaybackSessionListeners
} from './playback/session.svelte';

export { playbackState } from './playback/state.svelte';
export type { PlaybackProgress, TtsVoiceStatus, TtsVoiceStatusPayload } from './playback/state.svelte';
export {
  endPlayback,
  nextAyah,
  pausePlayback,
  playFromAyah,
  playSurah,
  prevAyah,
  resumePlayback,
  retryPlayback,
  startPlayback,
  togglePlayback,
  toggleRepeat,
  toggleShuffle
} from './playback/session.svelte';
export {
  loadSettings,
  rememberLastSurah,
  setLang,
  setPrefetch,
  setQari,
  setReadTafsir
} from './playback/settings.svelte';
export {
  refreshCacheStatus,
  refreshTtsVoiceStatus,
  setTtsEnabled
} from './playback/status.svelte';

/** Wires up the session listeners and the tafsir voice poll. */
export async function initPlaybackListeners() {
  await initPlaybackSessionListeners();
  startTafsirVoicePolling();
}

export function destroyPlaybackListeners() {
  destroyPlaybackSessionListeners();
  stopTafsirVoicePolling();
}
