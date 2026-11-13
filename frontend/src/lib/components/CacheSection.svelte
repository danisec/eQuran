<script lang="ts">
  import Icon from '$lib/components/Icon.svelte';
  import { t } from '$lib/i18n';
  import { playbackState } from '$lib/stores/playback.svelte';

  let open = $state(false);

  let percent = $derived(
    playbackState.cacheTotal > 0 ? (playbackState.cacheReady / playbackState.cacheTotal) * 100 : 0
  );
</script>

<div class="advanced-settings">
  <button
    type="button"
    class="advanced-settings-trigger"
    aria-expanded={open}
    onclick={() => (open = !open)}
  >
    <span>
      <span class="advanced-settings-title">{t('settings.advanced')}</span>
      <span class="advanced-settings-summary">{t('settings.advancedHint')}</span>
    </span>
    <Icon name="chevron-down" size={16} class="advanced-settings-chevron {open ? 'open' : ''}" />
  </button>

  {#if open}
    <div class="cache-status-panel">
      <h3 class="cache-status-title">{t('settings.cache')}</h3>
      {#if playbackState.cacheTotal > 0}
        <p class="cache-status-copy">
          {playbackState.cacheReady} / {playbackState.cacheTotal} {t('settings.cacheReady')}
        </p>
        <div class="cache-status-track">
          <div class="cache-status-fill" style={`width: ${percent}%`}></div>
        </div>
        {#if playbackState.cacheReady === 0}
          <p class="meta mt-2">{t('settings.cacheNone')}</p>
        {/if}
      {:else}
        <p class="cache-status-copy">{t('common.loading')}</p>
      {/if}

      {#if playbackState.playing && playbackState.phase === 'cache'}
        <p class="cache-status-live">
          <span></span>
          {t('settings.offline')}
        </p>
      {/if}
    </div>
  {/if}
</div>
