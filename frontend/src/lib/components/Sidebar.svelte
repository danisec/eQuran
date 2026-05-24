<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { loadSurahList, loadLastSurah, selectSurah, selectSurahAyah, surahState } from '$lib/stores/surah.svelte';
  import { bookmarkState, loadBookmarks, removeBookmark } from '$lib/stores/bookmarks.svelte';
  import { themeState, toggleTheme } from '$lib/stores/theme.svelte';

  let searchInput: HTMLInputElement;
  let activeTab: 'quran' | 'bookmark' = $state('quran');

  onMount(() => {
    void loadSurahList();
    void loadLastSurah();
    void loadBookmarks();
    window.addEventListener('keydown', handleKeydown);
  });

  onDestroy(() => {
    window.removeEventListener('keydown', handleKeydown);
  });

  function handleKeydown(event: KeyboardEvent) {
    if ((event.metaKey || event.ctrlKey) && event.key === 'k') {
      event.preventDefault();
      searchInput?.focus();
    }
    if (event.key === 'Escape') {
      searchInput?.blur();
      surahState.query = '';
    }
  }

  let filteredSurah = $derived(
    surahState.list.filter((surah) =>
      `${surah.nomor} ${surah.namaLatin}`.toLowerCase().includes(surahState.query.toLowerCase())
    )
  );
</script>

<aside class="row-span-1 flex min-h-0 flex-col border-r border-[#d7c294] bg-[#f8efd9]/80">
  <div class="p-6 pb-0">
    <div class="mb-8 text-center">
      <div class="relative mx-auto mb-2 grid h-20 w-20 place-items-center rounded-full bg-[#0b5a43] text-3xl text-[#f0d07a]">
        ☪
      </div>
      <h1 class="text-4xl font-semibold text-[#0b4a39]">eQuran</h1>
      <button
        class="mx-auto mt-3 inline-flex items-center gap-1 rounded-full border border-[#d1b87d] bg-white/60 px-2 py-1 text-[0.62rem] font-bold uppercase tracking-[0.1em] text-[#0b5a43] shadow-sm hover:bg-[#fff7e5]"
        type="button"
        aria-label={themeState.mode === 'dark' ? 'Aktifkan Light Mode' : 'Aktifkan Dark Mode'}
        aria-pressed={themeState.mode === 'dark'}
        title={themeState.mode === 'dark' ? 'Light Mode' : 'Dark Mode'}
        onclick={toggleTheme}
      >
        <span class="flex h-5 w-5 items-center justify-center rounded-full bg-[#0b5a43] text-[#f0d07a]">
          {#if themeState.mode === 'dark'}
            <svg viewBox="0 0 24 24" class="h-3 w-3 fill-current" aria-hidden="true">
              <circle cx="12" cy="12" r="4" />
              <path d="M12 1.75a.75.75 0 0 1 .75.75v2a.75.75 0 0 1-1.5 0v-2a.75.75 0 0 1 .75-.75M12 18.75a.75.75 0 0 1 .75.75v2a.75.75 0 0 1-1.5 0v-2a.75.75 0 0 1 .75-.75M21.5 11.25a.75.75 0 0 1 0 1.5h-2a.75.75 0 0 1 0-1.5zM5.25 12a.75.75 0 0 1-.75.75h-2a.75.75 0 0 1 0-1.5h2a.75.75 0 0 1 .75.75M18.72 4.22a.75.75 0 0 1 1.06 1.06l-1.42 1.42a.75.75 0 1 1-1.06-1.06zM6.7 17.3a.75.75 0 0 1 0 1.06l-1.42 1.42a.75.75 0 1 1-1.06-1.06l1.42-1.42a.75.75 0 0 1 1.06 0M19.78 18.72a.75.75 0 1 1-1.06 1.06l-1.42-1.42a.75.75 0 0 1 1.06-1.06zM5.28 4.22 6.7 5.64A.75.75 0 1 1 5.64 6.7L4.22 5.28a.75.75 0 0 1 1.06-1.06" />
            </svg>
          {:else}
            <svg viewBox="0 0 24 24" class="h-3 w-3 fill-current" aria-hidden="true">
              <path d="M20.2 15.65A8.35 8.35 0 0 1 8.35 3.8a.75.75 0 0 1 .88.92 6.85 6.85 0 0 0 10.05 7.95.75.75 0 0 1 .92.88z" />
            </svg>
          {/if}
        </span>
        {themeState.mode === 'dark' ? 'Dark' : 'Light'}
      </button>
      <div class="mx-auto mt-3 h-px w-28 bg-[#c19a4a]"></div>
    </div>

    <label class="mb-5 flex items-center gap-3 rounded-xl border border-[#d1b87d] bg-white/50 px-4 py-3 text-sm text-[#514a3b]">
      <span>⌕</span>
      <input bind:this={searchInput} class="w-full bg-transparent outline-none" bind:value={surahState.query} placeholder="Cari surah atau ayah..." />
      <kbd class="rounded border border-[#d1b87d] px-1 text-xs">⌘K</kbd>
    </label>

    <div class="mb-3 flex items-center gap-3 text-xs font-semibold uppercase tracking-widest text-[#b0832f]">
      <span class="h-px flex-1 bg-[#d8bf84]"></span>
      {activeTab === 'quran' ? 'Daftar Surah' : 'Bookmarks'}
      <span class="h-px flex-1 bg-[#d8bf84]"></span>
    </div>
  </div>

  <div class="min-h-0 flex-1 overflow-y-auto px-6 pb-2">
    {#if activeTab === 'quran'}
      {#each filteredSurah as surah}
        <button
          class="mb-2 grid w-full grid-cols-[40px_1fr_auto] items-center gap-3 rounded-xl px-3 py-3 text-left transition {surahState.selected.nomor === surah.nomor ? 'bg-[#0d5b43] text-white shadow-lg' : 'hover:bg-white/60'}"
          onclick={() => selectSurah(surah.nomor)}
        >
          <span class="grid h-8 w-8 place-items-center rounded-full border border-[#d1b87d] bg-[#fff7e5] text-[#58492a]">{surah.nomor}</span>
          <span>
            <span class="block font-medium">{surah.namaLatin}</span>
            <span class="text-xs opacity-80">{surah.jumlahAyat} ayah</span>
          </span>
          <span class="arabic text-xl">{surah.nama}</span>
        </button>
      {/each}
    {:else}
      {#if bookmarkState.list.length === 0}
        <div class="flex h-full flex-col items-center justify-center gap-3 text-sm text-[#8a7a5a]">
          <span class="text-4xl">🔖</span>
          <p>Belum ada bookmark</p>
          <p class="text-xs">Klik ikon bookmark pada ayat untuk menambahkan</p>
        </div>
      {:else}
        {#each bookmarkState.list as bookmark}
          <div class="mb-2 flex items-center gap-3 rounded-xl px-3 py-3 transition hover:bg-white/60">
            <button
              class="flex flex-1 items-center gap-3 text-left"
              onclick={() => selectSurahAyah(bookmark.surah, bookmark.ayah)}
            >
              <span class="grid h-8 w-8 shrink-0 place-items-center rounded-full border border-[#e74c3c] bg-[#fdf0ef] text-sm text-[#e74c3c]">
                <svg viewBox="0 0 24 24" class="h-4 w-4 fill-current" aria-hidden="true">
                  <path d="M7 4.75A2.75 2.75 0 0 1 9.75 2h4.5A2.75 2.75 0 0 1 17 4.75v16l-5-3.15L7 20.75z" />
                </svg>
              </span>
              <span>
                <span class="block text-sm font-medium">{bookmark.surahName} : {bookmark.ayah}</span>
                <span class="arabic block text-xs text-[#5a5345]">{bookmark.ayahText}</span>
              </span>
            </button>
            <button
              class="grid h-7 w-7 shrink-0 place-items-center rounded-full text-xs text-[#8a7a5a] transition hover:bg-[#fdf0ef] hover:text-[#e74c3c]"
              aria-label="Remove bookmark"
              onclick={() => removeBookmark(bookmark.surah, bookmark.ayah)}
            >✕</button>
          </div>
        {/each}
      {/if}
    {/if}
  </div>

  <div class="flex border-t border-[#d7c294]">
    <button
      class="flex flex-1 items-center justify-center gap-2 py-4 text-sm font-medium transition {activeTab === 'quran' ? 'text-[#0b5a43]' : 'text-[#8a7a5a] hover:text-[#0b5a43]'}"
      onclick={() => activeTab = 'quran'}
    >
      📖 Quran
    </button>
    <button
      class="flex flex-1 items-center justify-center gap-2 py-4 text-sm font-medium transition {activeTab === 'bookmark' ? 'text-[#0b5a43]' : 'text-[#8a7a5a] hover:text-[#0b5a43]'}"
      onclick={() => activeTab = 'bookmark'}
    >
      <svg viewBox="0 0 24 24" class="h-4 w-4 {activeTab === 'bookmark' ? 'fill-current' : 'fill-none'}" aria-hidden="true">
        <path d="M7 4.75A2.75 2.75 0 0 1 9.75 2h4.5A2.75 2.75 0 0 1 17 4.75v16l-5-3.15L7 20.75z" stroke="currentColor" stroke-width="1.8" stroke-linejoin="round" />
      </svg>
      Bookmark {bookmarkState.list.length > 0 ? `(${bookmarkState.list.length})` : ''}
    </button>
  </div>
</aside>
