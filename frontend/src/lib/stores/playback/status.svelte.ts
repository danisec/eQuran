import { invoke } from '@tauri-apps/api/core';

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
  playbackState.ttsVoiceMessage = result.message;

  if (!result.ready) {
    playbackState.ttsEnabled = false;
  }
}
