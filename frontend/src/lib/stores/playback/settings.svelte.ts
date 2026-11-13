import { invoke } from '@tauri-apps/api/core';

import { LANG_MAP, QARI_MAP, playbackState } from './state.svelte';

type PlaybackSettingsPayload = {
  qari: string;
  lang: string;
  prefetchTts: boolean;
  readTafsir: boolean;
  volume: number;
  lastSurah: number;
};

const DEFAULT_VOLUME = 70;

/**
 * Writes the whole settings object, with optional overrides.
 *
 * `save_settings` replaces the payload outright, so every caller must send a
 * complete object. Building it in one place stops partial callers (such as
 * "remember the last surah" during navigation) from silently resetting the
 * reader's qari, language, or Read Tafsir preference.
 */
function persist(overrides: Partial<PlaybackSettingsPayload> = {}) {
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

export function setQari(qari: string) {
  playbackState.qari = qari;
  playbackState.qariLabel = QARI_MAP[qari] ?? qari;
  persist({ qari });
}

export function setLang(lang: string) {
  playbackState.lang = lang;
  playbackState.langLabel = LANG_MAP[lang] ?? lang;
  persist({ lang });
}

export function setPrefetch(prefetch: boolean) {
  playbackState.prefetch = prefetch;
  persist({ prefetchTts: prefetch });
}

export function setReadTafsir(readTafsir: boolean) {
  playbackState.readTafsir = readTafsir;
  persist({ readTafsir });
}

/** Remembers the surah the reader is on, without touching other preferences. */
export async function rememberLastSurah(surahNumber: number) {
  playbackState.lastSurah = surahNumber;
  await loadSettings();
  persist({ lastSurah: surahNumber });
}
