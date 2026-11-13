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
      <Icon name="close" size={16} />
    </button>
  </header>

  <div class="min-h-0 flex-1 overflow-y-auto px-2 pb-4">
    {#if bookmarkState.list.length === 0}
      <div class="flex flex-col items-center gap-2 px-4 py-12 text-center">
        <Icon name="bookmark" size={22} />
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
                <VerseRosette number={mark.ayah} size={28} />
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
                <Icon name="close" size={14} />
              </button>
            </div>
          {/each}
        </section>
      {/each}
    {/if}
  </div>
</aside>
