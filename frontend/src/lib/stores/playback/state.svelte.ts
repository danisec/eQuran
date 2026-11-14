/**
 * Shared playback state and the maps that turn ids into labels.
 *
 * Everything here is read by more than one of the playback modules
 * (session, settings, status), so it lives on its own to keep the dependency
 * direction one-way: those modules import this, never each other.
 */

export type TtsVoiceStatus = 'checking' | 'ready' | 'missingRuntime' | 'missingScript' | 'failed';

export type PlaybackProgress = {
  surah: number;
  ayah: number;
  phase: string;
  current: number;
  total: number;
};

export type TtsVoiceStatusPayload = {
  status: string;
  ready: boolean;
  message: string;
};

export const QARI_MAP: Record<string, string> = {
  misyari: 'Mishary Rashid Alafasy',
  sudais: 'Abdurrahman As-Sudais',
  dossari: 'Ibrahim Al-Dossari',
  juhany: 'Abdullah Al-Juhany',
  qasim: 'Abdul Muhsin Al-Qasim',
  yasser: 'Yasser Al-Dosari'
};

export const LANG_MAP: Record<string, string> = {
  id: 'Indonesian',
  en: 'English'
};

/**
 * Volume is not exposed yet: `save_settings` still requires the field, so it is
 * written as a constant rather than left to drift per call site.
 */
export const DEFAULT_VOLUME = 70;

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
  /** Bytes the media cache holds, and its cap (`null` when uncapped). */
  cacheUsageBytes: 0,
  cacheLimitBytes: null as number | null,
  cacheFileCount: 0,
  cacheQariBytes: 0,
  cacheTranslationBytes: 0,
  cacheTafsirBytes: 0,
  current: 0,
  total: 0,
  ttsVoiceStatus: 'checking' as TtsVoiceStatus,
  ttsVoiceMessage: 'Checking Supertonic 3 voice status.'
});
