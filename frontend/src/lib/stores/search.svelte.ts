/**
 * Full-text search over the local Quran index.
 *
 * The index is built lazily by the backend on the first query, so this store
 * also owns the "first run takes a while" state and the progress events the
 * backend emits while it builds.
 */
import { invoke } from '@tauri-apps/api/core';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';

import { locale, type Locale } from '$lib/i18n';

export type SearchHit = {
  surah: number;
  ayah: number;
  surahNameLatin: string;
  score: number;
  snippet: string;
  matchedField: string;
};

export type IndexStatus = {
  ready: boolean;
  ayahCount: number;
  builtAt: number | null;
  building: boolean;
};

type IndexProgress = {
  done: number;
  total: number;
};

/**
 * Topic chips, each one a query the index answers well. The chip is a promise:
 * it must be a term the active translation actually contains, so English chips
 * search the English field and Indonesian chips the Indonesian one.
 */
const TOPIC_QUERIES: Record<Locale, readonly string[]> = {
  id: ['sabar', 'syukur', 'rezeki', 'orang tua', 'amanah', 'taubat', 'sedekah', 'ilmu'],
  en: ['patience', 'gratitude', 'provision', 'parents', 'trust', 'repentance', 'charity', 'knowledge']
};

/** Topic chips for the language the reader chose. */
export function topicQueries(): readonly string[] {
  return TOPIC_QUERIES[locale()];
}

export const searchState = $state({
  query: '',
  hits: [] as SearchHit[],
  searching: false,
  building: false,
  buildDone: 0,
  buildTotal: 0,
  hasSearched: false,
  error: '',
  indexedAyah: 0
});

let progressUnlisten: UnlistenFn | null = null;
let debounceTimer: ReturnType<typeof setTimeout> | null = null;

/** Subscribe to index build progress. Safe to call more than once. */
export async function initSearchListeners() {
  if (progressUnlisten) return;
  progressUnlisten = await listen<IndexProgress>('index-progress', (event) => {
    searchState.buildDone = event.payload.done;
    searchState.buildTotal = event.payload.total;
  });
}

export function destroySearchListeners() {
  progressUnlisten?.();
  progressUnlisten = null;
  if (debounceTimer) {
    clearTimeout(debounceTimer);
    debounceTimer = null;
  }
}

/** Load the index status without triggering a build. */
export async function refreshIndexStatus() {
  try {
    const status = await invoke<IndexStatus>('get_index_status');
    searchState.indexedAyah = status.ayahCount;
    searchState.building = status.building;
  } catch {
    searchState.indexedAyah = 0;
  }
}

/** Run a search immediately, building the index if this is the first run. */
export async function runSearch(query: string, limit = 40) {
  const trimmed = query.trim();
  searchState.query = query;
  searchState.error = '';

  if (trimmed.length < 2) {
    searchState.hits = [];
    searchState.hasSearched = false;
    searchState.searching = false;
    return;
  }

  searchState.searching = true;
  searchState.building = searchState.indexedAyah === 0;

  try {
    // The language decides which index fields count, so an English query never
    // matches Indonesian tafsir text.
    const hits = await invoke<SearchHit[]>('search_quran', {
      query: trimmed,
      limit,
      lang: locale()
    });
    searchState.hits = hits;
    searchState.hasSearched = true;
    // The first successful query is also the moment the index becomes real,
    // so refresh the count instead of guessing at it.
    void refreshIndexStatus();
  } catch (error) {
    searchState.error = typeof error === 'string' ? error : String(error);
    searchState.hits = [];
    searchState.hasSearched = true;
  } finally {
    searchState.searching = false;
    searchState.building = false;
  }
}

/** Debounce keystrokes so a search runs once the reader pauses. */
export function scheduleSearch(query: string, delay = 260) {
  searchState.query = query;
  if (debounceTimer) clearTimeout(debounceTimer);
  debounceTimer = setTimeout(() => {
    void runSearch(query);
  }, delay);
}

/** A topic chip runs its query at once. */
export function searchTopic(topic: string) {
  if (debounceTimer) clearTimeout(debounceTimer);
  void runSearch(topic);
}

export function clearSearch() {
  if (debounceTimer) clearTimeout(debounceTimer);
  searchState.query = '';
  searchState.hits = [];
  searchState.hasSearched = false;
  searchState.error = '';
}
