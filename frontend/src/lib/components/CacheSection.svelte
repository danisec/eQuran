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

<style>
  /* Advanced disclosure */
  .advanced-settings {
    border-top: 1px solid var(--border);
    padding-top: 16px;
  }
  
  .advanced-settings-trigger {
    display: flex;
    width: 100%;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
    padding: 10px 12px;
    border: 1px solid var(--border);
    border-radius: var(--r-control);
    background: var(--surface);
    color: var(--text);
    text-align: left;
    transition: border-color var(--speed-fast) var(--ease), background-color var(--speed-fast) var(--ease);
  }
  
  .advanced-settings-trigger:hover,
  .advanced-settings-trigger[aria-expanded='true'] {
    border-color: var(--border-strong);
    background: var(--raised);
  }
  
  .advanced-settings-title {
    display: block;
    font-size: 14px;
    font-weight: 500;
  }
  
  .advanced-settings-summary {
    display: block;
    margin-top: 2px;
    color: var(--text-muted);
    font-size: 12px;
  }
  
  :global(.advanced-settings-chevron) {
    width: 16px;
    height: 16px;
    flex: 0 0 auto;
    color: var(--text-muted);
    transition: transform var(--speed-fast) var(--ease);
  }
  
  :global(.advanced-settings-chevron.open) {
    transform: rotate(180deg);
  }
  
  .cache-status-panel {
    margin-top: 12px;
    border: 1px solid var(--border);
    border-radius: var(--r-control);
    background: var(--surface);
    padding: 16px;
  }
  
  .cache-status-title {
    margin-bottom: 10px;
    font-size: 13px;
    font-weight: 500;
    color: var(--text);
  }
  
  .cache-status-copy {
    margin-bottom: 12px;
    color: var(--text-muted);
    font-size: 12px;
    line-height: 1.45;
  }
  
  .cache-status-track {
    height: 6px;
    overflow: hidden;
    border-radius: 3px;
    background: var(--track);
  }
  
  .cache-status-fill {
    height: 100%;
    border-radius: 3px;
    background: var(--accent);
    transition: width var(--speed) var(--ease);
  }
  
  .cache-status-live {
    display: flex;
    align-items: center;
    gap: 8px;
    margin-top: 12px;
    color: var(--text-muted);
    font-size: 12px;
  }
  
  .cache-status-live span {
    width: 6px;
    height: 6px;
    border-radius: 50%;
    background: currentColor;
  }
</style>
