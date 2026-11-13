<script lang="ts">
  import AudioPlayer from '$lib/components/AudioPlayer.svelte';
  import FocusModeView from '$lib/components/FocusModeView.svelte';
  import HomeView from '$lib/components/HomeView.svelte';
  import MainContent from '$lib/components/MainContent.svelte';
  import MarksRail from '$lib/components/MarksRail.svelte';
  import PlaybackSettings from '$lib/components/PlaybackSettings.svelte';
  import Sidebar from '$lib/components/Sidebar.svelte';
  import { focusState } from '$lib/stores/focus.svelte';
  import { panelState } from '$lib/stores/panels.svelte';
  import { viewState } from '$lib/stores/view.svelte';

  // Navigation rail, content, then the reader's marks on the right. Settings
  // are a dialog, so they are not a column. Focus mode is a single pane. The
  // track sizes are variables so a narrow window can collapse the marks track
  // and float the rail over the reader instead (see the 1180px media query).
  // Collapsed, the left rail shrinks to an icon column rather than to nothing.
  const shellStyle = $derived(
    focusState.enabled
      ? 'grid-template-columns: 1fr; grid-template-rows: 1fr;'
      : `--col-left: ${panelState.sidebarOpen ? '280px' : '64px'}; --col-right: ${
          panelState.marksOpen ? '340px' : '0px'
        }; grid-template-columns: var(--col-left) minmax(0, 1fr) var(--col-right); grid-template-rows: 1fr;`
  );
</script>

<main
  class="app-shell relative grid h-screen overflow-hidden border {focusState.enabled ? 'focus-shell' : ''}"
  style={shellStyle}
>
  {#if focusState.enabled}
    <FocusModeView />
  {:else}
    <Sidebar />
    {#if viewState.view === 'home'}
      <HomeView />
    {:else}
      <MainContent />
    {/if}
    <MarksRail />
    <AudioPlayer />
    <PlaybackSettings />
  {/if}
</main>
