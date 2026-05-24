<script lang="ts">
  import AudioPlayer from '$lib/components/AudioPlayer.svelte';
  import FocusModeView from '$lib/components/FocusModeView.svelte';
  import MainContent from '$lib/components/MainContent.svelte';
  import PlaybackSettings from '$lib/components/PlaybackSettings.svelte';
  import Sidebar from '$lib/components/Sidebar.svelte';
  import { focusState } from '$lib/stores/focus.svelte';
</script>

<main class="grid h-screen overflow-hidden border border-[#d6bd7f] bg-[#fbf4e4]/95 shadow-2xl {focusState.enabled ? 'focus-shell grid-cols-1 grid-rows-1' : 'grid-cols-[300px_1fr_340px] grid-rows-[1fr_120px]'}">
  {#if focusState.enabled}
    <FocusModeView />
  {:else}
    <Sidebar />
    <MainContent />
    <PlaybackSettings />
    <AudioPlayer />
  {/if}
</main>
