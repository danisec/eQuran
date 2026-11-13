import { invoke } from '@tauri-apps/api/core';
import { surahState } from './surah.svelte';

export type TafsirAyah = {
  ayat: number;
  teks: string;
};

export type TafsirSurah = {
  nomor: number;
  nama: string;
  namaLatin: string;
  jumlahAyat: number;
  tempatTurun: string;
  arti: string;
  deskripsi: string;
  tafsir: TafsirAyah[];
};

export const tafsirState = $state({
  cache: new Map<number, TafsirSurah>(),
  loadingSurah: 0,
  speakingKey: '',
  error: ''
});

/** Ayah whose tafsir voice is currently being generated or played. */
export const tafsirVoiceState = $state({
  activeKey: '',
  paused: false
});

export function isTafsirVoiceActive(surahNumber: number, ayahNumber: number) {
  return tafsirVoiceState.activeKey === `${surahNumber}:${ayahNumber}`;
}

export async function loadTafsir(surahNumber: number) {
  if (tafsirState.cache.has(surahNumber)) {
    return tafsirState.cache.get(surahNumber);
  }

  tafsirState.loadingSurah = surahNumber;
  tafsirState.error = '';
  try {
    const tafsir = await invoke<TafsirSurah>('get_tafsir', { number: surahNumber });
    tafsirState.cache = new Map(tafsirState.cache).set(surahNumber, tafsir);
    attachTafsirToSelectedSurah(surahNumber);
    return tafsir;
  } catch (error) {
    tafsirState.error = error instanceof Error ? error.message : String(error);
    return undefined;
  } finally {
    if (tafsirState.loadingSurah === surahNumber) {
      tafsirState.loadingSurah = 0;
    }
  }
}

export function tafsirForAyah(surahNumber: number, ayahNumber: number) {
  return tafsirState.cache.get(surahNumber)?.tafsir.find((tafsir) => tafsir.ayat === ayahNumber);
}

/**
 * Returns the tafsir text for an ayah, loading the surah's tafsir on demand.
 *
 * Used by the translation voice so English playback can read the tafsir of the
 * ayah instead of the Indonesian verse translation.
 */
export async function tafsirTextForAyah(surahNumber: number, ayahNumber: number): Promise<string> {
  const cached = tafsirForAyah(surahNumber, ayahNumber);
  if (cached) {
    return cached.teks;
  }

  const surah = await loadTafsir(surahNumber);
  return surah?.tafsir.find((tafsir) => tafsir.ayat === ayahNumber)?.teks ?? '';
}

/**
 * Copies the loaded tafsir into `surahState.selected.ayat` so the playback
 * engine can read it without another round trip.
 */
export function attachTafsirToSelectedSurah(surahNumber: number) {
  const tafsir = tafsirState.cache.get(surahNumber);
  if (!tafsir || surahState.selected.nomor !== surahNumber) {
    return;
  }

  for (const ayah of surahState.selected.ayat) {
    const match = tafsir.tafsir.find((entry) => entry.ayat === ayah.nomorAyat);
    if (match) {
      ayah.tafsir = match.teks;
    }
  }
}

export async function playTafsirVoice(surahNumber: number, ayahNumber: number, text: string) {
  const key = `${surahNumber}:${ayahNumber}`;
  tafsirState.speakingKey = key;
  tafsirVoiceState.activeKey = key;
  tafsirVoiceState.paused = false;
  tafsirState.error = '';
  try {
    await invoke('play_tafsir_voice', {
      surahNumber,
      ayahNumber,
      text
    });
  } catch (error) {
    tafsirState.error = error instanceof Error ? error.message : String(error);
  } finally {
    if (tafsirState.speakingKey === key) {
      tafsirState.speakingKey = '';
    }
    if (tafsirVoiceState.activeKey === key) {
      tafsirVoiceState.activeKey = '';
      tafsirVoiceState.paused = false;
    }
  }
}

export async function pauseTafsirVoice() {
  try {
    await invoke('pause_tafsir_voice');
    tafsirVoiceState.paused = true;
  } catch (error) {
    tafsirState.error = error instanceof Error ? error.message : String(error);
  }
}

export async function resumeTafsirVoice() {
  try {
    await invoke('resume_tafsir_voice');
    tafsirVoiceState.paused = false;
  } catch (error) {
    tafsirState.error = error instanceof Error ? error.message : String(error);
  }
}

export async function stopTafsirVoice() {
  tafsirVoiceState.activeKey = '';
  tafsirVoiceState.paused = false;
  tafsirState.speakingKey = '';
  try {
    await invoke('stop_playback');
  } catch (error) {
    tafsirState.error = error instanceof Error ? error.message : String(error);
  }
}
