<script lang="ts">
  import '../app.css';
  import { onMount, onDestroy } from 'svelte';
  import { handleGlobalKeydown } from '$lib/shortcuts';
  import {
    destroyPlaybackListeners,
    initPlaybackListeners,
    loadSettings,
    refreshTtsVoiceStatus
  } from '$lib/stores/playback.svelte';
  import { destroySearchListeners } from '$lib/stores/search.svelte';
  import { initTheme } from '$lib/stores/theme.svelte';
  import { initPanels } from '$lib/stores/panels.svelte';
  import { initView } from '$lib/stores/view.svelte';

  let { children } = $props();

  onMount(() => {
    initTheme();
    initPanels();
    initView();
    void loadSettings();
    void initPlaybackListeners();
    void refreshTtsVoiceStatus();
    window.addEventListener('keydown', handleGlobalKeydown);
  });

  onDestroy(() => {
    destroyPlaybackListeners();
    destroySearchListeners();
    window.removeEventListener('keydown', handleGlobalKeydown);
  });
</script>

{@render children()}
