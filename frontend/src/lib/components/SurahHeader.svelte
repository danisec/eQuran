<script lang="ts">
  import Icon from '$lib/components/Icon.svelte';
  import SurahPicker from './SurahPicker.svelte';
  import { revelationLabel, surahMeaning, t } from '$lib/i18n';
  import { enableFocusMode, setFocusAyah } from '$lib/stores/focus.svelte';
  import { playbackState, playSurah, togglePlayback } from '$lib/stores/playback.svelte';
  import { surahState } from '$lib/stores/surah.svelte';
  import { bookmarkState } from '$lib/stores/bookmarks.svelte';
  import { panelState, toggleMarks } from '$lib/stores/panels.svelte';

  function handleFocusMode() {
    setFocusAyah(surahState.visibleAyah || playbackState.currentAyah || 1);
    enableFocusMode();
  }

  // The bar's primary action: one press listens to the whole surah, and the
  // same button pauses or resumes it. It only acts on this surah's session, so
  // a session playing elsewhere is not hijacked.
  let sameSurahSession = $derived(playbackState.sessionSurah === surahState.selected.nomor);
  let listenActive = $derived(
    sameSurahSession && (playbackState.playing || playbackState.phase === 'paused')
  );
  let listenLabel = $derived(
    listenActive
      ? playbackState.playing
        ? t('play.pause')
        : t('play.resume')
      : sameSurahSession && playbackState.phase === 'finished'
        ? t('play.replay')
        : t('play.listen')
  );

  function handleListen() {
    if (listenActive || (sameSurahSession && playbackState.phase === 'finished')) {
      togglePlayback(surahState.selected.nomor, surahState.selected.jumlahAyat);
      return;
    }
    playSurah(surahState.selected.nomor, surahState.selected.jumlahAyat);
  }
</script>

<header class="relative mb-8">
  <button class="button absolute left-0 top-0" type="button" onclick={handleListen}>
    <Icon name={listenActive && playbackState.playing ? 'pause' : 'play'} size={16} filled />
    {listenLabel}
  </button>

  <div class="absolute right-0 top-0 flex items-center gap-1">
    <SurahPicker />
    <button
      class="icon-button {panelState.marksOpen ? 'is-on' : ''}"
      type="button"
      aria-expanded={panelState.marksOpen}
      aria-label={t('marks.toggle')}
      title={t('marks.toggle')}
      onclick={toggleMarks}
    >
      <Icon name="bookmark" filled={panelState.marksOpen} />
      {#if bookmarkState.list.length > 0}
        <span class="count-dot tnum">{bookmarkState.list.length}</span>
      {/if}
    </button>
    <button class="button" type="button" onclick={handleFocusMode}>
      <Icon name="maximize" size={16} />
      {t('focus.enter')}
    </button>
  </div>

  <div class="px-24 text-center">
    <h1 class="text-[30px] font-medium leading-tight tracking-[-0.02em] text-[var(--text)]">
      {surahState.selected.namaLatin}
    </h1>
    <p class="arabic mt-1 text-[24px] leading-none text-[var(--text-secondary)]">
      {surahState.selected.nama}
    </p>
    <p class="mt-3 flex items-center justify-center gap-2 text-[13px] text-[var(--text-muted)]">
      <span>{revelationLabel(surahState.selected.tempatTurun)}</span>
      <span>·</span>
      <span class="tnum">{surahState.selected.jumlahAyat} {t('play.ayahCount')}</span>
      <span>·</span>
      <span>{surahMeaning(surahState.selected.nomor, surahState.selected.arti)}</span>
    </p>
  </div>
</header>
