<script lang="ts">
  import { tick } from 'svelte';
  import Icon from '$lib/components/Icon.svelte';
  import type { IconName } from '$lib/components/Icon.svelte';
  import CustomSelect from './CustomSelect.svelte';
  import CustomToggle from './CustomToggle.svelte';
  import { t } from '$lib/i18n';
  import {
    playbackState,
    refreshCacheStatus,
    refreshTtsVoiceStatus,
    setLang,
    setPrefetch,
    setQari,
    setReadTafsir,
    setTtsEnabled
  } from '$lib/stores/playback.svelte';
  import { surahState } from '$lib/stores/surah.svelte';
  import { closeSettings, panelState } from '$lib/stores/panels.svelte';

  let dialog = $state<HTMLDivElement | undefined>(undefined);
  let closeButton = $state<HTMLButtonElement | undefined>(undefined);
  let showAdvanced = $state(false);

  // Refresh when the modal opens, and again whenever the surah or reciter
  // changes while it stays open, so the figure is never stale.
  let wasOpen = $state(false);
  $effect(() => {
    if (!panelState.settingsOpen) {
      wasOpen = false;
      return;
    }
    const surah = surahState.selected.nomor;
    const totalAyat = surahState.selected.jumlahAyat;
    const qari = playbackState.qari;

    if (!wasOpen) {
      wasOpen = true;
      void refreshTtsVoiceStatus();
      void tick().then(() => closeButton?.focus());
    }
    void refreshCacheStatus(surah, totalAyat, qari);
  });

  let cachePercent = $derived(
    playbackState.cacheTotal > 0 ? (playbackState.cacheReady / playbackState.cacheTotal) * 100 : 0
  );
  let isTtsVoiceReady = $derived(playbackState.ttsVoiceStatus === 'ready');
  let ttsToggleDisabled = $derived(
    playbackState.ttsVoiceStatus === 'checking' || playbackState.ttsVoiceStatus === 'failed'
  );
  let voiceStatusState = $derived(
    playbackState.ttsVoiceStatus === 'ready'
      ? 'is-ready'
      : playbackState.ttsVoiceStatus === 'checking'
        ? ''
        : 'is-failed'
  );
  let voiceStatusIcon = $derived.by((): IconName => {
    if (playbackState.ttsVoiceStatus === 'ready') return 'check';
    if (playbackState.ttsVoiceStatus === 'checking') return 'loader';
    return 'alert';
  });
  let voiceStatusCopy = $derived(
    playbackState.ttsVoiceStatus === 'ready'
      ? t('settings.voiceReady')
      : playbackState.ttsVoiceStatus === 'checking'
        ? t('settings.voiceChecking')
        // The backend names the real cause (interpreter version, missing files);
        // the generic hint is only a fallback when it has nothing to say.
        : playbackState.ttsVoiceMessage.trim() || t('settings.voiceFailedHint')
  );

  /** Keeps Tab and Shift+Tab inside the dialog while it is open. */
  function trapFocus(event: KeyboardEvent) {
    if (event.key !== 'Tab' || !dialog) return;
    const focusable = dialog.querySelectorAll<HTMLElement>(
      'button:not([disabled]), input:not([disabled]), select, textarea, [href], [tabindex]:not([tabindex="-1"])'
    );
    if (focusable.length === 0) return;
    const first = focusable[0];
    const last = focusable[focusable.length - 1];
    if (event.shiftKey && document.activeElement === first) {
      event.preventDefault();
      last.focus();
    } else if (!event.shiftKey && document.activeElement === last) {
      event.preventDefault();
      first.focus();
    }
  }

  const qariOptions = [
    { value: 'misyari', label: 'Mishary Rashid Alafasy', description: 'Balanced recitation' },
    { value: 'sudais', label: 'Abdurrahman As-Sudais', description: 'Makkah imam' },
    { value: 'dossari', label: 'Ibrahim Al-Dossari', description: 'Clear cadence' },
    { value: 'juhany', label: 'Abdullah Al-Juhany', description: 'Haram reciter' },
    { value: 'qasim', label: 'Abdul Muhsin Al-Qasim', description: 'Madinah imam' },
    { value: 'yasser', label: 'Yasser Al-Dosari', description: 'Warm tone' }
  ];

  const languageOptions = [
    { value: 'id', label: 'Indonesian', description: 'Terjemahan Indonesia' },
    { value: 'en', label: 'English', description: 'English translation' }
  ];
</script>

<svelte:window
  onkeydown={(event) => {
    if (!panelState.settingsOpen) return;
    if (event.key === 'Escape') {
      event.preventDefault();
      closeSettings();
    }
  }}
/>

{#if panelState.settingsOpen}
  <!-- The backdrop is a button so a pointer click and the keyboard both close it -->
  <div class="settings-layer">
    <button
      class="settings-backdrop"
      type="button"
      aria-label={t('settings.close')}
      tabindex="-1"
      onclick={closeSettings}
    ></button>

    <div
      bind:this={dialog}
      class="settings-dialog"
      role="dialog"
      aria-modal="true"
      aria-labelledby="settings-heading"
      tabindex="-1"
      onkeydown={trapFocus}
    >
      <header class="mb-5 flex items-start justify-between gap-4">
        <div>
          <h2 id="settings-heading" class="text-[18px] font-medium text-[var(--text)]">
            {t('settings.title')}
          </h2>
          <p class="meta mt-1">{t('settings.subtitle')}</p>
        </div>
        <button
          bind:this={closeButton}
          class="icon-button"
          type="button"
          aria-label={t('settings.close')}
          onclick={closeSettings}
        >
          <Icon name="close" />
        </button>
      </header>

      <div class="flex flex-col gap-4">
        <CustomSelect
          label={t('settings.qari')}
          value={playbackState.qari}
          options={qariOptions}
          onChange={setQari}
        />

        <CustomSelect
          label={t('settings.language')}
          value={playbackState.lang}
          options={languageOptions}
          onChange={setLang}
        />

        <!-- The voice row only earns space when it says something actionable -->
        {#if playbackState.ttsVoiceStatus !== 'ready'}
          <div class="voice-status {voiceStatusState}">
            <span class="voice-status-icon">
              <Icon
                name={voiceStatusIcon}
                class={playbackState.ttsVoiceStatus === 'checking' ? 'animate-spin' : ''}
              />
            </span>
            <div>
              <h3 class="voice-status-title">{t('settings.voice')}</h3>
              <p class="voice-status-copy">{voiceStatusCopy}</p>
            </div>
          </div>
        {/if}

        {#if isTtsVoiceReady || ttsToggleDisabled}
          <CustomToggle
            label={t('settings.tts')}
            description={t('settings.ttsHint')}
            checked={playbackState.ttsEnabled}
            disabled={ttsToggleDisabled}
            onChange={setTtsEnabled}
          />

          <CustomToggle
            label={t('settings.prefetch')}
            description={t('settings.prefetchHint')}
            checked={playbackState.prefetch}
            disabled={ttsToggleDisabled}
            onChange={setPrefetch}
          />

          <CustomToggle
            label={t('settings.readTafsir')}
            description={t('settings.readTafsirHint')}
            checked={playbackState.readTafsir}
            disabled={ttsToggleDisabled}
            onChange={setReadTafsir}
          />
        {/if}

        <div class="advanced-settings">
          <button
            type="button"
            class="advanced-settings-trigger"
            aria-expanded={showAdvanced}
            onclick={() => (showAdvanced = !showAdvanced)}
          >
            <span>
              <span class="advanced-settings-title">{t('settings.advanced')}</span>
              <span class="advanced-settings-summary">{t('settings.advancedHint')}</span>
            </span>
            <Icon
              name="chevron-down"
              size={16}
              class="advanced-settings-chevron {showAdvanced ? 'open' : ''}"
            />
          </button>

          {#if showAdvanced}
            <div class="cache-status-panel">
              <h3 class="cache-status-title">{t('settings.cache')}</h3>
              {#if playbackState.cacheTotal > 0}
                <p class="cache-status-copy">
                  {playbackState.cacheReady} / {playbackState.cacheTotal} {t('settings.cacheReady')}
                </p>
                <div class="cache-status-track">
                  <div class="cache-status-fill" style={`width: ${cachePercent}%`}></div>
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
      </div>
    </div>
  </div>
{/if}
