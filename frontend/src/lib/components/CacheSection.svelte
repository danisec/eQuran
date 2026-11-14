<script lang="ts">
  import Icon from '$lib/components/Icon.svelte';
  import { formatBytes, formatNumber, t } from '$lib/i18n';
  import {
    playbackState,
    clearCache,
    refreshCacheStatus,
    refreshCacheUsage
  } from '$lib/stores/playback.svelte';
  import { surahState } from '$lib/stores/surah.svelte';

  let open = $state(false);

  // The surah readiness bar: how many of this surah's ayahs already have qari
  // audio. It answers "is this surah ready?", not "how much disk is used".
  let surahPercent = $derived(
    playbackState.cacheTotal > 0 ? (playbackState.cacheReady / playbackState.cacheTotal) * 100 : 0
  );

  // The storage ledger: what the narration occupies against its cap.
  let limit = $derived(playbackState.cacheLimitBytes);
  let used = $derived(playbackState.cacheUsageBytes);
  let capped = $derived(limit !== null && limit > 0);
  let storagePercent = $derived(capped && limit ? Math.min(100, (used / limit) * 100) : 0);

  /** Clearing mid-session would pull a file out from under the audio thread. */
  let blocked = $derived(playbackState.playing || playbackState.phase === 'paused');

  let clearing = $state(false);
  let notice = $state('');

  let breakdown = $derived(
    [
      { label: t('settings.storageQari'), bytes: playbackState.cacheQariBytes },
      { label: t('settings.storageTranslation'), bytes: playbackState.cacheTranslationBytes },
      { label: t('settings.storageTafsir'), bytes: playbackState.cacheTafsirBytes }
    ].filter((row) => row.bytes > 0)
  );

  $effect(() => {
    // Load the ledger the first time the panel is opened, so a closed dialog
    // never pays for the directory walk.
    if (open) void refreshCacheUsage();
  });

  async function onClear() {
    if (blocked || clearing) return;
    clearing = true;
    notice = '';
    const error = await clearCache();
    clearing = false;
    if (error) {
      notice = error;
      return;
    }
    notice = t('settings.cacheCleared');
    // The surah readiness bar just changed too.
    void refreshCacheStatus(
      surahState.selected.nomor,
      surahState.selected.jumlahAyat,
      playbackState.qari
    );
  }
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
          <div class="cache-status-fill" style={`width: ${surahPercent}%`}></div>
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

      <!-- The storage ledger: the one rich moment, a meter with a mono figure. -->
      <div class="storage">
        <div class="storage-head">
          <span class="storage-figure tnum">
            {#if capped}
              {t('settings.storageOf', formatBytes(used), formatBytes(limit ?? 0))}
            {:else}
              {t('settings.storageUsedOnly', formatBytes(used))}
            {/if}
          </span>
          <span class="storage-files tnum">
            {t('settings.storageFiles', formatNumber(playbackState.cacheFileCount))}
          </span>
        </div>

        <div
          class="storage-track"
          role="progressbar"
          aria-label={t('settings.cache')}
          aria-valuemin={0}
          aria-valuemax={capped ? (limit ?? 0) : used}
          aria-valuenow={used}
          aria-valuetext={t('settings.storageOf', formatBytes(used), formatBytes(limit ?? used))}
        >
          <div class="storage-fill" style={`width: ${capped ? storagePercent : used > 0 ? 100 : 0}%`}></div>
        </div>

        <dl class="storage-legend">
          {#each breakdown as row (row.label)}
            <div class="storage-legend-row">
              <dt>{row.label}</dt>
              <dd class="tnum">{formatBytes(row.bytes)}</dd>
            </div>
          {/each}
        </dl>

        <p class="meta storage-hint">
          {capped
            ? t('settings.storageHint', formatBytes(limit ?? 0))
            : t('settings.storageHintUncapped')}
        </p>

        <div class="storage-actions">
          <button
            class="button"
            type="button"
            disabled={blocked || clearing}
            onclick={onClear}
          >
            {clearing ? t('settings.clearing') : t('settings.clearCache')}
          </button>
          {#if notice}
            <span class="storage-notice">{notice}</span>
          {:else if blocked}
            <span class="storage-notice">{t('settings.cacheClearBlocked')}</span>
          {/if}
        </div>
      </div>
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

  /* Storage ledger */
  .storage {
    margin-top: 16px;
    border-top: 1px solid var(--border);
    padding-top: 16px;
  }

  .storage-head {
    display: flex;
    align-items: baseline;
    justify-content: space-between;
    gap: 12px;
  }

  .storage-figure {
    font-size: 15px;
    font-weight: 500;
    color: var(--text);
  }

  .storage-files {
    color: var(--text-muted);
    font-family: 'IBM Plex Mono', ui-monospace, monospace;
    font-size: 11px;
  }

  .storage-track {
    height: 6px;
    margin-top: 10px;
    overflow: hidden;
    border-radius: 3px;
    background: var(--track);
  }

  .storage-fill {
    height: 100%;
    border-radius: 3px;
    background: var(--accent);
    transition: width var(--speed) var(--ease);
  }

  .storage-legend {
    display: flex;
    flex-direction: column;
    gap: 6px;
    margin-top: 12px;
  }

  .storage-legend-row {
    display: flex;
    align-items: baseline;
    justify-content: space-between;
    gap: 12px;
  }

  .storage-legend-row dt {
    color: var(--text-muted);
    font-size: 12px;
  }

  .storage-legend-row dd {
    color: var(--text-secondary);
    font-size: 12px;
  }

  .storage-hint {
    margin-top: 12px;
    line-height: 1.45;
  }

  .storage-actions {
    display: flex;
    align-items: center;
    gap: 10px;
    margin-top: 12px;
  }

  .storage-actions .button:disabled {
    cursor: not-allowed;
    opacity: 0.55;
  }

  .storage-notice {
    color: var(--text-muted);
    font-size: 12px;
  }
</style>
