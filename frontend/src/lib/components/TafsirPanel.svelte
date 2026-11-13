<script lang="ts">
  import Icon from '$lib/components/Icon.svelte';
  import { t } from '$lib/i18n';
  import { surahState } from '$lib/stores/surah.svelte';
  import {
    isTafsirVoiceActive,
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
  let speaking = $derived(tafsirState.speakingKey === `${surahState.selected.nomor}:${ayahNumber}`);
  let voiceActive = $derived(isTafsirVoiceActive(surahState.selected.nomor, ayahNumber));
  let voicePaused = $derived(voiceActive && tafsirVoiceState.paused);
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
    <p class="tafsir-body whitespace-pre-line">{tafsir.teks}</p>
  {:else}
    <p class="tafsir-status">{tafsirState.error || t('ayah.tafsirMissing')}</p>
  {/if}

  {#if tafsirState.error && tafsir}
    <p class="tafsir-status mt-2 text-[var(--mark)]">{tafsirState.error}</p>
  {/if}
</section>
