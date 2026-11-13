<script lang="ts">
  import Icon from '$lib/components/Icon.svelte';
  import { t } from '$lib/i18n';
  import { playbackState } from '$lib/stores/playback.svelte';
  import { surahState } from '$lib/stores/surah.svelte';
  import { formatTafsirText } from '$lib/tafsir';
  import {
    isTafsirVoiceActive,
    loadTafsir,
    pauseTafsirVoice,
    playTafsirVoice,
    resumeTafsirVoice,
    stopTafsirVoice,
    tafsirForAyah,
    tafsirState,
    tafsirVoiceState
  } from '$lib/stores/tafsir.svelte';

  type Props = {
    ayahNumber: number;
    /** Loaded lazily by the card when the panel opens. */
    loading: boolean;
  };

  let { ayahNumber, loading }: Props = $props();

  let tafsir = $derived(tafsirForAyah(surahState.selected.nomor, ayahNumber));
  // The English source arrives without paragraph breaks; keep the raw text for
  // speech and format only what is displayed.
  let tafsirBody = $derived(
    tafsir ? formatTafsirText(tafsir.teks) : ''
  );
  let speaking = $derived(tafsirState.speakingKey === `${surahState.selected.nomor}:${ayahNumber}`);
  let voiceActive = $derived(isTafsirVoiceActive(surahState.selected.nomor, ayahNumber));
  let voicePaused = $derived(voiceActive && tafsirVoiceState.paused);

  // Switching the translation language while this panel is open must replace
  // the commentary, not leave the previous language in place.
  $effect(() => {
    const surahNumber = surahState.selected.nomor;
    if (
      !tafsirForAyah(surahNumber, ayahNumber, playbackState.lang) &&
      tafsirState.loadingSurah !== surahNumber
    ) {
      void loadTafsir(surahNumber, playbackState.lang);
    }
  });
</script>

<section class="tafsir-panel">
  <div class="mb-4 flex items-center justify-between gap-3">
    <h3 class="tafsir-title">{t('ayah.tafsirTitle', ayahNumber)}</h3>

    {#if tafsir}
      <div class="flex items-center gap-2">
        {#if voiceActive}
          <button
            class="tafsir-voice-button"
            type="button"
            aria-label={voicePaused
              ? t('ayah.tafsirResume', ayahNumber)
              : t('ayah.tafsirPause', ayahNumber)}
            onclick={() => (voicePaused ? resumeTafsirVoice() : pauseTafsirVoice())}
          >
            <Icon name={voicePaused ? 'play' : 'pause'} size={14} filled={voicePaused} />
            {voicePaused ? t('ayah.resume') : t('ayah.pause')}
          </button>
        {/if}
        <button
          class="tafsir-voice-button {voiceActive ? 'is-on' : ''}"
          type="button"
          aria-label={t('ayah.tafsirVoice', ayahNumber)}
          disabled={speaking}
          onclick={() =>
            voiceActive
              ? stopTafsirVoice()
              : playTafsirVoice(surahState.selected.nomor, ayahNumber, tafsir.teks)}
        >
          <Icon name={voiceActive ? 'stop' : 'volume'} size={14} filled={voiceActive} />
          {speaking ? t('ayah.speaking') : voiceActive ? t('play.stop') : t('ayah.voice')}
        </button>
      </div>
    {:else}
      <span class="tafsir-detail-badge">{t('common.loading')}</span>
    {/if}
  </div>

  {#if loading}
    <p class="tafsir-status">{t('ayah.tafsirLoading')}</p>
  {:else if tafsir}
    <p class="tafsir-body whitespace-pre-line">{tafsirBody}</p>
  {:else}
    <p class="tafsir-status">{tafsirState.error || t('ayah.tafsirMissing')}</p>
  {/if}

  {#if tafsirState.error && tafsir}
    <p class="tafsir-status mt-2 text-[var(--mark)]">{tafsirState.error}</p>
  {/if}
</section>

<style>
.tafsir-panel {
  margin-top: 20px;
  border: 1px solid var(--border);
  border-radius: var(--r-card);
  background: var(--raised);
  padding: 20px;
}

.tafsir-title {
  font-size: 15px;
  font-weight: 500;
  color: var(--text);
}

.tafsir-body {
  font-size: 16px;
  line-height: 1.7;
  color: var(--text-secondary);
}

.tafsir-status {
  font-size: 13px;
  color: var(--text-muted);
}

.tafsir-voice-button {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  height: 28px;
  padding: 0 10px;
  border: 1px solid var(--border-strong);
  border-radius: 8px;
  background: var(--surface);
  color: var(--text-secondary);
  font-size: 12px;
  font-weight: 500;
  transition: background-color var(--speed-fast) var(--ease), color var(--speed-fast) var(--ease);
}

.tafsir-voice-button:hover {
  background: var(--raised);
  color: var(--text);
}

.tafsir-voice-button :global(svg) {
  width: 14px;
  height: 14px;
  flex: 0 0 auto;
}

.tafsir-detail-badge {
  font-size: 12px;
  color: var(--text-muted);
}
</style>
