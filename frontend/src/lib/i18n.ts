/**
 * The interface speaks the same language as the translation the reader chose.
 * One dictionary per locale, no runtime library — the surface is small enough
 * that a plain lookup with a typed key list stays readable and cannot drift.
 */
import { playbackState } from '$lib/stores/playback.svelte';

export type Locale = 'id' | 'en';

type Dictionary = Record<string, string>;

const id: Dictionary = {
  'nav.home': 'Beranda',
  'nav.read': 'Baca',
  'nav.bookmarks': 'Tanda',
  'nav.searchSurah': 'Cari surah',
  'nav.chooseSurah': 'Pilih surah',
  'nav.noBookmarks': 'Belum ada tanda',
  'nav.noBookmarksHint': 'Tandai ayat dari tombol di setiap ayat',
  'nav.removeBookmark': 'Hapus tanda',
  'nav.collapseSidebar': 'Ciutkan bilah navigasi',
  'nav.expandSidebar': 'Bentangkan bilah navigasi',

  'theme.toLight': 'Mode terang',
  'theme.toDark': 'Mode gelap',

  'marks.toggle': 'Tanda',
  'marks.close': 'Tutup panel tanda',
  'marks.ref': '%s %s:%s',

  'settings.title': 'Pengaturan',
  'settings.subtitle': 'Qari, bahasa, dan suara terjemahan',
  'settings.open': 'Buka pengaturan',
  'settings.close': 'Tutup pengaturan',
  'settings.qari': 'Qari',
  'settings.language': 'Bahasa terjemahan',
  'settings.voice': 'Suara terjemahan',
  'settings.voiceChecking': 'Memeriksa Supertonic 3…',
  'settings.voiceReady': 'Supertonic 3 siap membacakan terjemahan',
  'settings.voiceFailedHint': 'Pasang paket Python supertonic untuk mengaktifkan suara',
  'settings.tts': 'Bacakan terjemahan',
  'settings.ttsHint': 'Putar terjemahan setelah murattal',
  'settings.prefetch': 'Siapkan suara ayat berikutnya',
  'settings.prefetchHint': 'Sintesis lebih awal agar tidak menunggu',
  'settings.readTafsir': 'Bacakan tafsir',
  'settings.readTafsirHint': 'Lanjutkan ke tafsir setelah terjemahan',
  'settings.advanced': 'Lanjutan',
  'settings.advancedHint': 'Rincian penyiapan audio',
  'settings.cache': 'Cache audio',
  'settings.cacheNone': 'Belum ada audio qari tersimpan untuk surah ini',
  'settings.cacheReady': 'ayat punya audio qari',
  'settings.offline': 'Tersimpan untuk pemutaran offline',

  'play.stop': 'Berhenti',
  'play.phase.qari': 'Qari',
  'play.phase.translation': 'Terjemahan',
  'play.phase.tafsir': 'Tafsir',
  'play.ayahCount': 'ayat',
  'play.ayahLabel': 'ayat %s',
  'play.controls': 'Kontrol pemutaran',
  'play.progress': 'Kemajuan ayat',
  'play.progressValue': 'Ayat %s dari %s',
  'play.previous': 'Ayat sebelumnya',
  'play.next': 'Ayat berikutnya',
  'play.play': 'Putar',
  'play.pause': 'Jeda',
  'play.resume': 'Lanjutkan',
  'play.listen': 'Dengarkan',
  'play.replay': 'Putar ulang',
  'play.retry': 'Coba lagi',
  'play.finished': 'Surah selesai',
  'play.repeatOn': 'Ulangi pemutaran',
  'play.repeatOff': 'Matikan ulangan',
  'play.endSession': 'Akhiri pemutaran',

  'ayah.play': 'Putar ayat %s',
  'ayah.bookmark': 'Tandai ayat %s',
  'ayah.copy': 'Salin ayat %s',
  'ayah.tafsir': 'Tafsir',
  'ayah.tafsirTitle': 'Tafsir ayat %s',
  'ayah.tafsirLoading': 'Memuat tafsir…',
  'ayah.tafsirMissing': 'Tafsir belum tersedia untuk ayat ini',
  'ayah.tafsirVoice': 'Bacakan tafsir ayat %s',
  'ayah.tafsirPause': 'Jeda bacaan tafsir ayat %s',
  'ayah.tafsirResume': 'Lanjutkan bacaan tafsir ayat %s',
  'ayah.speaking': 'Membacakan…',
  'ayah.voice': 'Suara',
  'ayah.pause': 'Jeda',
  'ayah.resume': 'Lanjut',
  'ayah.playing': 'Diputar',
  'ayah.paused': 'Dijeda',

  'home.title': 'Al-Qur’an',
  'home.subtitle': 'Cari ayat, topik, atau tafsir — lalu buka atau putar',
  'home.search': 'Cari surah, ayat, topik, atau tafsir',
  'home.surahResults': 'Surah',
  'home.surahIndex': 'Daftar surah',
  'home.continue': 'Lanjut membaca',
  'home.lastBookmark': 'Tanda terakhir',
  'home.statAyah': 'ayat terindeks',
  'home.statBookmarks': 'ayat ditandai',
  'home.searching': 'Memuat indeks…',
  'home.noResults': 'Tidak ada ayat yang cocok',
  'home.noResultsHint': 'Coba kata lain, atau cari topik lewat tafsir',
  'home.results': '%s ayat',
  'home.resultOpen': 'Buka',
  'home.resultPlay': 'Putar',
  'home.indexBuilding': 'Menyiapkan indeks %s/%s',
  'home.hint': 'Coba cari: sabar, rezeki, orang tua, atau kutipan Arab',

  'focus.close': 'Tutup mode fokus',
  'focus.enter': 'Mode fokus',

  'common.close': 'Tutup',
  'common.clear': 'Hapus',
  'common.loading': 'Memuat…',
};

const en: Dictionary = {
  'nav.home': 'Home',
  'nav.read': 'Read',
  'nav.bookmarks': 'Marks',
  'nav.searchSurah': 'Search surah',
  'nav.chooseSurah': 'Choose surah',
  'nav.noBookmarks': 'No marks yet',
  'nav.noBookmarksHint': 'Mark a verse from the button on any verse',
  'nav.removeBookmark': 'Remove mark',
  'nav.collapseSidebar': 'Collapse navigation rail',
  'nav.expandSidebar': 'Expand navigation rail',

  'theme.toLight': 'Light mode',
  'theme.toDark': 'Dark mode',

  'marks.toggle': 'Marks',
  'marks.close': 'Close marks panel',
  'marks.ref': '%s %s:%s',

  'settings.title': 'Settings',
  'settings.subtitle': 'Reciter, language, and translation voice',
  'settings.open': 'Open settings',
  'settings.close': 'Close settings',
  'settings.qari': 'Reciter',
  'settings.language': 'Translation language',
  'settings.voice': 'Translation voice',
  'settings.voiceChecking': 'Checking Supertonic 3…',
  'settings.voiceReady': 'Supertonic 3 is ready to read translations',
  'settings.voiceFailedHint': 'Install the supertonic Python package to enable voice',
  'settings.tts': 'Read translation',
  'settings.ttsHint': 'Play the translation after the recitation',
  'settings.prefetch': 'Prepare the next verse voice',
  'settings.prefetchHint': 'Synthesise early so there is no wait',
  'settings.readTafsir': 'Read tafsir',
  'settings.readTafsirHint': 'Continue into the tafsir after the translation',
  'settings.advanced': 'Advanced',
  'settings.advancedHint': 'Audio preparation details',
  'settings.cache': 'Audio cache',
  'settings.cacheNone': 'No qari audio stored for this surah yet',
  'settings.cacheReady': 'verses have qari audio',
  'settings.offline': 'Stored for offline playback',

  'play.stop': 'Stop',
  'play.phase.qari': 'Reciter',
  'play.phase.translation': 'Translation',
  'play.phase.tafsir': 'Tafsir',
  'play.ayahCount': 'verses',
  'play.ayahLabel': 'verse %s',
  'play.controls': 'Playback controls',
  'play.progress': 'Verse progress',
  'play.progressValue': 'Verse %s of %s',
  'play.previous': 'Previous verse',
  'play.next': 'Next verse',
  'play.play': 'Play',
  'play.pause': 'Pause',
  'play.resume': 'Resume',
  'play.listen': 'Listen',
  'play.replay': 'Play again',
  'play.retry': 'Try again',
  'play.finished': 'Surah complete',
  'play.repeatOn': 'Repeat playback',
  'play.repeatOff': 'Turn off repeat',
  'play.endSession': 'End playback',

  'ayah.play': 'Play verse %s',
  'ayah.bookmark': 'Mark verse %s',
  'ayah.copy': 'Copy verse %s',
  'ayah.tafsir': 'Tafsir',
  'ayah.tafsirTitle': 'Tafsir of verse %s',
  'ayah.tafsirLoading': 'Loading tafsir…',
  'ayah.tafsirMissing': 'No tafsir is available for this verse yet',
  'ayah.tafsirVoice': 'Read tafsir of verse %s',
  'ayah.tafsirPause': 'Pause tafsir of verse %s',
  'ayah.tafsirResume': 'Resume tafsir of verse %s',
  'ayah.speaking': 'Reading…',
  'ayah.voice': 'Voice',
  'ayah.pause': 'Pause',
  'ayah.resume': 'Resume',
  'ayah.playing': 'Playing',
  'ayah.paused': 'Paused',

  'home.title': 'The Quran',
  'home.subtitle': 'Search verses, topics or tafsir — then open or play',
  'home.search': 'Search surahs, verses, topics or tafsir',
  'home.surahResults': 'Surahs',
  'home.surahIndex': 'Surah index',
  'home.continue': 'Continue reading',
  'home.lastBookmark': 'Latest mark',
  'home.statAyah': 'verses indexed',
  'home.statBookmarks': 'verses marked',
  'home.searching': 'Loading the index…',
  'home.noResults': 'No verse matches',
  'home.noResultsHint': 'Try another word, or search a topic through the tafsir',
  'home.results': '%s verses',
  'home.resultOpen': 'Open',
  'home.resultPlay': 'Play',
  'home.indexBuilding': 'Preparing the index %s/%s',
  'home.hint': 'Try: patience, provision, parents, or an Arabic quote',

  'focus.close': 'Close focus mode',
  'focus.enter': 'Focus mode',

  'common.close': 'Close',
  'common.clear': 'Clear',
  'common.loading': 'Loading…',
};

const dictionaries: Record<Locale, Dictionary> = { id, en };

/** Current interface locale, derived from the translation language. */
export function locale(): Locale {
  return playbackState.lang === 'en' ? 'en' : 'id';
}

/**
 * Translate a key for the active locale. `%s` placeholders are filled from
 * `values` in order. A missing key falls back to Indonesian, then to the key
 * itself, so a forgotten string is visible rather than blank.
 */
export function t(key: string, ...values: Array<string | number>): string {
  const current = locale();
  const template = dictionaries[current][key] ?? dictionaries.id[key] ?? key;
  if (values.length === 0) return template;
  let index = 0;
  return template.replace(/%s/g, () => String(values[index++]));
}
