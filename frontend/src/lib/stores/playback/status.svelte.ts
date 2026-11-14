import { invoke } from '@tauri-apps/api/core';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';

import { readableError } from './errors';
import { playbackState, type TtsVoiceStatus, type TtsVoiceStatusPayload } from './state.svelte';

/** Enables or disables the translation voice for the next session. */
export function setTtsEnabled(enabled: boolean) {
  if (enabled && playbackState.ttsVoiceStatus !== 'ready') {
    playbackState.ttsEnabled = false;
    playbackState.ttsVoiceMessage = 'Supertonic 3 voice is not ready yet.';
    return;
  }

  playbackState.ttsEnabled = enabled;
  void invoke('set_tts_enabled', { enabled }).catch((error: unknown) => {
    playbackState.ttsEnabled = false;
    playbackState.ttsVoiceMessage = readableError(error, 'Unable to enable TTS translation.');
  });
}

export async function refreshTtsVoiceStatus() {
  playbackState.ttsVoiceStatus = 'checking';
  playbackState.ttsVoiceMessage = 'Checking Supertonic 3 voice status.';

  try {
    const result = await invoke<TtsVoiceStatusPayload>('get_tts_voice_status');
    applyTtsVoiceStatus(result);
  } catch (error) {
    playbackState.ttsVoiceStatus = 'failed';
    playbackState.ttsVoiceMessage = readableError(
      error,
      'Unable to check Supertonic 3 voice status.'
    );
    playbackState.ttsEnabled = false;
  }
}

let setupProgressUnlisten: UnlistenFn | null = null;

/**
 * Runs the bundled setup script and streams its output into the panel.
 *
 * The backend writes the venv under the app's data directory, so this needs no
 * password. On success the returned status is applied directly; on failure the
 * reader keeps the log so the Python or network error is visible.
 */
export async function setupTtsRuntime(): Promise<string> {
  if (playbackState.ttsSettingUp) {
    return '';
  }

  playbackState.ttsSettingUp = true;
  playbackState.ttsSetupLog = [];
  let error = '';

  try {
    if (!setupProgressUnlisten) {
      setupProgressUnlisten = await listen<{ line: string }>('tts-setup-progress', (event) => {
        // Keep the tail: pip is chatty and the panel shows recent activity.
        const next = [...playbackState.ttsSetupLog, event.payload.line];
        playbackState.ttsSetupLog = next.slice(-40);
      });
    }

    const result = await invoke<TtsVoiceStatusPayload>('setup_tts_runtime');
    applyTtsVoiceStatus(result);
    if (result.ready) {
      // A freshly built runtime should read translations by default. The
      // earlier "not ready" status turned the toggle off, so restore it now
      // instead of leaving the reader to enable it by hand after setup.
      playbackState.ttsEnabled = true;
    } else {
      error = result.detail || 'Setup finished but the voice is still not ready.';
    }
  } catch (invokeError) {
    error = readableError(invokeError, 'Unable to set up the speech runtime.');
  } finally {
    playbackState.ttsSettingUp = false;
  }

  return error;
}

/**
 * Reads how much qari audio for `surahNumber` is already on disk.
 *
 * `totalAyat` comes from the selected surah, not from the backend: the command
 * counts files, and only the surah knows how many ayahs there should be. Without
 * a denominator the panel could never show progress, which is why it used to
 * report "nothing stored" even when the cache was full.
 */
export async function refreshCacheStatus(
  surahNumber: number,
  totalAyat: number,
  qari = playbackState.qari
) {
  try {
    const result = await invoke<{ cachedCount: number }>('get_cache_status', {
      surahNumber,
      qari
    });
    playbackState.cacheReady = Math.min(result.cachedCount, totalAyat);
    playbackState.cacheTotal = totalAyat;
  } catch {
    playbackState.cacheReady = 0;
    playbackState.cacheTotal = totalAyat;
  }
}

type CacheUsagePayload = {
  totalBytes: number;
  fileCount: number;
  qariBytes: number;
  translationBytes: number;
  tafsirBytes: number;
  limitBytes: number | null;
};

/**
 * Reads what the media cache holds on disk and against what cap.
 *
 * The qari progress bar answers "is this surah ready?"; this answers "how much
 * disk is the narration using?", which is what the storage control needs.
 */
export async function refreshCacheUsage() {
  try {
    const result = await invoke<CacheUsagePayload>('get_cache_usage');
    applyCacheUsage(result);
  } catch {
    playbackState.cacheUsageBytes = 0;
    playbackState.cacheLimitBytes = null;
    playbackState.cacheFileCount = 0;
  }
}

/**
 * Empties the media cache. Returns an error message when playback is running,
 * so the caller can show why it did nothing.
 */
export async function clearCache(): Promise<string> {
  try {
    const result = await invoke<CacheUsagePayload>('clear_cache');
    applyCacheUsage(result);
    return '';
  } catch (error) {
    return readableError(error, 'Unable to clear the audio cache.');
  }
}

function applyCacheUsage(result: CacheUsagePayload) {
  playbackState.cacheUsageBytes = result.totalBytes;
  playbackState.cacheLimitBytes = result.limitBytes;
  playbackState.cacheFileCount = result.fileCount;
  playbackState.cacheQariBytes = result.qariBytes;
  playbackState.cacheTranslationBytes = result.translationBytes;
  playbackState.cacheTafsirBytes = result.tafsirBytes;
}

function applyTtsVoiceStatus(result: TtsVoiceStatusPayload) {
  playbackState.ttsVoiceStatus = result.ready ? 'ready' : (result.status as TtsVoiceStatus);
  playbackState.ttsVoiceMessage = result.detail;
  playbackState.ttsCanSetup = result.canSetup;
  playbackState.ttsSetupCommand = result.setupCommand;

  if (!result.ready) {
    playbackState.ttsEnabled = false;
  }
}
