<script lang="ts">
  import { onMount } from 'svelte';
  import Icon from '$lib/components/Icon.svelte';
  import { t } from '$lib/i18n';
  import { bookmarkState, loadBookmarks } from '$lib/stores/bookmarks.svelte';
  import { playFromAyah, playbackState } from '$lib/stores/playback.svelte';
  import {
    clearSearch,
    initSearchListeners,
    refreshIndexStatus,
    runSearch,
    scheduleSearch,
    searchState,
    searchTopic,
    TOPIC_QUERIES
  } from '$lib/stores/search.svelte';
  import { loadLastSurah, loadSurahList, selectSurah, selectSurahAyah, surahState } from '$lib/stores/surah.svelte';
  import { allSurahs, matchSurahs } from '$lib/stores/surahSearch.svelte';
  import { showReader } from '$lib/stores/view.svelte';
  import { openMarks } from '$lib/stores/panels.svelte';

  let searchInput = $state<HTMLInputElement | undefined>(undefined);
  let surahMatches = $derived(matchSurahs(searchState.query));
  let searched = $derived(searchState.hasSearched || searchState.hits.length > 0 || surahMatches.length > 0);

  onMount(() => {
    void initSearchListeners();
    void refreshIndexStatus();
    void loadBookmarks();
    void loadSurahList();
    void loadLastSurah();
  });

  let totalBookmarks = $derived(bookmarkState.list.length);
  // The backend appends on add, so the last entry is the most recent mark.
  let lastBookmark = $derived(bookmarkState.list[bookmarkState.list.length - 1]);
  let surahIndex = $derived(allSurahs());

  /** Open a surah from the index or from a search hit. */
  async function openSurah(number: number) {
    await selectSurah(number);
    showReader();
  }

  /** Open a verse in the reader and scroll to it. */
  async function openVerse(surah: number, ayah: number) {
    await selectSurahAyah(surah, ayah);
    showReader();
  }

  /** Play a verse from the results without leaving the list first. */
  async function playVerse(surah: number, ayah: number) {
    await selectSurahAyah(surah, ayah);
    showReader();
    const total = surahIndex.find((entry) => entry.nomor === surah)?.jumlahAyat ?? 0;
    playFromAyah(surah, ayah, total);
  }

  async function continueReading() {
    await loadLastSurah();
    showReader();
  }

  function handleSearchKey(event: KeyboardEvent) {
    if (event.key === 'Enter') {
      event.preventDefault();
      void runSearch(searchState.query);
    }
    if (event.key === 'Escape') {
      searchInput?.blur();
      clearSearch();
    }
  }

  /**
   * Splits a snippet into plain and matching segments so the query words can be
   * marked. Matching is case-insensitive and per word; the snippet comes from
   * the corpus, never from the reader, and Svelte escapes each segment, so this
   * stays safe without injecting HTML.
   */
  function segments(text: string): Array<{ text: string; match: boolean }> {
    const words = searchState.query
      .trim()
      .toLowerCase()
      .split(/\s+/)
      .filter((word) => word.length > 1);
    if (words.length === 0) return [{ text, match: false }];

    const parts: Array<{ text: string; match: boolean }> = [];
    let index = 0;
    while (index < text.length) {
      let best = -1;
      let bestLength = 0;
      for (const word of words) {
        const found = text.toLowerCase().indexOf(word, index);
        if (found !== -1 && (best === -1 || found < best)) {
          best = found;
          bestLength = word.length;
        }
      }
      if (best === -1) {
        parts.push({ text: text.slice(index), match: false });
        break;
      }
      if (best > index) parts.push({ text: text.slice(index, best), match: false });
      parts.push({ text: text.slice(best, best + bestLength), match: true });
      index = best + bestLength;
    }
    return parts;
  }
</script>

<svelte:window
  onkeydown={(event) => {
    if ((event.metaKey || event.ctrlKey) && event.key === 'k') {
      event.preventDefault();
      searchInput?.focus();
    }
  }}
/>

<section class="h-full overflow-y-auto px-8">
  <div
    class="flex min-h-full w-full flex-col pt-6 pb-10 {playbackState.playing ||
    playbackState.phase === 'paused' ||
    playbackState.phase === 'finished'
      ? 'pb-28'
      : ''}"
  >
    <!-- The search is the page: it gets the whole width and the top of the fold -->
    <header class="shrink-0">
      <h1 class="text-[32px] font-medium leading-none tracking-[-0.02em] text-[var(--text)]">
        {t('home.title')}
      </h1>
      <p class="mt-3 max-w-[52ch] text-[15px] text-[var(--text-secondary)]">{t('home.subtitle')}</p>

      <div class="home-search mt-6">
        <Icon name="search" size={20} />
        <input
          bind:this={searchInput}
          value={searchState.query}
          oninput={(event) => scheduleSearch((event.currentTarget as HTMLInputElement).value)}
          onkeydown={handleSearchKey}
          placeholder={t('home.search')}
          aria-label={t('home.search')}
        />
        <span class="keycap">⌘K</span>
      </div>

      <div class="chip-row mt-4">
        {#each TOPIC_QUERIES as topic}
          <button class="chip" type="button" onclick={() => searchTopic(topic)}>{topic}</button>
        {/each}
      </div>
    </header>

    <div class="mt-8 flex flex-col">
      {#if surahMatches.length > 0}
        <!-- A surah match is almost always what the reader wanted, so it leads -->
        <h2 class="section-label mb-3">{t('home.surahResults')}</h2>
        <div class="mb-8 grid grid-cols-2 gap-3 xl:grid-cols-3">
          {#each surahMatches as surah (surah.nomor)}
            <button class="surah-tile" type="button" onclick={() => openSurah(surah.nomor)}>
              <span class="nav-row-index">{String(surah.nomor).padStart(3, '0')}</span>
              <span class="min-w-0 flex-1">
                <span class="surah-tile-name">{surah.namaLatin}</span>
                <span class="meta mt-1 block">{surah.arti} · {surah.jumlahAyat} {t('play.ayahCount')}</span>
              </span>
              <span class="surah-tile-arabic arabic">{surah.nama}</span>
            </button>
          {/each}
        </div>
      {/if}

      {#if searchState.building || searchState.searching}
        <p class="flex items-center gap-3 text-[14px] text-[var(--text-secondary)]">
          <Icon name="loader" size={18} class="animate-spin" />
          {#if searchState.building && searchState.buildTotal > 0}
            <span>{t('home.indexBuilding', searchState.buildDone, searchState.buildTotal)}</span>
          {:else}
            <span>{t('home.searching')}</span>
          {/if}
        </p>
      {:else if searchState.error}
        <p class="text-[14px] text-[var(--mark)]">{searchState.error}</p>
      {:else if searched && searchState.hits.length === 0 && surahMatches.length === 0}
        <div>
          <p class="text-[15px] font-medium text-[var(--text)]">{t('home.noResults')}</p>
          <p class="meta mt-1">{t('home.noResultsHint')}</p>
        </div>
      {:else if searchState.hits.length > 0}
        <div class="mb-3 flex items-baseline justify-between">
          <h2 class="section-label">{t('home.results', searchState.hits.length)}</h2>
          <button class="button is-quiet" type="button" onclick={clearSearch}>
            <Icon name="close" size={14} />
            {t('common.clear')}
          </button>
        </div>
        <div class="flex flex-col gap-3">
          {#each searchState.hits as hit (hit.surah + ':' + hit.ayah)}
            <article class="result-row">
              <div class="min-w-0">
                <p class="result-ref">{hit.surahNameLatin} {hit.surah}:{hit.ayah}</p>
                {#if hit.matchedField === 'arab'}
                  <p class="result-arabic arabic">{hit.snippet}</p>
                {:else}
                  <p class="result-snippet">
                    {#each segments(hit.snippet) as segment}
                      {#if segment.match}<mark>{segment.text}</mark>{:else}{segment.text}{/if}
                    {/each}
                  </p>
                {/if}
              </div>
              <div class="result-actions">
                <button class="button" type="button" onclick={() => openVerse(hit.surah, hit.ayah)}>
                  {t('home.resultOpen')}
                </button>
                <button
                  class="button is-on"
                  type="button"
                  onclick={() => playVerse(hit.surah, hit.ayah)}
                  aria-label="{t('home.resultPlay')} {hit.surah}:{hit.ayah}"
                >
                  <Icon name="play" size={16} filled />
                </button>
              </div>
            </article>
          {/each}
        </div>
      {:else}
        <!-- Idle: what the reader most likely wants next, then the whole index -->
        <div class="grid grid-cols-[minmax(0,1.25fr)_minmax(0,1fr)] gap-4">
          <button class="continue-card" type="button" onclick={continueReading}>
            <span class="min-w-0">
              <span class="section-label">{t('home.continue')}</span>
              <span class="mt-2 block text-[20px] font-medium text-[var(--text)]">
                {surahState.selected.namaLatin}
              </span>
              <span class="meta mt-1 block">
                {surahState.selected.jumlahAyat} {t('play.ayahCount')} · {surahState.selected.tempatTurun ===
                'Madinah'
                  ? 'Madaniyah'
                  : 'Makkiyah'}
              </span>
            </span>
            <Icon name="arrow-right" size={20} />
          </button>

          {#if lastBookmark}
            <button
              class="continue-card"
              type="button"
              onclick={() => openMarks()}
            >
              <span class="min-w-0">
                <span class="section-label">{t('home.lastBookmark')}</span>
                <span class="mt-2 block text-[15px] font-medium text-[var(--text)]">
                  {lastBookmark.surahName} : {lastBookmark.ayah}
                </span>
                <span class="arabic mt-1 block truncate text-[17px] text-[var(--text-secondary)]">
                  {lastBookmark.ayahText}
                </span>
              </span>
              <Icon name="bookmark-filled" size={18} />
            </button>
          {:else}
            <div class="continue-card cursor-default">
              <span class="min-w-0">
                <span class="section-label">{t('nav.bookmarks')}</span>
                <span class="mt-2 block text-[15px] text-[var(--text-muted)]">{t('nav.noBookmarks')}</span>
                <span class="meta mt-1 block">{t('nav.noBookmarksHint')}</span>
              </span>
            </div>
          {/if}
        </div>

        <p class="meta mt-4 flex flex-wrap items-center gap-x-2 gap-y-1">
          <span class="tnum">
            {searchState.indexedAyah > 0 ? searchState.indexedAyah.toLocaleString('id-ID') : '—'}
          </span>
          <span>{t('home.statAyah')}</span>
          <span aria-hidden="true">·</span>
          <span class="tnum">{totalBookmarks}</span>
          <span>{t('home.statBookmarks')}</span>
          <span aria-hidden="true">·</span>
          <span>{t('home.hint')}</span>
        </p>

        <!-- The index of the book: 114 surahs, the thing a mushaf opens with -->
        <h2 class="section-label mb-3 mt-8">{t('home.surahIndex')}</h2>
        <div class="grid grid-cols-2 gap-3 xl:grid-cols-3">
          {#each surahIndex as surah (surah.nomor)}
            <button
              class="surah-tile {surahState.selected.nomor === surah.nomor ? 'is-current' : ''}"
              type="button"
              onclick={() => openSurah(surah.nomor)}
            >
              <span class="nav-row-index">{String(surah.nomor).padStart(3, '0')}</span>
              <span class="min-w-0 flex-1">
                <span class="surah-tile-name">{surah.namaLatin}</span>
                <span class="meta mt-1 block">{surah.arti} · {surah.jumlahAyat} {t('play.ayahCount')}</span>
              </span>
              <span class="surah-tile-arabic arabic">{surah.nama}</span>
            </button>
          {/each}
        </div>
      {/if}
    </div>
  </div>
</section>
