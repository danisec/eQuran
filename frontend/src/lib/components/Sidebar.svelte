<script lang="ts">
  import { onMount } from 'svelte';
  import Icon from '$lib/components/Icon.svelte';
  import { t } from '$lib/i18n';
  import { bookmarkState, loadBookmarks } from '$lib/stores/bookmarks.svelte';
  import { themeState, toggleTheme } from '$lib/stores/theme.svelte';
  import { openSettings, panelState, toggleMarks, toggleSidebar } from '$lib/stores/panels.svelte';
  import { playbackState } from '$lib/stores/playback.svelte';
  import { showHome, showReader, viewState } from '$lib/stores/view.svelte';

  onMount(() => {
    void loadBookmarks();
  });

  // Collapsed keeps the rail on screen at icon width instead of hiding it, so
  // navigation and the utilities below stay reachable. Labels drop, tooltips
  // take over, and the footer stacks vertically (see `.is-collapsed` in CSS).
  let collapsed = $derived(!panelState.sidebarOpen);
</script>

<aside
  class="sidebar pane row-span-1 flex min-h-0 flex-col overflow-hidden hairline-r {collapsed
    ? 'is-collapsed'
    : ''} {playbackState.playing || playbackState.phase === 'paused' ? 'pb-[130px]' : 'pb-0'}"
>
  <!-- No brand row: the window title already names the app, and the rosette is
       the verse medallion where it belongs. The sidebar opens on navigation. -->
  <nav class="flex flex-col gap-0.5 px-4 pt-5" aria-label="eQuran">
    <button
      class="nav-row {viewState.view === 'home' ? 'is-current' : ''}"
      type="button"
      aria-current={viewState.view === 'home'}
      aria-label={t('nav.home')}
      title={collapsed ? t('nav.home') : undefined}
      onclick={showHome}
    >
      <Icon name="home" />
      <span class="nav-row-name">{t('nav.home')}</span>
      <span></span>
    </button>
    <button
      class="nav-row {viewState.view === 'reader' ? 'is-current' : ''}"
      type="button"
      aria-current={viewState.view === 'reader'}
      aria-label={t('nav.read')}
      title={collapsed ? t('nav.read') : undefined}
      onclick={showReader}
    >
      <Icon name="book" />
      <span class="nav-row-name">{t('nav.read')}</span>
      <span></span>
    </button>
    <button
      class="nav-row {panelState.marksOpen ? 'is-open' : ''}"
      type="button"
      aria-expanded={panelState.marksOpen}
      aria-label={t('nav.bookmarks')}
      title={collapsed ? t('nav.bookmarks') : undefined}
      onclick={toggleMarks}
    >
      <span class="nav-icon">
        <Icon name="bookmark" filled={panelState.marksOpen} />
        {#if collapsed && bookmarkState.list.length > 0}
          <span class="count-dot tnum">{bookmarkState.list.length}</span>
        {/if}
      </span>
      <span class="nav-row-name">{t('nav.bookmarks')}</span>
      <span class="nav-row-meta tnum">
        {bookmarkState.list.length > 0 ? bookmarkState.list.length : ''}
      </span>
    </button>
  </nav>

  <!-- The rail is navigation only. The reader's marks live in their own panel
       on the right (MarksRail), so this column stays about where you are. -->

  <div class="min-h-0 flex-1"></div>

  <!-- Footer is utilities plus the collapse control. It is the only place the
       rail can be toggled, so it is always reachable from both Beranda and
       Baca. "Where you are" is carried by the reader header and the Beranda
       continue card, not here. -->
  <div class="hairline-t sidebar-foot">
    <div class="sidebar-utils">
      <button
        class="icon-button"
        type="button"
        aria-label={t('settings.open')}
        title={t('settings.open')}
        onclick={openSettings}
      >
        <Icon name="settings" />
      </button>
      <button
        class="icon-button"
        type="button"
        aria-label={themeState.mode === 'dark' ? t('theme.toLight') : t('theme.toDark')}
        title={themeState.mode === 'dark' ? t('theme.toLight') : t('theme.toDark')}
        aria-pressed={themeState.mode === 'dark'}
        onclick={(event) => {
          // The reveal starts where the reader pressed, not at a fixed corner.
          const rect = event.currentTarget.getBoundingClientRect();
          toggleTheme({ x: rect.left + rect.width / 2, y: rect.top + rect.height / 2 });
        }}
      >
        <Icon name={themeState.mode === 'dark' ? 'sun' : 'moon'} />
      </button>
    </div>
    <button
      class="icon-button"
      type="button"
      aria-expanded={panelState.sidebarOpen}
      aria-label={panelState.sidebarOpen ? t('nav.collapseSidebar') : t('nav.expandSidebar')}
      title={panelState.sidebarOpen ? t('nav.collapseSidebar') : t('nav.expandSidebar')}
      onclick={toggleSidebar}
    >
      <Icon name={panelState.sidebarOpen ? 'chevron-left' : 'chevron-right'} />
    </button>
  </div>
</aside>

<style>
.nav-row {
  position: relative;
  isolation: isolate;
  display: grid;
  width: 100%;
  grid-template-columns: 24px minmax(0, 1fr) auto;
  align-items: center;
  justify-content: center;
  gap: 12px;
  height: 44px;
  padding: 0 10px;
  border: 0;
  border-radius: var(--r-control);
  background: transparent;
  color: var(--text);
  text-align: left;
  /* The track widths and gap collapse to a single icon column when the rail
     narrows, so they tween alongside the rail itself rather than snapping.
     `justify-content: center` only bites once the tracks are narrower than the
     row (the collapsed case); expanded, the 1fr track fills the row. */
  transition:
    background-color var(--speed-fast) var(--ease),
    color var(--speed-fast) var(--ease),
    grid-template-columns var(--speed) var(--ease),
    column-gap var(--speed) var(--ease);
}

.nav-row:hover {
  background: var(--surface);
}

.sidebar-foot {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  justify-content: space-between;
  gap: 8px;
  padding: 12px 16px;
  transition: padding var(--speed) var(--ease);
}

.sidebar-utils {
  display: flex;
  flex-wrap: wrap;
  min-width: 0;
  align-items: center;
  gap: 4px;
  transition: gap var(--speed) var(--ease);
}

.nav-row > .nav-icon {
  justify-self: center;
}

.nav-icon {
  position: relative;
  display: grid;
  place-items: center;
  width: 24px;
}
</style>
