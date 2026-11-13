import { invoke } from '@tauri-apps/api/core';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import { tafsirState, tafsirVoiceState, tafsirTextForAyah } from './tafsir.svelte';

type PlaybackProgress = {
  surah: number;
  ayah: number;
  phase: string;
  current: number;
  total: number;
};

type TtsVoiceStatusPayload = {
  status: string;
  ready: boolean;
  message: string;
};

export type TtsVoiceStatus = 'checking' | 'ready' | 'missingRuntime' | 'missingScript' | 'failed';

const QARI_MAP: Record<string, string> = {
  misyari: 'Mishary Rashid Alafasy',
  sudais: 'Abdurrahman As-Sudais',
  dossari: 'Ibrahim Al-Dossari',
  juhany: 'Abdullah Al-Juhany',
  qasim: 'Abdul Muhsin Al-Qasim',
  yasser: 'Yasser Al-Dosari'
};

const LANG_MAP: Record<string, string> = {
  id: 'Indonesian',
  en: 'English'
};

const DEFAULT_VOLUME = 70;

export const playbackState = $state({
  currentAyah: 1,
  playing: false,
  phase: 'idle',
  /**
   * The surah the *session* belongs to, which is not necessarily the surah the
   * reader is browsing. The bar, the active verse and the skip controls all
   * read this, so navigating the mushaf never changes what is playing.
   */
  sessionSurah: 0,
  errorMessage: '',
  notice: '',
  qari: 'misyari',
  qariLabel: 'Mishary Rashid Alafasy',
  lang: 'id',
  langLabel: 'Indonesian',
  prefetch: true,
  readTafsir: false,
  lastSurah: 0,
  ttsEnabled: true,
  repeat: false,
  shuffle: false,
  cacheReady: 0,
  cacheTotal: 0,
  current: 0,
  total: 0,
  ttsVoiceStatus: 'checking' as TtsVoiceStatus,
  ttsVoiceMessage: 'Checking Supertonic 3 voice status.'
});

type Session = {
  surah: number;
  /** Where the reader started; replay and repeat return here. */
  from: number;
  to?: number;
  total: number;
};

let session: Session | null = null;
/** True while the backend still owns a running (possibly paused) engine. */
let engineAlive = false;
let phaseBeforePause = 'recitation';

/**
 * Writes the whole settings object, with optional overrides.
 *
 * `save_settings` replaces the payload outright, so every caller must send a
 * complete object. Building it in one place stops partial callers (such as
 * "remember the last surah" during navigation) from silently resetting the
 * reader's qari, language, or Read Tafsir preference.
 */
function persistSettings(overrides: Partial<PlaybackSettingsPayload> = {}) {
  const settings: PlaybackSettingsPayload = {
    qari: playbackState.qari,
    lang: playbackState.lang,
    prefetchTts: playbackState.prefetch,
    readTafsir: playbackState.readTafsir,
    volume: DEFAULT_VOLUME,
    lastSurah: playbackState.lastSurah,
    ...overrides
  };
  void invoke('save_settings', { settings });
}

type PlaybackSettingsPayload = {
  qari: string;
  lang: string;
  prefetchTts: boolean;
  readTafsir: boolean;
  volume: number;
  lastSurah: number;
};

let progressUnlisten: UnlistenFn | null = null;
let finishedUnlisten: UnlistenFn | null = null;
let errorUnlisten: UnlistenFn | null = null;
let noticeUnlisten: UnlistenFn | null = null;
let tafsirPollTimer: ReturnType<typeof setInterval> | null = null;

export async function initPlaybackListeners() {
  if (progressUnlisten || finishedUnlisten) {
    return;
  }

  progressUnlisten = await listen<PlaybackProgress>('playback-progress', (event) => {
    playbackState.sessionSurah = event.payload.surah;
    playbackState.currentAyah = event.payload.ayah;
    playbackState.phase = event.payload.phase;
    playbackState.current = event.payload.current;
    playbackState.total = event.payload.total;
  });

  finishedUnlisten = await listen('playback-finished', () => {
    engineAlive = false;
    if (playbackState.repeat && session) {
      void startPlayback(session.surah, session.from, session.to, session.total);
      return;
    }
    // The bar stays for a completed session so the reader gets a closing
    // action (replay) instead of the player simply vanishing.
    playbackState.playing = false;
    playbackState.phase = 'finished';
  });

  errorUnlisten = await listen<string>('playback-error', (event) => {
    engineAlive = false;
    playbackState.playing = false;
    playbackState.phase = 'paused';
    playbackState.errorMessage = event.payload;
  });

  noticeUnlisten = await listen<string>('playback-notice', (event) => {
    playbackState.notice = event.payload;
  });

  void syncTafsirVoiceState();
  tafsirPollTimer = setInterval(() => {
    if (tafsirVoiceState.activeKey) {
      void syncTafsirVoiceState();
    }
  }, 500);
}

/**
 * The tafsir voice button lives in a different component tree than the
 * playback controls, so the authoritative "still speaking" flag is polled from
 * the backend. Without this the button stays stuck on "Membacakan..." after
 * playback ends.
 */
async function syncTafsirVoiceState() {
  try {
    const active = await invoke<boolean>('is_tafsir_voice_active');
    if (!active && tafsirVoiceState.activeKey) {
      tafsirVoiceState.activeKey = '';
      tafsirVoiceState.paused = false;
      tafsirState.speakingKey = '';
      return;
    }

    if (active) {
      tafsirVoiceState.paused = await invoke<boolean>('is_tafsir_voice_paused');
    }
  } catch {
    // Backend not ready yet; the next tick retries.
  }
}

export function destroyPlaybackListeners() {
  progressUnlisten?.();
  finishedUnlisten?.();
  errorUnlisten?.();
  noticeUnlisten?.();
  if (tafsirPollTimer) {
    clearInterval(tafsirPollTimer);
    tafsirPollTimer = null;
  }
  progressUnlisten = null;
  finishedUnlisten = null;
  errorUnlisten = null;
  noticeUnlisten = null;
}

export async function startPlayback(
  surahNumber: number,
  fromAyah?: number,
  toAyah?: number,
  totalAyat = 0
) {
  const from = fromAyah ?? 1;
  playbackState.playing = true;
  playbackState.phase = 'recitation';
  playbackState.sessionSurah = surahNumber;
  playbackState.currentAyah = from;
  playbackState.current = from;
  playbackState.total = totalAyat;
  playbackState.errorMessage = '';
  playbackState.notice = '';
  phaseBeforePause = 'recitation';
  session = { surah: surahNumber, from, to: toAyah, total: totalAyat };

  try {
    // English playback reads the tafsir, so make sure it is loaded and attached
    // to the selected surah before the engine starts. A failure is not fatal:
    // the backend falls back to the verse translation.
    if (playbackState.lang === 'en') {
      await tafsirTextForAyah(surahNumber, from).catch(() => '');
    }

    await invoke('start_playback', {
      surahNumber,
      qari: playbackState.qari,
      lang: playbackState.lang,
      fromAyah: fromAyah ?? null,
      toAyah: toAyah ?? null,
      ttsEnabled: playbackState.ttsEnabled,
      readTafsir: playbackState.readTafsir
    });
    engineAlive = true;
  } catch (error) {
    // Keep the bar (paused) so the failure is visible in place and can be
    // retried, instead of leaving a silent button and an unhandled rejection.
    engineAlive = false;
    playbackState.playing = false;
    playbackState.phase = 'paused';
    playbackState.errorMessage = readableError(error, 'Unable to start playback.');
  }
}

/** Explicit "listen to this surah": always a fresh session from `from`. */
export function playSurah(surahNumber: number, totalAyat: number, fromAyah = 1) {
  void startPlayback(surahNumber, fromAyah, undefined, totalAyat);
}

/** "Listen from this ayah" continues through the end of the surah. */
export function playFromAyah(surahNumber: number, ayahNumber: number, totalAyat: number) {
  void startPlayback(surahNumber, ayahNumber, undefined, totalAyat);
}

export async function pausePlayback() {
  if (!engineAlive) {
    return;
  }
  if (playbackState.phase !== 'paused' && playbackState.phase !== 'finished') {
    phaseBeforePause = playbackState.phase;
  }
  await invoke('pause_playback');
  playbackState.playing = false;
  playbackState.phase = 'paused';
}

export async function resumePlayback() {
  // After a failed session the engine is gone but the bar still offers resume;
  // rebuild it at the same ayah instead of flipping a dead flag.
  if (!engineAlive) {
    if (!session) {
      return;
    }
    const kept = session;
    await startPlayback(kept.surah, playbackState.currentAyah, kept.to, kept.total);
    session = kept;
    return;
  }

  await invoke('resume_playback');
  playbackState.playing = true;
  playbackState.phase = phaseBeforePause || 'recitation';
}

/**
 * Ends the session for good.
 *
 * Pause keeps the engine alive so resume continues from the same sample; end
 * cancels it and dismisses the bar.
 */
export async function endPlayback() {
  await invoke('stop_playback');
  engineAlive = false;
  session = null;
  playbackState.playing = false;
  playbackState.phase = 'idle';
  playbackState.sessionSurah = 0;
  playbackState.errorMessage = '';
  playbackState.notice = '';
  tafsirVoiceState.activeKey = '';
  tafsirVoiceState.paused = false;
  tafsirState.speakingKey = '';
}

/** Retries the last ayah that failed, keeping the session's start point. */
export async function retryPlayback() {
  if (!session) {
    return;
  }
  const kept = session;
  await invoke('stop_playback').catch(() => undefined);
  await startPlayback(kept.surah, playbackState.currentAyah, kept.to, kept.total);
  session = kept;
}

export function togglePlayback(surahNumber: number, totalAyat = 0) {
  if (playbackState.playing) {
    void pausePlayback();
    return;
  }

  if (playbackState.phase === 'paused' && engineAlive) {
    void resumePlayback();
    return;
  }

  if (session && (playbackState.phase === 'paused' || playbackState.phase === 'finished')) {
    // Resume a dead engine or replay a finished session from where it began.
    const restartAt = playbackState.phase === 'finished' ? session.from : playbackState.currentAyah;
    const kept = session;
    void startPlayback(kept.surah, restartAt, kept.to, kept.total).then(() => {
      session = kept;
    });
    return;
  }

  void startPlayback(surahNumber, 1, undefined, totalAyat);
}

/** Moves the playhead inside the live session; never wraps past the ends. */
async function skipTo(ayahNumber: number) {
  if (!session) {
    return;
  }
  const kept = session;
  await invoke('stop_playback').catch(() => undefined);
  await startPlayback(kept.surah, ayahNumber, kept.to, kept.total);
  session = kept;
}

export function nextAyah(_surahNumber: number, totalAyat: number) {
  const total = playbackState.total || totalAyat;
  if (!session || playbackState.currentAyah >= total) {
    return;
  }
  void skipTo(playbackState.currentAyah + 1);
}

export function prevAyah(_surahNumber: number, _totalAyat: number) {
  if (!session || playbackState.currentAyah <= 1) {
    return;
  }
  void skipTo(playbackState.currentAyah - 1);
}

export function toggleRepeat() {
  playbackState.repeat = !playbackState.repeat;
}

export function toggleShuffle() {
  playbackState.shuffle = !playbackState.shuffle;
}

export function setQari(qari: string) {
  playbackState.qari = qari;
  playbackState.qariLabel = QARI_MAP[qari] ?? qari;
  persistSettings({ qari });
}

export function setLang(lang: string) {
  playbackState.lang = lang;
  playbackState.langLabel = LANG_MAP[lang] ?? lang;
  persistSettings({ lang });
}

export function setPrefetch(prefetch: boolean) {
  playbackState.prefetch = prefetch;
  persistSettings({ prefetchTts: prefetch });
}

export function setReadTafsir(readTafsir: boolean) {
  playbackState.readTafsir = readTafsir;
  persistSettings({ readTafsir });
}

/** Remembers the surah the reader is on, without touching other preferences. */
export async function rememberLastSurah(surahNumber: number) {
  playbackState.lastSurah = surahNumber;
  await loadSettings();
  persistSettings({ lastSurah: surahNumber });
}

let settingsReady: Promise<void> | null = null;

/**
 * Hydrates the interface from the persisted settings, once.
 *
 * Without this the app always opened with the defaults and the next save
 * would write those defaults back, so a chosen qari or language never
 * survived a restart. The promise is memoized so callers can await it before
 * writing, which keeps a navigation from racing the initial load.
 */
export function loadSettings(): Promise<void> {
  settingsReady ??= (async () => {
    try {
      const settings = await invoke<PlaybackSettingsPayload>('get_settings');
      if (settings.qari) {
        playbackState.qari = settings.qari;
        playbackState.qariLabel = QARI_MAP[settings.qari] ?? settings.qari;
      }
      if (settings.lang) {
        playbackState.lang = settings.lang;
        playbackState.langLabel = LANG_MAP[settings.lang] ?? settings.lang;
      }
      playbackState.prefetch = settings.prefetchTts ?? true;
      playbackState.readTafsir = settings.readTafsir ?? false;
      playbackState.lastSurah = settings.lastSurah ?? 0;
    } catch {
      // Settings are optional; the defaults above keep the app usable.
    }
  })();
  return settingsReady;
}

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
    playbackState.ttsVoiceMessage = readableError(error, 'Unable to check Supertonic 3 voice status.');
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

function applyTtsVoiceStatus(result: TtsVoiceStatusPayload) {
  playbackState.ttsVoiceStatus = result.ready ? 'ready' : (result.status as TtsVoiceStatus);
  playbackState.ttsVoiceMessage = result.message;

  if (!result.ready) {
    playbackState.ttsEnabled = false;
  }
}

function readableError(error: unknown, fallback: string) {
  if (typeof error === 'string' && error.trim()) {
    return error;
  }

  if (error instanceof Error && error.message.trim()) {
    return error.message;
  }

  return fallback;
}
