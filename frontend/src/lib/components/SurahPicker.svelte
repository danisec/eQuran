<script lang="ts">
  /**
   * A compact surah switcher for the reader header. The full index lives on the
   * home page, but switching surah mid-read should not require leaving the text.
   */
  import { onMount } from 'svelte';
  import Icon from '$lib/components/Icon.svelte';
  import { t } from '$lib/i18n';
  import { loadSurahList, selectSurah, surahState } from '$lib/stores/surah.svelte';

  let open = $state(false);
  let query = $state('');
  let root = $state<HTMLDivElement | undefined>(undefined);
  let input = $state<HTMLInputElement | undefined>(undefined);

  onMount(() => {
    void loadSurahList();
  });

  let matches = $derived.by(() => {
    const needle = `${query}`.trim().toLowerCase();
    const list = surahState.list;
    if (needle === '') return list;
    return list.filter((surah) =>
      `${surah.nomor} ${surah.namaLatin} ${surah.arti}`.toLowerCase().includes(needle)
    );
  });

  function toggle() {
    open = !open;
    if (open) {
      query = '';
      void Promise.resolve().then(() => input?.focus());
    }
  }

  function choose(number: number) {
    open = false;
    void selectSurah(number);
  }
</script>

<svelte:window
  onmousedown={(event) => {
    if (open && root && !root.contains(event.target as Node)) open = false;
  }}
  onkeydown={(event) => {
    if (event.key === 'Escape' && open) open = false;
  }}
/>

<div bind:this={root} class="relative">
  <button
    class="button"
    type="button"
    aria-haspopup="listbox"
    aria-expanded={open}
    aria-label={t('nav.chooseSurah')}
    onclick={toggle}
  >
    <Icon name="book" size={16} />
    {t('nav.chooseSurah')}
    <Icon name="chevron-down" size={14} />
  </button>

  {#if open}
    <div class="surah-picker" role="listbox" aria-label={t('nav.chooseSurah')}>
      <div class="p-2">
        <input
          bind:this={input}
          bind:value={query}
          class="h-9 w-full rounded-[8px] border border-[var(--border)] bg-[var(--surface)] px-3 text-[13px] text-[var(--text)] outline-none placeholder:text-[var(--text-muted)]"
          placeholder={t('nav.searchSurah')}
          aria-label={t('nav.searchSurah')}
        />
      </div>
      <div class="max-h-[46vh] overflow-y-auto p-1">
        {#each matches as surah (surah.nomor)}
          <button
            class="surah-picker-row {surahState.selected.nomor === surah.nomor ? 'is-current' : ''}"
            type="button"
            role="option"
            aria-selected={surahState.selected.nomor === surah.nomor}
            onclick={() => choose(surah.nomor)}
          >
            <span class="nav-row-index">{String(surah.nomor).padStart(3, '0')}</span>
            <span class="min-w-0 flex-1">
              <span class="nav-row-name truncate">{surah.namaLatin}</span>
              <span class="nav-row-meta">{surah.jumlahAyat} {t('play.ayahCount')}</span>
            </span>
            <span class="arabic shrink-0 text-[17px] text-[var(--text-secondary)]">{surah.nama}</span>
          </button>
        {/each}
        {#if matches.length === 0}
          <p class="meta px-3 py-4 text-center">{t('home.noResults')}</p>
        {/if}
      </div>
    </div>
  {/if}
</div>
