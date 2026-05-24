import { invoke } from '@tauri-apps/api/core';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';

type PlaybackProgress = {
  surah: number;
  ayah: number;
  phase: string;
  current: number;
  total: number;
};

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
  qari: 'misyari',
  qariLabel: 'Mishary Rashid Alafasy',
  lang: 'id',
  langLabel: 'Indonesian',
  prefetch: true,
  ttsEnabled: true,
  repeat: false,
  shuffle: false,
  cacheReady: 0,
  cacheTotal: 0,
  current: 0,
  total: 0
});

let progressUnlisten: UnlistenFn | null = null;
let finishedUnlisten: UnlistenFn | null = null;
let lastPlayedSurah = 1;

export async function initPlaybackListeners() {
  if (progressUnlisten || finishedUnlisten) {
    return;
  }

  progressUnlisten = await listen<PlaybackProgress>('playback-progress', (event) => {
    playbackState.currentAyah = event.payload.ayah;
    playbackState.phase = event.payload.phase;
    playbackState.current = event.payload.current;
    playbackState.total = event.payload.total;
  });

  finishedUnlisten = await listen('playback-finished', () => {
    if (playbackState.repeat && playbackState.playing) {
      void startPlayback(lastPlayedSurah, 1, undefined);
    } else {
      playbackState.playing = false;
      playbackState.phase = 'idle';
    }
  });
}

export function destroyPlaybackListeners() {
  progressUnlisten?.();
  finishedUnlisten?.();
  progressUnlisten = null;
  finishedUnlisten = null;
}

export async function startPlayback(surahNumber: number, fromAyah?: number, toAyah?: number) {
  playbackState.playing = true;
  playbackState.phase = 'recitation';
  lastPlayedSurah = surahNumber;
  try {
    await invoke('start_playback', {
      surahNumber,
      qari: playbackState.qari,
      lang: playbackState.lang,
      fromAyah: fromAyah ?? null,
      toAyah: toAyah ?? null,
      ttsEnabled: playbackState.ttsEnabled
    });
  } catch (error) {
    playbackState.playing = false;
    playbackState.phase = 'idle';
    throw error;
  }
}

export async function stopPlayback() {
  await invoke('stop_playback');
  playbackState.playing = false;
  playbackState.phase = 'paused';
}

export function togglePlayback(surahNumber: number, fromAyah?: number, toAyah?: number) {
  if (playbackState.playing) {
    void stopPlayback();
  } else {
    if (fromAyah !== undefined) {
      void startPlayback(surahNumber, fromAyah, toAyah);
    } else if (playbackState.phase === 'paused') {
      void startPlayback(surahNumber, playbackState.currentAyah, toAyah);
    } else {
      void startPlayback(surahNumber, 1, undefined);
    }
  }
}

export function nextAyah(surahNumber: number, totalAyat: number) {
  const next = playbackState.currentAyah < totalAyat ? playbackState.currentAyah + 1 : 1;
  void stopPlayback().then(() => startPlayback(surahNumber, next, undefined));
}

export function prevAyah(surahNumber: number, totalAyat: number) {
  const prev = playbackState.currentAyah > 1 ? playbackState.currentAyah - 1 : totalAyat;
  void stopPlayback().then(() => startPlayback(surahNumber, prev, undefined));
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
  void invoke('save_settings', {
    settings: { qari, lang: playbackState.lang, prefetchTts: playbackState.prefetch, volume: DEFAULT_VOLUME }
  });
}

export function setLang(lang: string) {
  playbackState.lang = lang;
  playbackState.langLabel = LANG_MAP[lang] ?? lang;
  void invoke('save_settings', {
    settings: { qari: playbackState.qari, lang, prefetchTts: playbackState.prefetch, volume: DEFAULT_VOLUME }
  });
}

export function setPrefetch(prefetch: boolean) {
  playbackState.prefetch = prefetch;
  void invoke('save_settings', {
    settings: { qari: playbackState.qari, lang: playbackState.lang, prefetchTts: prefetch, volume: DEFAULT_VOLUME, lastSurah: 0 }
  });
}

export function setTtsEnabled(enabled: boolean) {
  playbackState.ttsEnabled = enabled;
  void invoke('set_tts_enabled', { enabled });
}

export async function refreshCacheStatus(surahNumber: number) {
  try {
    const result = await invoke<{ cachedCount: number }>('get_cache_status', {
      surahNumber,
      qari: playbackState.qari
    });
    playbackState.cacheReady = result.cachedCount;
  } catch {
    playbackState.cacheReady = 0;
  }
}
