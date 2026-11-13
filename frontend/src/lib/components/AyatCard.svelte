<script lang="ts">
  import Icon from '$lib/components/Icon.svelte';
  import VerseRosette from '$lib/components/VerseRosette.svelte';
  import { t } from '$lib/i18n';
  import { playbackState, pausePlayback, resumePlayback, playFromAyah } from '$lib/stores/playback.svelte';
  import { surahState } from '$lib/stores/surah.svelte';
  import { bookmarkState, toggleBookmark } from '$lib/stores/bookmarks.svelte';
  import {
    loadTafsir,
    playTafsirVoice,
    pauseTafsirVoice,
    resumeTafsirVoice,
    stopTafsirVoice,
    isTafsirVoiceActive,
    tafsirForAyah,
    tafsirState,
    tafsirVoiceState
  } from '$lib/stores/tafsir.svelte';
  import type { Ayat } from '$lib/stores/surah.svelte';

  let { ayah }: { ayah: Ayat } = $props();

  // The marker belongs to the playback session, not to the number alone: the
  // same ayah number exists in every surah, so without the surah check browsing
  // away would highlight an unrelated verse.
  let isSessionAyah = $derived(
    playbackState.sessionSurah === surahState.selected.nomor &&
      playbackState.currentAyah === ayah.nomorAyat
  );
  let isActive = $derived(
    isSessionAyah && (playbackState.playing || playbackState.phase === 'paused')
  );
  let isBookmarked = $derived(
    bookmarkState.list.some(
      (b) => b.surah === surahState.selected.nomor && b.ayah === ayah.nomorAyat
    )
  );
  let translation = $derived(
    playbackState.lang === 'en' && ayah.teksEnglish ? ayah.teksEnglish : ayah.teksIndonesia
  );

  let copied = $state(false);
  let showTafsir = $state(false);
  let copyTimer: ReturnType<typeof setTimeout> | undefined;

  /**
   * A short Arabic quote for a mark, cut on a word boundary so it never ends
   * mid-word or mid-diacritic. The API separates words with spaces and its
   * tanwin/long-vowel marks are non-spacing, so splitting on whitespace is safe.
   */
  function snippet(text: string, maxChars = 48): string {
    const clean = text.trim();
    if (clean.length <= maxChars) return clean;
    const words = clean.split(/\s+/);
    let out = '';
    for (const word of words) {
      const next = out === '' ? word : `${out} ${word}`;
      if (next.length > maxChars) break;
      out = next;
    }
    return out === '' ? clean.slice(0, maxChars) : `${out} …`;
  }

  let tafsir = $derived(tafsirForAyah(surahState.selected.nomor, ayah.nomorAyat));
  let tafsirLoading = $derived(
    tafsirState.loadingSurah === surahState.selected.nomor && showTafsir && !tafsir
  );
  let tafsirSpeaking = $derived(
    tafsirState.speakingKey === `${surahState.selected.nomor}:${ayah.nomorAyat}`
  );
  let tafsirVoiceActive = $derived(isTafsirVoiceActive(surahState.selected.nomor, ayah.nomorAyat));
  let tafsirVoicePaused = $derived(tafsirVoiceActive && tafsirVoiceState.paused);

  async function copyAyah() {
    const text = [
      `Surah ${surahState.selected.namaLatin} ayat ${ayah.nomorAyat}`,
      ayah.teksArab,
      ayah.teksLatin,
      translation
    ].join('\n');

    if (navigator.clipboard) {
      await navigator.clipboard.writeText(text);
    } else {
      const textarea = document.createElement('textarea');
      textarea.value = text;
      textarea.style.position = 'fixed';
      textarea.style.opacity = '0';
      document.body.appendChild(textarea);
      textarea.select();
      document.execCommand('copy');
      document.body.removeChild(textarea);
    }

    copied = true;
    if (copyTimer) clearTimeout(copyTimer);
    copyTimer = setTimeout(() => {
      copied = false;
    }, 1400);
  }

  async function toggleTafsirDetail() {
    showTafsir = !showTafsir;
    if (showTafsir && !tafsir) {
      await loadTafsir(surahState.selected.nomor);
    }
  }
</script>

<article
  class="verse-card {isActive ? 'is-active' : ''}"
>
  <div class="mb-5 flex items-start justify-between gap-6">
    <VerseRosette number={ayah.nomorAyat} size={40} active={isActive} />

    {#if isActive}
      <span class="verse-active-note mt-3">
        <span></span>
        {playbackState.playing ? t('ayah.playing') : t('ayah.paused')}
      </span>
    {/if}

    <p class="verse-arabic arabic flex-1">
      {ayah.teksArab}
    </p>
  </div>

  <p class="verse-latin mb-2">{ayah.teksLatin}</p>
  <p class="verse-translation">{translation}</p>

  <div class="mt-6 flex items-center justify-end gap-2">
    <button
      class="icon-button {isActive ? 'is-on' : ''}"
      type="button"
      aria-label={t('ayah.play', ayah.nomorAyat)}
      onclick={() => {
        if (isActive) {
          void (playbackState.playing ? pausePlayback() : resumePlayback());
        } else {
          playFromAyah(surahState.selected.nomor, ayah.nomorAyat, surahState.selected.jumlahAyat);
        }
      }}
    >
      <Icon name={isActive && playbackState.playing ? 'pause' : 'play'} filled />
    </button>

    <button
      class="icon-button {isBookmarked ? 'is-mark' : ''}"
      type="button"
      aria-label={t('ayah.bookmark', ayah.nomorAyat)}
      onclick={() =>
        toggleBookmark(
          surahState.selected.nomor,
          ayah.nomorAyat,
          surahState.selected.namaLatin,
          snippet(ayah.teksArab),
          translation
        )}
    >
      <Icon name={isBookmarked ? 'bookmark-filled' : 'bookmark'} filled={isBookmarked} />
    </button>

    <button class="icon-button" type="button" aria-label={t('ayah.copy', ayah.nomorAyat)} onclick={copyAyah}>
      <Icon name={copied ? 'check' : 'copy'} />
    </button>

    <button
      class="button {showTafsir ? 'is-on' : ''}"
      type="button"
      aria-expanded={showTafsir}
      onclick={toggleTafsirDetail}
    >
      {t('ayah.tafsir')}
      <Icon name="chevron-down" size={16} />
    </button>
  </div>

  {#if showTafsir}
    <section class="tafsir-panel">
      <div class="mb-4 flex items-center justify-between gap-3">
        <h3 class="tafsir-title">{t('ayah.tafsirTitle', ayah.nomorAyat)}</h3>

        {#if tafsir}
          <div class="flex items-center gap-2">
            {#if tafsirVoiceActive}
              <button
                class="tafsir-voice-button"
                type="button"
                aria-label={tafsirVoicePaused
                  ? t('ayah.tafsirResume', ayah.nomorAyat)
                  : t('ayah.tafsirPause', ayah.nomorAyat)}
                onclick={() => (tafsirVoicePaused ? resumeTafsirVoice() : pauseTafsirVoice())}
              >
                <Icon name={tafsirVoicePaused ? 'play' : 'pause'} size={14} filled={tafsirVoicePaused} />
                {tafsirVoicePaused ? t('ayah.resume') : t('ayah.pause')}
              </button>
            {/if}
            <button
              class="tafsir-voice-button {tafsirVoiceActive ? 'is-on' : ''}"
              type="button"
              aria-label={t('ayah.tafsirVoice', ayah.nomorAyat)}
              disabled={tafsirSpeaking}
              onclick={() =>
                tafsirVoiceActive
                  ? stopTafsirVoice()
                  : playTafsirVoice(surahState.selected.nomor, ayah.nomorAyat, tafsir.teks)}
            >
              <Icon name={tafsirVoiceActive ? 'stop' : 'volume'} size={14} filled={tafsirVoiceActive} />
              {tafsirSpeaking
                ? t('ayah.speaking')
                : tafsirVoiceActive
                  ? t('play.stop')
                  : t('ayah.voice')}
            </button>
          </div>
        {:else}
          <span class="tafsir-detail-badge">{t('common.loading')}</span>
        {/if}
      </div>

      {#if tafsirLoading}
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
  {/if}
</article>
