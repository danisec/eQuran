<script lang="ts">
  import '../app.css';
  import { onMount, onDestroy } from 'svelte';
  import { syncDocumentLang } from '$lib/i18n';
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
  import { initHomeIndex } from '$lib/stores/homeIndex.svelte';
  import { initView } from '$lib/stores/view.svelte';

  let { children } = $props();

  // The document language follows the translation setting, so screen readers,
  // spellcheck and hyphenation match what is on screen.
  $effect(() => {
    syncDocumentLang();
  });

  onMount(() => {
    initTheme();
    initPanels();
    initHomeIndex();
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
