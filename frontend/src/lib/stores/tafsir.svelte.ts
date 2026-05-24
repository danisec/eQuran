import { invoke } from '@tauri-apps/api/core';

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

export async function loadTafsir(surahNumber: number) {
  if (tafsirState.cache.has(surahNumber)) {
    return tafsirState.cache.get(surahNumber);
  }

  tafsirState.loadingSurah = surahNumber;
  tafsirState.error = '';
  try {
    const tafsir = await invoke<TafsirSurah>('get_tafsir', { number: surahNumber });
    tafsirState.cache = new Map(tafsirState.cache).set(surahNumber, tafsir);
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

export async function playTafsirVoice(surahNumber: number, ayahNumber: number, text: string) {
  const key = `${surahNumber}:${ayahNumber}`;
  tafsirState.speakingKey = key;
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
  }
}
