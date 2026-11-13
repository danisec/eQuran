<script lang="ts">
  import Icon from '$lib/components/Icon.svelte';
  import { t } from '$lib/i18n';
  import { bookmarkState, toggleBookmark } from '$lib/stores/bookmarks.svelte';
  import { pausePlayback, playbackState, playFromAyah, resumePlayback } from '$lib/stores/playback.svelte';
  import { surahState } from '$lib/stores/surah.svelte';
  import type { Ayat } from '$lib/stores/surah.svelte';

  type Props = {
    ayah: Ayat;
    /** Translation as the reader sees it (English or Indonesian). */
    translation: string;
    isActive: boolean;
  };

  let { ayah, translation, isActive }: Props = $props();

  let isBookmarked = $derived(
    bookmarkState.list.some(
      (b) => b.surah === surahState.selected.nomor && b.ayah === ayah.nomorAyat
    )
  );

  let copied = $state(false);
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

  async function copyAyah() {
    const text = [
      t('ayah.copyRef', surahState.selected.namaLatin, ayah.nomorAyat),
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
</script>

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
      translation,
      playbackState.lang
    )}
>
  <Icon name={isBookmarked ? 'bookmark-filled' : 'bookmark'} filled={isBookmarked} />
</button>

<button class="icon-button" type="button" aria-label={t('ayah.copy', ayah.nomorAyat)} onclick={copyAyah}>
  <Icon name={copied ? 'check' : 'copy'} />
</button>
