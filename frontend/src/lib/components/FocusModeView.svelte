<script lang="ts">
  import PlaybackControls from '$lib/components/PlaybackControls.svelte';
  import { disableFocusMode, focusState, setFocusAyah } from '$lib/stores/focus.svelte';
  import { nextAyah, playbackState, prevAyah, togglePlayback } from '$lib/stores/playback.svelte';
  import { surahState } from '$lib/stores/surah.svelte';

  $effect(() => {
    if (playbackState.playing) {
      setFocusAyah(playbackState.currentAyah);
    }
  });

  let activeAyah = $derived(
    surahState.selected.ayat.find((ayah) => ayah.nomorAyat === focusState.selectedAyah) ?? surahState.selected.ayat[0]
  );
  let translation = $derived(playbackState.lang === 'en' && activeAyah?.teksEnglish ? activeAyah.teksEnglish : activeAyah?.teksIndonesia);
  let progressPercent = $derived(
    surahState.selected.jumlahAyat > 0 ? (focusState.selectedAyah / surahState.selected.jumlahAyat) * 100 : 0
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
    if (playbackState.playing) {
      prevAyah(surahState.selected.nomor, surahState.selected.jumlahAyat);
      return;
    }

    const previous = focusState.selectedAyah > 1 ? focusState.selectedAyah - 1 : surahState.selected.jumlahAyat;
    setFocusAyah(previous);
  }

  function handleNext() {
    if (playbackState.playing) {
      nextAyah(surahState.selected.nomor, surahState.selected.jumlahAyat);
      return;
    }

    const next = focusState.selectedAyah < surahState.selected.jumlahAyat ? focusState.selectedAyah + 1 : 1;
    setFocusAyah(next);
  }

  function handlePlayPause() {
    togglePlayback(surahState.selected.nomor, focusState.selectedAyah, undefined);
  }
</script>

<section class="focus-mode-screen">
  <header class="focus-mode-header">
    <div class="focus-brand-lockup">
      <div class="focus-logo-mark">☪</div>
      <div>
        <div class="focus-logo-text">eQuran</div>
        <div class="focus-mode-label">Focus Mode</div>
      </div>
    </div>

    <div class="focus-surah-title">
      <span class="text-[#c8a951]">❊</span>
      <div>
        <h1>Surah {surahState.selected.namaLatin}</h1>
        <p>{surahState.selected.tempatTurun === 'Madinah' ? 'Madaniyah' : 'Makkiyah'} • {surahState.selected.jumlahAyat} Ayah</p>
      </div>
      <span class="text-[#c8a951]">❊</span>
    </div>

    <button class="focus-close" type="button" aria-label="Keluar dari Focus Mode" onclick={disableFocusMode}>
      <span aria-hidden="true">×</span>
      Exit
    </button>
  </header>

  {#if activeAyah}
    <article class="focus-verse-card focus-density-{focusDensity} focus-arabic-density-{arabicDensity}">
      <div class="focus-card-ornament" aria-hidden="true"></div>
      <div class="focus-card-meta">
        <div class="focus-ayah-badge">{activeAyah.nomorAyat}</div>
        <div class="focus-phase-pill {playbackState.phase === 'recitation' ? 'is-active' : ''}">Qari {playbackState.phase === 'recitation' ? '●' : '○'}</div>
        <div class="focus-phase-pill {playbackState.phase === 'translation' ? 'is-active' : ''}">{playbackState.lang.toUpperCase()} {playbackState.phase === 'translation' ? '●' : '○'}</div>
      </div>
      <div class="focus-verse-scroll">
        <p class="arabic focus-arabic">{activeAyah.teksArab}</p>
        <p class="focus-latin">{activeAyah.teksLatin}</p>
        <div class="focus-divider"><span></span><b>◆</b><span></span></div>
        <p class="focus-translation">{translation}</p>
      </div>
    </article>
  {/if}

  <footer class="focus-controls">
    <div class="focus-now-playing">
      <div class="focus-mini-surah arabic">{surahState.selected.nama}</div>
      <div>
        <p>Now Playing: {surahState.selected.namaLatin} • Ayah {focusState.selectedAyah}</p>
        <span>🎙 {playbackState.qariLabel}</span>
      </div>
    </div>

    <div class="focus-control-center">
      <div class="focus-counter">{focusState.selectedAyah} / {surahState.selected.jumlahAyat}</div>
      <div class="focus-progress" aria-hidden="true">
        <div class="focus-progress-fill" style={`width: ${progressPercent}%`}></div>
      </div>
    </div>

    <PlaybackControls
      playing={playbackState.playing}
      variant="focus"
      onPrevious={handlePrevious}
      onPlayPause={handlePlayPause}
      onNext={handleNext}
    />
  </footer>
</section>
