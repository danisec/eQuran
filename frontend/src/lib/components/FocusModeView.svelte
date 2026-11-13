<script lang="ts">
  import Icon from '$lib/components/Icon.svelte';
  import PlayerBar from '$lib/components/PlayerBar.svelte';
  import VerseRosette from '$lib/components/VerseRosette.svelte';
  import { revelationLabel, t } from '$lib/i18n';
  import { disableFocusMode, focusState, setFocusAyah } from '$lib/stores/focus.svelte';
  import { endPlayback, nextAyah, playbackState, playFromAyah, prevAyah, togglePlayback, toggleRepeat } from '$lib/stores/playback.svelte';
  import { surahState } from '$lib/stores/surah.svelte';

  $effect(() => {
    if (playbackState.playing) {
      setFocusAyah(playbackState.currentAyah);
    }
  });

  let activeAyah = $derived(
    surahState.selected.ayat.find((ayah) => ayah.nomorAyat === focusState.selectedAyah) ??
      surahState.selected.ayat[0]
  );
  let translation = $derived(
    playbackState.lang === 'en' && activeAyah?.teksEnglish
      ? activeAyah.teksEnglish
      : activeAyah?.teksIndonesia
  );
  // Once playback starts the bar follows the recitation, so the verse number
  // and the progress always describe the same ayah.
  let displayAyah = $derived(
    playbackState.sessionSurah === surahState.selected.nomor && playbackState.phase !== 'idle'
      ? playbackState.currentAyah
      : focusState.selectedAyah
  );
  let verseLength = $derived(
    (activeAyah?.teksArab.length ?? 0) +
      (activeAyah?.teksLatin.length ?? 0) +
      (translation?.length ?? 0)
  );
  let focusDensity = $derived(verseLength > 900 ? 'dense' : verseLength > 520 ? 'comfortable' : 'spacious');
  let arabicLength = $derived(activeAyah?.teksArab.length ?? 0);
  let arabicDensity = $derived(
    arabicLength > 520 ? 'x-dense' : arabicLength > 340 ? 'dense' : arabicLength > 180 ? 'comfortable' : 'spacious'
  );

  function handlePrevious() {
    if (playbackState.phase !== 'idle' && playbackState.sessionSurah === surahState.selected.nomor) {
      prevAyah(surahState.selected.nomor, surahState.selected.jumlahAyat);
      return;
    }
    setFocusAyah(focusState.selectedAyah > 1 ? focusState.selectedAyah - 1 : surahState.selected.jumlahAyat);
  }

  function handleNext() {
    if (playbackState.phase !== 'idle' && playbackState.sessionSurah === surahState.selected.nomor) {
      nextAyah(surahState.selected.nomor, surahState.selected.jumlahAyat);
      return;
    }
    setFocusAyah(focusState.selectedAyah < surahState.selected.jumlahAyat ? focusState.selectedAyah + 1 : 1);
  }

  function handlePlayPause() {
    if (playbackState.sessionSurah === surahState.selected.nomor && playbackState.phase !== 'idle') {
      togglePlayback(surahState.selected.nomor, surahState.selected.jumlahAyat);
      return;
    }
    // In focus mode the verse on screen is the starting point, not ayah 1.
    playFromAyah(
      surahState.selected.nomor,
      focusState.selectedAyah,
      surahState.selected.jumlahAyat
    );
  }
</script>

<section class="focus-mode-screen">
  <header class="focus-mode-header">
    <div class="focus-surah-title">
      <div>
        <h1 class="text-[22px] font-medium text-[var(--text)]">{surahState.selected.namaLatin}</h1>
        <p class="mt-1 text-[13px] text-[var(--text-muted)]">
          {revelationLabel(surahState.selected.tempatTurun)} · {surahState.selected.jumlahAyat} {t('play.ayahCount')}
        </p>
      </div>
    </div>

    <button class="button focus-close" type="button" aria-label={t('focus.close')} onclick={disableFocusMode}>
      <Icon name="close" size={16} />
      {t('common.close')}
    </button>
  </header>

  {#if activeAyah}
    <article class="focus-verse-card focus-density-{focusDensity} focus-arabic-density-{arabicDensity}">
      <div class="focus-card-meta">
        <VerseRosette number={activeAyah.nomorAyat} size={44} active={playbackState.playing} />
        <div class="focus-phase-pill {playbackState.phase === 'recitation' ? 'is-active' : ''}">
          {t('play.phase.qari')}
        </div>
        <div class="focus-phase-pill {playbackState.phase === 'translation' ? 'is-active' : ''}">
          {playbackState.lang.toUpperCase()}
        </div>
        {#if playbackState.readTafsir}
          <div class="focus-phase-pill {playbackState.phase === 'tafsir' ? 'is-active' : ''}">
            {t('play.phase.tafsir')}
          </div>
        {/if}
      </div>

      <div class="focus-verse-scroll">
        <p class="arabic focus-arabic">{activeAyah.teksArab}</p>
        <p class="focus-latin">{activeAyah.teksLatin}</p>
        <div class="focus-divider"><span></span><span></span></div>
        <p class="focus-translation">{translation}</p>
      </div>
    </article>
  {/if}

  <PlayerBar
    ayah={displayAyah}
    total={surahState.selected.jumlahAyat}
    playing={playbackState.playing}
    repeat={playbackState.repeat}
    showRepeat
    qariLabel={playbackState.qariLabel}
    langLabel={playbackState.langLabel}
    surahName={surahState.selected.namaLatin}
    surahArabic={surahState.selected.nama}
    onPrevious={handlePrevious}
    onPlayPause={handlePlayPause}
    onNext={handleNext}
    onRepeat={toggleRepeat}
    onClose={() => void endPlayback()}
  />
</section>

<style>
.focus-mode-screen {
  position: relative;
  display: grid;
  height: 100vh;
  grid-template-rows: auto minmax(0, 1fr) auto;
  overflow: hidden;
  padding: 24px 32px;
  background: var(--page);
  color: var(--text);
}

.focus-mode-header {
  position: relative;
  display: flex;
  align-items: center;
  justify-content: flex-end;
  min-height: 40px;
  padding-bottom: 16px;
}

.focus-surah-title {
  position: absolute;
  left: 50%;
  transform: translateX(-50%);
  text-align: center;
  white-space: nowrap;
  pointer-events: none;
}

.focus-phase-pill {
  display: inline-flex;
  align-items: center;
  height: 24px;
  padding: 0 10px;
  border: 1px solid var(--border);
  border-radius: 6px;
  color: var(--text-muted);
  font-size: 12px;
}

.focus-close {
  position: relative;
  z-index: 1;
}

.focus-verse-card {
  display: flex;
  width: min(72rem, calc(100vw - 8rem));
  max-height: min(33rem, calc(100vh - 15.5rem));
  flex-direction: column;
  align-self: center;
  justify-self: center;
  border: 1px solid var(--border);
  border-radius: 20px;
  background: var(--surface);
  padding: 28px;
  box-shadow: var(--shadow-raised);
}

.focus-card-meta {
  flex: 0 0 auto;
  display: flex;
  align-items: center;
  gap: 12px;
  margin-bottom: 20px;
}

.focus-verse-scroll {
  position: relative;
  z-index: 1;
  min-height: 0;
  overflow-y: auto;
  padding: 0 4px 4px;
  scrollbar-width: thin;
  scrollbar-color: var(--border-strong) transparent;
}

.focus-arabic {
  font-size: 44px;
  line-height: 1.9;
  color: var(--text);
  text-align: center;
}

.focus-latin {
  margin-top: 16px;
  font-family: 'IBM Plex Serif', Georgia, serif;
  font-size: 20px;
  line-height: 1.5;
  color: var(--text-secondary);
  text-align: center;
}

.focus-divider {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 12px;
  margin: 20px 0;
  color: var(--border-strong);
}

.focus-divider span {
  height: 1px;
  width: 48px;
  background: currentColor;
}

.focus-translation {
  font-size: 17px;
  line-height: 1.7;
  color: var(--text-secondary);
  text-align: center;
}
</style>
