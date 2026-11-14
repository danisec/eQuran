<script lang="ts">
  import { onMount } from 'svelte';
  import Icon from '$lib/components/Icon.svelte';
  import { JUZ_RANGES, revelationOf } from '$lib/data/quranGrouping';
  import { highlightSegments } from '$lib/highlight';
  import { formatNumber, locale, revelationLabel, surahMeaning, t } from '$lib/i18n';
  import { bookmarkState, loadBookmarks } from '$lib/stores/bookmarks.svelte';
  import {
    homeIndexState,
    setIndexTab,
    toggleAlphabetDirection,
    type IndexTab
  } from '$lib/stores/homeIndex.svelte';
  import { playFromAyah, playbackState } from '$lib/stores/playback.svelte';
  import {
    clearSearch,
    initSearchListeners,
    refreshIndexStatus,
    runSearch,
    scheduleSearch,
    searchState,
    searchTopic,
    topicQueries
  } from '$lib/stores/search.svelte';
  import { loadLastSurah, loadSurahList, selectSurah, selectSurahAyah, surahState } from '$lib/stores/surah.svelte';
  import { allSurahs, matchSurahs } from '$lib/stores/surahSearch.svelte';
  import { showReader } from '$lib/stores/view.svelte';
  import { openMarks } from '$lib/stores/panels.svelte';

  let searchInput = $state<HTMLInputElement | undefined>(undefined);
  let surahMatches = $derived(matchSurahs(searchState.query));
  let searched = $derived(searchState.hasSearched || searchState.hits.length > 0 || surahMatches.length > 0);

  // The shortcut is Cmd+K on macOS and Ctrl+K elsewhere; the keycap must say
  // the same thing the keybinding does.
  const isMac = /Mac|iPhone|iPad/.test(navigator.platform);
  const shortcutKey = isMac ? '⌘ K' : 'Ctrl K';

  // A language switch changes which index fields search can see, so the visible
  // results are re-run instead of staying in the previous language.
  let resultsLocale = $state(locale());
  $effect(() => {
    const current = locale();
    if (current === resultsLocale) return;
    resultsLocale = current;
    if (searchState.hasSearched && searchState.query.trim().length >= 2) {
      void runSearch(searchState.query);
    }
  });

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

  /** How the index below the search is grouped. */
  let indexTab = $derived(homeIndexState.tab);

  const indexTabs: { id: IndexTab; label: string }[] = [
    { id: 'surah', label: 'home.tabSurah' },
    { id: 'juz', label: 'home.tabJuz' },
    { id: 'revelation', label: 'home.tabRevelation' },
    { id: 'alphabet', label: 'home.tabAlphabet' }
  ];

  /**
   * Alphabetical order is the only list with a direction: readers look for a
   * name from either end. The order of revelation has one meaning (first
   * revealed to last), so it stays fixed and gets no toggle.
   */
  let alphabetDesc = $derived(homeIndexState.alphabetDesc);

  let sortLabel = $derived(alphabetDesc ? 'Z–A' : 'A–Z');

  function toggleSort() {
    if (indexTab === 'alphabet') toggleAlphabetDirection();
  }

  // Juz is a list of ranges, not surahs, so it reads its own table.
  let indexRows = $derived.by(() => {
    if (indexTab === 'revelation') {
      return [...surahIndex].sort((a, b) => revelationOf(a.nomor) - revelationOf(b.nomor));
    }
    if (indexTab === 'alphabet') {
      const direction = alphabetDesc ? -1 : 1;
      return [...surahIndex].sort(
        (a, b) => direction * a.namaLatin.localeCompare(b.namaLatin, 'id')
      );
    }
    return surahIndex;
  });

  /** A juz part needs the surah's own name and meaning, not just its number. */
  function surahByNumber(number: number) {
    return surahIndex.find((surah) => surah.nomor === number);
  }

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
        <span class="keycap">{shortcutKey}</span>
      </div>

      <div class="chip-row mt-4">
        {#each topicQueries() as topic}
          <button
            class="chip"
            class:is-active={searchState.query.trim() === topic}
            type="button"
            onclick={() => searchTopic(topic)}
          >
            {topic}
          </button>
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
                <span class="meta mt-1 block">{surahMeaning(surah.nomor, surah.arti)} · {surah.jumlahAyat} {t('play.ayahCount')}</span>
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
          <h2 class="section-label" aria-live="polite">{t('home.results', searchState.hits.length)}</h2>
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
                    {#each highlightSegments(hit.snippet, searchState.query) as segment}
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
                {surahMeaning(surahState.selected.nomor, surahState.selected.arti)} ·
                {surahState.selected.jumlahAyat}
                {t('play.ayahCount')} · {revelationLabel(surahState.selected.tempatTurun)}
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
                  {t('marks.ref', lastBookmark.surahName, lastBookmark.surah, lastBookmark.ayah)}
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

        {#if searchState.indexedAyah > 0 || totalBookmarks > 0}
          <p class="meta mt-4 flex flex-wrap items-center gap-x-2 gap-y-1">
            {#if searchState.indexedAyah > 0}
              <span class="tnum">{formatNumber(searchState.indexedAyah)}</span>
              <span>{t('home.statAyah')}</span>
            {/if}
            {#if searchState.indexedAyah > 0 && totalBookmarks > 0}
              <span aria-hidden="true">·</span>
            {/if}
            {#if totalBookmarks > 0}
              <span class="tnum">{formatNumber(totalBookmarks)}</span>
              <span>{t('home.statBookmarks')}</span>
            {/if}
          </p>
        {/if}

        <!-- The index of the book: 114 surahs, the thing a mushaf opens with -->
        <div class="mt-8 flex flex-wrap items-center justify-between gap-x-4 gap-y-2">
          <h2 class="section-label">{t('home.browse')}</h2>
          <div class="flex flex-wrap items-center gap-2">
            <div class="index-tabs" role="tablist" aria-label={t('home.browse')}>
              {#each indexTabs as tab (tab.id)}
                <button
                  class="index-tab"
                  class:is-active={indexTab === tab.id}
                  type="button"
                  role="tab"
                  aria-selected={indexTab === tab.id}
                  onclick={() => setIndexTab(tab.id)}
                >
                  {t(tab.label)}
                </button>
              {/each}
            </div>
            {#if indexTab === 'alphabet'}
              <!-- Sort direction belongs to the content, so it sits next to the
                   list rather than becoming a fifth tab. -->
              <button
                class="sort-toggle tnum"
                type="button"
                aria-label={t('home.sortDirection')}
                onclick={toggleSort}
              >
                {sortLabel}
                <Icon name={alphabetDesc ? 'chevron-down' : 'chevron-up'} size={13} />
              </button>
            {/if}
          </div>
        </div>

        {#if indexTab === 'juz'}
          <div class="mt-6 flex flex-col gap-8">
            {#each JUZ_RANGES as range (range.juz)}
              <section class="juz-section" aria-labelledby={`juz-${range.juz}`}>
                <div class="juz-head">
                  <h3 class="juz-title" id={`juz-${range.juz}`}>
                    {t('home.juzTitle', range.juz)}
                  </h3>
                  <span class="juz-meta tnum">
                    {range.startSurah}:{range.startAyah} – {range.endSurah}:{range.endAyah}
                    · {formatNumber(range.verses)}
                    {t('play.ayahCount')}
                  </span>
                </div>

                <div class="mt-3 grid grid-cols-2 gap-3 xl:grid-cols-3">
                  {#each range.parts as part (part.surah)}
                    {@const surah = surahByNumber(part.surah)}
                    <button
                      class="surah-tile {surahState.selected.nomor === part.surah ? 'is-current' : ''}"
                      type="button"
                      onclick={() => openVerse(part.surah, part.from)}
                    >
                      <span class="nav-row-index">{String(part.surah).padStart(3, '0')}</span>
                      <span class="min-w-0 flex-1">
                        <span class="surah-tile-name">{surah?.namaLatin ?? part.surah}</span>
                        <span class="meta mt-1 block">
                          {surahMeaning(part.surah, surah?.arti ?? '')} ·
                          {#if part.from > 1}
                            <span class="juz-continued">{t('home.continued')}</span> ·
                          {/if}
                          <span class="juz-range">
                            {t('home.ayahRange', part.from, part.to)}
                          </span>
                        </span>
                      </span>
                      {#if surah}
                        <span class="surah-tile-arabic arabic">{surah.nama}</span>
                      {/if}
                    </button>
                  {/each}
                </div>
              </section>
            {/each}
          </div>
        {:else}
          <div class="mt-4 grid grid-cols-2 gap-3 xl:grid-cols-3">
            {#each indexRows as surah (surah.nomor)}
              <button
                class="surah-tile {surahState.selected.nomor === surah.nomor ? 'is-current' : ''}"
                type="button"
                onclick={() => openSurah(surah.nomor)}
              >
                <span class="nav-row-index">{String(surah.nomor).padStart(3, '0')}</span>
                <span class="min-w-0 flex-1">
                  <span class="surah-tile-name">{surah.namaLatin}</span>
                  <span class="meta mt-1 block">
                    {surahMeaning(surah.nomor, surah.arti)} · {surah.jumlahAyat}
                    {t('play.ayahCount')}
                    {#if indexTab === 'revelation'}
                      · {t('home.revealed', revelationOf(surah.nomor))}
                    {/if}
                  </span>
                </span>
                <span class="surah-tile-arabic arabic">{surah.nama}</span>
              </button>
            {/each}
          </div>
        {/if}
      {/if}
    </div>
  </div>
</section>

<style>
.home-search {
  display: flex;
  width: 100%;
  align-items: center;
  gap: 12px;
  height: 52px;
  padding: 0 16px;
  border: 1px solid var(--border-strong);
  border-radius: 14px;
  background: var(--surface);
  transition: border-color var(--speed-fast) var(--ease);
}

.home-search:focus-within {
  border-color: var(--accent);
}

.home-search :global(svg) {
  width: 20px;
  height: 20px;
  flex: 0 0 auto;
  fill: none;
  stroke: var(--text-muted);
  stroke-width: 1.5;
  stroke-linecap: round;
  stroke-linejoin: round;
}

.home-search input {
  width: 100%;
  border: 0;
  background: transparent;
  color: var(--text);
  font-size: 16px;
  outline: none;
}

.home-search input::placeholder {
  color: var(--text-muted);
}

.chip-row {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
}

.chip {
  height: 32px;
  padding: 0 12px;
  border: 1px solid var(--border);
  border-radius: 8px;
  background: var(--surface);
  color: var(--text-secondary);
  font-size: 13px;
  transition: border-color var(--speed-fast) var(--ease), color var(--speed-fast) var(--ease),
    background-color var(--speed-fast) var(--ease);
}

.chip:hover {
  border-color: var(--accent);
  color: var(--accent);
}

.chip.is-active {
  border-color: var(--accent);
  background: var(--accent-soft);
  color: var(--accent);
}

.index-tabs {
  display: flex;
  flex-wrap: wrap;
  gap: 4px;
  width: fit-content;
  padding: 3px;
  border: 1px solid var(--border);
  border-radius: 10px;
  background: var(--raised);
}

.index-tab {
  height: 30px;
  padding: 0 12px;
  border: 0;
  border-radius: 8px;
  background: transparent;
  color: var(--text-secondary);
  font-size: 13px;
  font-weight: 500;
  transition:
    background-color var(--speed-fast) var(--ease),
    color var(--speed-fast) var(--ease);
}

.index-tab:hover {
  color: var(--text);
}

.index-tab.is-active {
  background: var(--surface);
  color: var(--text);
  box-shadow: var(--shadow-card);
}

/* Sort direction: a quiet control next to the list it orders, borderless so
   it does not read as a fifth tab */
.sort-toggle {
  display: inline-flex;
  height: 30px;
  align-items: center;
  gap: 4px;
  padding: 0 6px;
  border: 0;
  border-radius: 6px;
  background: transparent;
  color: var(--text-muted);
  font-size: 12px;
  font-weight: 500;
  transition: color var(--speed-fast) var(--ease);
}

.sort-toggle:hover {
  color: var(--text);
}

.sort-toggle :global(svg) {
  color: var(--text-muted);
}

/* Juz reads as 30 titled groups: one heading, then the cards of the surahs
   inside it, in the same shape as the surah index */
.juz-head {
  display: flex;
  flex-wrap: wrap;
  align-items: baseline;
  gap: 4px 12px;
}

.juz-title {
  font-size: 14px;
  font-weight: 500;
  color: var(--text);
}

.juz-meta {
  color: var(--text-muted);
  font-size: 12px;
}

.juz-range {
  font-family: 'IBM Plex Mono', ui-monospace, monospace;
}

/* The card starts mid-surah, so the juz only holds its tail */
.juz-continued {
  color: var(--text-secondary);
}

.result-row {
  display: grid;
  width: 100%;
  grid-template-columns: minmax(0, 1fr) auto;
  gap: 16px;
  align-items: start;
  padding: 16px;
  border: 1px solid var(--border);
  border-radius: var(--r-card);
  background: var(--surface);
  text-align: left;
  transition: border-color var(--speed-fast) var(--ease);
}

.result-row:hover {
  border-color: var(--accent);
}

.result-ref {
  font-size: 12px;
  font-weight: 500;
  color: var(--accent);
}

.result-snippet {
  margin-top: 6px;
  font-size: 14px;
  line-height: 1.55;
  color: var(--text-secondary);
}

.result-snippet mark {
  border-radius: 3px;
  background: var(--accent-soft);
  color: inherit;
  padding: 0 2px;
}

.result-arabic {
  margin-top: 6px;
  font-size: 22px;
  line-height: 1.9;
  color: var(--text);
}

.result-actions {
  display: flex;
  gap: 8px;
}

.continue-card {
  display: flex;
  width: 100%;
  align-items: center;
  justify-content: space-between;
  gap: 16px;
  padding: 20px;
  border: 1px solid var(--border);
  border-radius: var(--r-card);
  background: var(--surface);
  text-align: left;
  transition: border-color var(--speed-fast) var(--ease);
}

.continue-card:hover {
  border-color: var(--accent);
}

.surah-tile {
  display: flex;
  width: 100%;
  align-items: center;
  gap: 12px;
  padding: 12px;
  border: 1px solid var(--border);
  border-radius: var(--r-control);
  background: var(--surface);
  text-align: left;
  transition:
    border-color var(--speed-fast) var(--ease),
    background-color var(--speed-fast) var(--ease);
}

.surah-tile:hover {
  border-color: var(--border-strong);
  background: var(--raised);
}

.surah-tile-name {
  display: block;
  font-size: 14px;
  font-weight: 500;
  line-height: 1.3;
  color: var(--text);
}

.surah-tile-arabic {
  flex: 0 0 auto;
  max-width: 42%;
  font-size: 20px;
  line-height: 1.6;
  color: var(--text-secondary);
  text-align: right;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}
</style>
