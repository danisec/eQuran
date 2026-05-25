import { invoke } from '@tauri-apps/api/core';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';

type PlaybackProgress = {
  surah: number;
  ayah: number;
  phase: string;
  current: number;
  total: number;
};

type NaturalIndonesianVoiceStatusPayload = {
  status: string;
  ready: boolean;
  canDownload: boolean;
  message: string;
};

type NaturalIndonesianInstallProgressPayload = {
  step: string;
  message: string;
  percent: number | null;
};

export type NaturalIndonesianStatus = 'checking' | 'missing' | 'installing' | 'ready' | 'failed';

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
  total: 0,
  naturalIndonesianStatus: 'checking' as NaturalIndonesianStatus,
  naturalIndonesianMessage: 'Checking Natural Indonesian Voice status.',
  naturalIndonesianProgress: null as number | null,
  naturalIndonesianInstalling: false,
  naturalIndonesianCanDownload: false
});

let progressUnlisten: UnlistenFn | null = null;
let finishedUnlisten: UnlistenFn | null = null;
let voiceInstallUnlisten: UnlistenFn | null = null;
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

  voiceInstallUnlisten = await listen<NaturalIndonesianInstallProgressPayload>(
    'natural-indonesian-install-progress',
    (event) => {
      playbackState.naturalIndonesianMessage = event.payload.message;
      playbackState.naturalIndonesianProgress = event.payload.percent;

      if (event.payload.step === 'ready') {
        playbackState.naturalIndonesianStatus = 'ready';
        playbackState.naturalIndonesianInstalling = false;
        playbackState.naturalIndonesianCanDownload = false;
      } else if (event.payload.step === 'failed') {
        playbackState.naturalIndonesianStatus = 'failed';
        playbackState.naturalIndonesianInstalling = false;
        playbackState.naturalIndonesianCanDownload = true;
        playbackState.ttsEnabled = false;
      } else {
        playbackState.naturalIndonesianStatus = 'installing';
        playbackState.naturalIndonesianInstalling = true;
      }
    }
  );
}

export function destroyPlaybackListeners() {
  progressUnlisten?.();
  finishedUnlisten?.();
  voiceInstallUnlisten?.();
  progressUnlisten = null;
  finishedUnlisten = null;
  voiceInstallUnlisten = null;
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
  if (enabled && playbackState.naturalIndonesianStatus !== 'ready') {
    playbackState.ttsEnabled = false;
    playbackState.naturalIndonesianMessage = 'Download Natural Indonesian Voice before enabling TTS Translation.';
    return;
  }

  playbackState.ttsEnabled = enabled;
  void invoke('set_tts_enabled', { enabled });
}

export async function refreshNaturalIndonesianVoiceStatus() {
  playbackState.naturalIndonesianStatus = 'checking';
  playbackState.naturalIndonesianMessage = 'Checking Natural Indonesian Voice status.';

  try {
    const result = await invoke<NaturalIndonesianVoiceStatusPayload>('get_natural_indonesian_voice_status');
    applyNaturalIndonesianVoiceStatus(result);
  } catch (error) {
    playbackState.naturalIndonesianStatus = 'failed';
    playbackState.naturalIndonesianMessage = readableError(error, 'Unable to check Natural Indonesian Voice status.');
    playbackState.naturalIndonesianCanDownload = true;
    playbackState.ttsEnabled = false;
  }
}

export async function installNaturalIndonesianVoice() {
  playbackState.naturalIndonesianStatus = 'installing';
  playbackState.naturalIndonesianInstalling = true;
  playbackState.naturalIndonesianProgress = 0;
  playbackState.naturalIndonesianMessage = 'Preparing Natural Indonesian Voice...';

  try {
    await invoke('install_natural_indonesian_voice');
    await refreshNaturalIndonesianVoiceStatus();
  } catch (error) {
    playbackState.naturalIndonesianStatus = 'failed';
    playbackState.naturalIndonesianInstalling = false;
    playbackState.naturalIndonesianCanDownload = true;
    playbackState.naturalIndonesianProgress = null;
    playbackState.naturalIndonesianMessage = readableError(
      error,
      'Natural Indonesian Voice setup failed. Check your connection and try again.'
    );
    playbackState.ttsEnabled = false;
  }
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

function applyNaturalIndonesianVoiceStatus(result: NaturalIndonesianVoiceStatusPayload) {
  playbackState.naturalIndonesianStatus = result.ready ? 'ready' : 'missing';
  playbackState.naturalIndonesianMessage = result.message;
  playbackState.naturalIndonesianCanDownload = result.canDownload;
  playbackState.naturalIndonesianInstalling = false;
  playbackState.naturalIndonesianProgress = result.ready ? 100 : null;

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
