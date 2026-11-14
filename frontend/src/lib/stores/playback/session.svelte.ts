import { invoke } from '@tauri-apps/api/core';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';

import { tafsirState, tafsirVoiceState } from '../tafsir.svelte';
import { readableError } from './errors';
import { playbackState, type PlaybackProgress } from './state.svelte';

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
    // The backend loads the English tafsir itself before the session starts, so
    // playback must not wait on a second network round trip here: a slow tafsir
    // request used to leave the bar spinning with nothing playing.
    await invoke('start_playback', {
      request: {
        surahNumber,
        qari: playbackState.qari,
        lang: playbackState.lang,
        fromAyah: fromAyah ?? null,
        toAyah: toAyah ?? null,
        ttsEnabled: playbackState.ttsEnabled,
        readTafsir: playbackState.readTafsir
      }
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

let progressUnlisten: UnlistenFn | null = null;
let finishedUnlisten: UnlistenFn | null = null;
let errorUnlisten: UnlistenFn | null = null;
let noticeUnlisten: UnlistenFn | null = null;

export async function initPlaybackSessionListeners() {
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
}

export function destroyPlaybackSessionListeners() {
  progressUnlisten?.();
  finishedUnlisten?.();
  errorUnlisten?.();
  noticeUnlisten?.();
  progressUnlisten = null;
  finishedUnlisten = null;
  errorUnlisten = null;
  noticeUnlisten = null;
}
