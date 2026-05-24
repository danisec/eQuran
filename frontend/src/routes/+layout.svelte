<script lang="ts">
  import '../app.css';
  import { onMount, onDestroy } from 'svelte';
  import { handleGlobalKeydown } from '$lib/shortcuts';
  import { destroyPlaybackListeners, initPlaybackListeners } from '$lib/stores/playback.svelte';
  import { initTheme } from '$lib/stores/theme.svelte';

  let { children } = $props();

  onMount(() => {
    initTheme();
    void initPlaybackListeners();
    window.addEventListener('keydown', handleGlobalKeydown);
  });

  onDestroy(() => {
    destroyPlaybackListeners();
    window.removeEventListener('keydown', handleGlobalKeydown);
  });
</script>

{@render children()}
