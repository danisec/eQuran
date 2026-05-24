import { invoke } from '@tauri-apps/api/core';

export type SurahListItem = {
  nomor: number;
  nama: string;
  namaLatin: string;
  jumlahAyat: number;
  tempatTurun: string;
  arti: string;
};

export type Ayat = {
  nomorAyat: number;
  teksArab: string;
  teksLatin: string;
  teksIndonesia: string;
  teksEnglish?: string;
};

export type EnglishAyah = {
  numberInSurah: number;
  textEnglish: string;
};

export type EnglishSurah = {
  ayahs: EnglishAyah[];
};

export type Surah = SurahListItem & {
  deskripsi: string;
  ayat: Ayat[];
};

const fallbackSurah: SurahListItem[] = [
  { nomor: 1, nama: 'الفاتحة', namaLatin: 'Al-Fatihah', jumlahAyat: 7, tempatTurun: 'Mekah', arti: 'Pembukaan' },
  { nomor: 2, nama: 'البقرة', namaLatin: 'Al-Baqarah', jumlahAyat: 286, tempatTurun: 'Madinah', arti: 'Sapi' },
  { nomor: 3, nama: 'اٰل عمران', namaLatin: 'Ali Imran', jumlahAyat: 200, tempatTurun: 'Madinah', arti: 'Keluarga Imran' },
  { nomor: 4, nama: 'النساۤء', namaLatin: 'An-Nisa', jumlahAyat: 176, tempatTurun: 'Madinah', arti: 'Wanita' }
];

const fallbackAlFatihah: Surah = {
  ...fallbackSurah[0],
  deskripsi: 'Surah pertama dalam Al-Quran.',
  ayat: [
    { nomorAyat: 1, teksArab: 'بِسْمِ اللّٰهِ الرَّحْمٰنِ الرَّحِيْمِ', teksLatin: 'Bismillāhir-raḥmānir-raḥīm(i).', teksIndonesia: 'Dengan nama Allah Yang Maha Pengasih lagi Maha Penyayang.' },
    { nomorAyat: 2, teksArab: 'اَلْحَمْدُ لِلّٰهِ رَبِّ الْعٰلَمِيْنَۙ', teksLatin: "Al-ḥamdu lillāhi rabbil-'ālamīn(a).", teksIndonesia: 'Segala puji bagi Allah, Tuhan semesta alam,' },
    { nomorAyat: 3, teksArab: 'الرَّحْمٰنِ الرَّحِيْمِۙ', teksLatin: 'Ar-raḥmānir-raḥīm(i).', teksIndonesia: 'Yang Maha Pengasih lagi Maha Penyayang,' },
    { nomorAyat: 4, teksArab: 'مٰلِكِ يَوْمِ الدِّيْنِۗ', teksLatin: 'Māliki yaumid-dīn(i).', teksIndonesia: 'Pemilik hari Pembalasan.' },
    { nomorAyat: 5, teksArab: 'اِيَّاكَ نَعْبُدُ وَاِيَّاكَ نَسْتَعِيْنُۗ', teksLatin: "Iyyāka na'budu wa iyyāka nasta'īn(u).", teksIndonesia: 'Hanya kepada Engkaulah kami menyembah dan hanya kepada Engkaulah kami mohon pertolongan.' },
    { nomorAyat: 6, teksArab: 'اِهْدِنَا الصِّرَاطَ الْمُسْتَقِيْمَۙ', teksLatin: 'Ihdinaṣ-ṣirāṭal-mustaqīm(a).', teksIndonesia: 'Tunjukilah kami jalan yang lurus,' },
    { nomorAyat: 7, teksArab: 'صِرَاطَ الَّذِيْنَ اَنْعَمْتَ عَلَيْهِمْ ەۙ غَيْرِ الْمَغْضُوْبِ عَلَيْهِمْ وَلَا الضَّاۤلِّيْنَ', teksLatin: "Ṣirāṭallażīna an'amta 'alaihim, ghairil-maghḍūbi 'alaihim wa laḍ-ḍāllīn(a).", teksIndonesia: '(yaitu) jalan orang-orang yang telah Engkau beri nikmat kepadanya; bukan (jalan) mereka yang dimurkai dan bukan (pula jalan) mereka yang sesat.' }
  ]
};

export const surahState = $state({
  list: fallbackSurah,
  selected: fallbackAlFatihah,
  query: '',
  pendingScrollAyah: 0,
  loading: false,
  error: ''
});

export async function loadSurahList() {
  surahState.loading = true;
  surahState.error = '';
  try {
    surahState.list = await invoke<SurahListItem[]>('get_surah_list');
  } catch (error) {
    surahState.error = error instanceof Error ? error.message : String(error);
  } finally {
    surahState.loading = false;
  }
}

export async function loadLastSurah() {
  try {
    const settings = await invoke<{ lastSurah: number }>('get_settings');
    if (settings.lastSurah && settings.lastSurah > 0) {
      await selectSurah(settings.lastSurah);
    }
  } catch {
  }
}

export async function selectSurah(number: number) {
  surahState.loading = true;
  surahState.error = '';
  try {
    surahState.selected = await invoke<Surah>('get_surah', { number });
    void loadEnglishTranslation(number);
    void invoke('save_settings', {
      settings: { qari: 'misyari', lang: 'id', prefetchTts: true, volume: 70, lastSurah: number }
    });
  } catch (error) {
    surahState.error = error instanceof Error ? error.message : String(error);
    if (number === fallbackAlFatihah.nomor) {
      surahState.selected = fallbackAlFatihah;
    }
  } finally {
    surahState.loading = false;
  }
}

export async function selectSurahAyah(surahNumber: number, ayahNumber: number) {
  surahState.pendingScrollAyah = ayahNumber;
  await selectSurah(surahNumber);
}

export function clearPendingScrollAyah() {
  surahState.pendingScrollAyah = 0;
}

async function loadEnglishTranslation(number: number) {
  try {
    const english = await invoke<EnglishSurah>('get_surah_english', { number });
    for (const ayah of surahState.selected.ayat) {
      const match = english.ayahs.find((e) => e.numberInSurah === ayah.nomorAyat);
      if (match) {
        ayah.teksEnglish = match.textEnglish;
      }
    }
  } catch {
  }
}
