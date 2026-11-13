<script lang="ts">
  /**
   * The marks rail: the reader's own verses, on the right.
   *
   * Navigation lives on the left, so the reader's marks — a growing collection
   * rather than a fixed set of places — get their own panel. Grouping by surah
   * and the Arabic + translation of each verse let a mark be read on its own,
   * which a one-line navigation row could not do.
   */
  import { onMount } from 'svelte';
  import Icon from '$lib/components/Icon.svelte';
  import VerseRosette from '$lib/components/VerseRosette.svelte';
  import { t } from '$lib/i18n';
  import {
    bookmarkState,
    groupBookmarks,
    loadBookmarks,
    removeBookmark
  } from '$lib/stores/bookmarks.svelte';
  import { toggleMarks, panelState } from '$lib/stores/panels.svelte';
  import { playbackState } from '$lib/stores/playback.svelte';
  import { selectSurahAyah } from '$lib/stores/surah.svelte';
  import { showReader } from '$lib/stores/view.svelte';

  onMount(() => {
    void loadBookmarks();
  });

  let groups = $derived(groupBookmarks(bookmarkState.list));

  async function openMark(surah: number, ayah: number) {
    await selectSurahAyah(surah, ayah);
    showReader();
  }
</script>

<aside
  inert={!panelState.marksOpen}
  aria-label={t('nav.bookmarks')}
  class="marks-rail hairline-l flex min-h-0 flex-col overflow-hidden bg-[var(--raised)] {panelState.marksOpen
    ? 'is-open'
    : 'pointer-events-none'} {playbackState.playing || playbackState.phase === 'paused'
    ? 'pb-[130px]'
    : ''}"
>
  <header class="flex shrink-0 items-center justify-between px-4 pb-2 pt-4">
    <h2 class="section-label">
      {t('nav.bookmarks')}
      {#if bookmarkState.list.length > 0}
        <span class="tnum ml-1 text-[var(--text)]">{bookmarkState.list.length}</span>
      {/if}
    </h2>
    <button
      class="icon-button"
      type="button"
      aria-label={t('marks.close')}
      title={t('marks.close')}
      onclick={toggleMarks}
    >
      <Icon name="close" />
    </button>
  </header>

  <div class="min-h-0 flex-1 overflow-y-auto px-2 pb-4">
    {#if bookmarkState.list.length === 0}
      <div class="flex flex-col items-center gap-2 px-4 py-12 text-center">
        <Icon name="bookmark" size={24} />
        <p class="text-[14px] text-[var(--text)]">{t('nav.noBookmarks')}</p>
        <p class="meta max-w-[28ch] leading-relaxed">{t('nav.noBookmarksHint')}</p>
      </div>
    {:else}
      {#each groups as group (group.surah)}
        <section>
          <h3 class="marks-group-label px-2">
            <span class="tnum">{String(group.surah).padStart(3, '0')}</span>
            <span>{group.surahName}</span>
          </h3>
          {#each group.items as mark (mark.surah + ':' + mark.ayah)}
            <div class="mark-row group">
              <button
                class="mark-open"
                type="button"
                onclick={() => openMark(mark.surah, mark.ayah)}
              >
                <VerseRosette number={mark.ayah} size={32} />
                <span class="min-w-0 flex-1">
                  <span class="mark-ref">
                    {t('marks.ref', mark.surahName, mark.surah, mark.ayah)}
                  </span>
                  <span class="arabic mark-arabic">{mark.ayahText}</span>
                  {#if mark.translation}
                    <span class="mark-translation">{mark.translation}</span>
                  {/if}
                </span>
              </button>
              <button
                class="mark-remove icon-button"
                type="button"
                aria-label="{t('nav.removeBookmark')} {mark.surah}:{mark.ayah}"
                title={t('nav.removeBookmark')}
                onclick={() => removeBookmark(mark.surah, mark.ayah)}
              >
                <Icon name="close" />
              </button>
            </div>
          {/each}
        </section>
      {/each}
    {/if}
  </div>
</aside>

<style>
.marks-group-label {
  display: flex;
  align-items: baseline;
  gap: 8px;
  padding-top: 12px;
  padding-bottom: 4px;
  font-size: 12px;
  font-weight: 500;
  color: var(--text-muted);
}

.mark-row {
  position: relative;
  display: flex;
  align-items: flex-start;
  border-radius: var(--r-control);
  transition: background-color var(--speed-fast) var(--ease);
}

.mark-row:hover {
  background: var(--surface);
}

.mark-open {
  display: flex;
  min-width: 0;
  flex: 1;
  align-items: flex-start;
  gap: 12px;
  padding: 12px 8px;
  border: 0;
  border-radius: var(--r-control);
  background: transparent;
  text-align: left;
}

/* The reference line is exactly as tall as the medallion beside it, so the
   ayah number and its citation share one centre line. */
.mark-ref {
  display: flex;
  align-items: center;
  min-height: 32px;
  padding-right: 44px;
  font-size: 12px;
  font-weight: 500;
  color: var(--accent);
}

.mark-arabic {
  display: block;
  margin-top: 4px;
  font-size: 19px;
  line-height: 1.7;
  color: var(--text);
}

.mark-translation {
  display: -webkit-box;
  margin-top: 4px;
  overflow: hidden;
  -webkit-box-orient: vertical;
  -webkit-line-clamp: 2;
  line-clamp: 2;
  font-size: 13px;
  line-height: 1.5;
  color: var(--text-secondary);
}

/* One control size with the header: 34px box, 18px glyph. `top: 11px` puts its
   centre on the medallion's centre (padding 12 + half of 32) exactly. */
.mark-remove {
  position: absolute;
  top: 11px;
  right: 8px;
  width: 34px;
  height: 34px;
  opacity: 0;
  pointer-events: none;
  transition:
    opacity var(--speed-fast) var(--ease),
    background-color var(--speed-fast) var(--ease),
    color var(--speed-fast) var(--ease);
}

.mark-row:hover .mark-remove,
.mark-row:focus-within .mark-remove,
.mark-remove:focus-visible {
  opacity: 1;
  pointer-events: auto;
}
</style>
