<script lang="ts">
  import Icon from '$lib/components/Icon.svelte';
  import TafsirPanel from '$lib/components/TafsirPanel.svelte';
  import VerseActions from '$lib/components/VerseActions.svelte';
  import VerseRosette from '$lib/components/VerseRosette.svelte';
  import { t } from '$lib/i18n';
  import { playbackState } from '$lib/stores/playback.svelte';
  import { surahState } from '$lib/stores/surah.svelte';
  import { loadTafsir, tafsirForAyah, tafsirState } from '$lib/stores/tafsir.svelte';
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
  let translation = $derived(
    playbackState.lang === 'en' && ayah.teksEnglish ? ayah.teksEnglish : ayah.teksIndonesia
  );

  let showTafsir = $state(false);
  let tafsirLoading = $derived(
    tafsirState.loadingSurah === surahState.selected.nomor &&
      showTafsir &&
      !tafsirForAyah(surahState.selected.nomor, ayah.nomorAyat)
  );

  async function toggleTafsirDetail() {
    showTafsir = !showTafsir;
    if (showTafsir && !tafsirForAyah(surahState.selected.nomor, ayah.nomorAyat)) {
      await loadTafsir(surahState.selected.nomor);
    }
  }
</script>

<article class="verse-card {isActive ? 'is-active' : ''}">
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
    <VerseActions {ayah} {translation} {isActive} />

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
    <TafsirPanel ayahNumber={ayah.nomorAyat} loading={tafsirLoading} />
  {/if}
</article>

<style>
.verse-card {
  border: 1px solid var(--border);
  border-radius: var(--r-card);
  background: var(--surface);
  padding: 24px;
  box-shadow: var(--shadow-card);
  transition: border-color var(--speed) var(--ease), background-color var(--speed) var(--ease);
}

.verse-card:hover {
  border-color: var(--border-strong);
}

.verse-arabic {
  font-size: 34px;
  line-height: 2;
  color: var(--text);
  text-align: right;
}

.verse-latin {
  font-family: 'IBM Plex Serif', Georgia, serif;
  font-size: 17px;
  line-height: 1.6;
  color: var(--text-secondary);
}

.verse-translation {
  font-size: 16px;
  line-height: 1.65;
  color: var(--text-secondary);
}

.verse-active-note {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 12px;
  color: var(--accent);
}

.verse-active-note span {
  width: 6px;
  height: 6px;
  border-radius: 50%;
  background: currentColor;
}
</style>
