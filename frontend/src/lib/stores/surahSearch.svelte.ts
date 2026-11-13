/**
 * Surah-level lookup for the home page.
 *
 * Matching surahs is a different job from searching verses: there are only 114
 * of them, so the work stays in the client and returns instantly, while the
 * verse index is asked in parallel. Both lists are shown together so "Al-Kahf"
 * opens the surah instead of listing verses that merely mention it.
 */
import { SURAH_META_EN } from '$lib/data/surahMeta';
import { surahState } from './surah.svelte';

export type SurahMatch = {
  nomor: number;
  nama: string;
  namaLatin: string;
  jumlahAyat: number;
  tempatTurun: string;
  arti: string;
};

/** Folds Latin diacritics, case, and hyphens so "al-kahf" matches "Al-Kahf". */
function fold(text: string): string {
  return text
    .normalize('NFD')
    .replace(/[\u0300-\u036f]/g, '')
    .toLowerCase()
    .replace(/[^a-z0-9\u0600-\u06ff]+/g, ' ')
    .trim();
}

/** Strips Arabic harakat so "الكهف" also matches a fully vocalised name. */
function foldArabic(text: string): string {
  return text
    .replace(/[\u064b-\u0652\u0670\u0640]/g, '')
    .replace(/\u0649/g, '\u064a')
    .replace(/\u0629/g, '\u0647')
    .trim();
}

/**
 * Returns the surahs whose number, Latin name, Arabic name, meaning, or
 * revelation place contains the query. Both language meanings are matched, so
 * "sapi" and "cow" both find Al-Baqarah whichever language the reader is in.
 * Exact number and prefix matches rank first; the rest keep canonical order.
 */
export function matchSurahs(query: string, limit = 6): SurahMatch[] {
  return matchSurahList(surahState.list, query, limit);
}

/**
 * Pure matcher, separated from the store so it can be tested without Svelte
 * runes. The UI calls [`matchSurahs`], which binds it to the loaded list.
 */
export function matchSurahList(list: SurahMatch[], query: string, limit = 6): SurahMatch[] {
  const raw = query.trim();
  if (raw.length === 0) return [];

  const plain = fold(raw);
  if (plain.length === 0) {
    // A pure Arabic query has no Latin folds left; match on the Arabic name.
    const arabic = foldArabic(raw);
    if (arabic.length === 0) return [];
    return list.filter((surah) => foldArabic(surah.nama).includes(arabic)).slice(0, limit);
  }

  const asNumber = /^\d{1,3}$/.test(plain) ? Number(plain) : null;
  const arabicQuery = foldArabic(raw);

  const scored = list
    .map((surah) => {
      const latin = fold(surah.namaLatin);
      const arti = fold(surah.arti);
      const meta = SURAH_META_EN[surah.nomor];
      const meaningEn = meta ? fold(meta.meaning) : '';
      const revelationEn = meta ? meta.revelation.toLowerCase() : '';
      const arabic = foldArabic(surah.nama);
      let score = 0;

      if (asNumber !== null && surah.nomor === asNumber) score = 100;
      else if (latin === plain) score = 90;
      else if (latin.startsWith(plain)) score = 70;
      else if (latin.includes(plain)) score = 50;
      else if (arti.includes(plain) || meaningEn.includes(plain)) score = 30;
      else if (arabicQuery.length > 0 && arabic.includes(arabicQuery)) score = 40;
      else if (fold(surah.tempatTurun).includes(plain) || revelationEn.includes(plain)) score = 10;

      return { surah, score };
    })
    .filter((entry) => entry.score > 0);

  scored.sort((a, b) => b.score - a.score || a.surah.nomor - b.surah.nomor);
  return scored.slice(0, limit).map((entry) => entry.surah);
}

/** Every surah, used for the index grid on the home page. */
export function allSurahs(): SurahMatch[] {
  return surahState.list;
}
