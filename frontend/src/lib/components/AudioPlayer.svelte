<script lang="ts">
  import PlayerBar from '$lib/components/PlayerBar.svelte';
  import {
    playbackState,
    togglePlayback,
    nextAyah,
    prevAyah,
    toggleRepeat,
    endPlayback,
    retryPlayback
  } from '$lib/stores/playback.svelte';
  import { surahState } from '$lib/stores/surah.svelte';

  let showPlayer = $derived(
    playbackState.playing || playbackState.phase === 'paused' || playbackState.phase === 'finished'
  );

  // The bar describes the session, so it resolves the session's surah even
  // after the reader has browsed somewhere else.
  let sessionInfo = $derived(
    playbackState.sessionSurah === surahState.selected.nomor
      ? surahState.selected
      : surahState.list.find((surah) => surah.nomor === playbackState.sessionSurah)
  );
  let sessionTotal = $derived(
    playbackState.total || sessionInfo?.jumlahAyat || surahState.selected.jumlahAyat
  );
</script>

{#if showPlayer}
  <div class="player-bar-layer">
    <PlayerBar
      ayah={playbackState.currentAyah}
      total={sessionTotal}
      playing={playbackState.playing}
      repeat={playbackState.repeat}
      showRepeat
      qariLabel={playbackState.qariLabel}
      langLabel={playbackState.langLabel}
      surahName={sessionInfo?.namaLatin ?? `Surah ${playbackState.sessionSurah}`}
      surahArabic={sessionInfo?.nama ?? ''}
      prevDisabled={playbackState.currentAyah <= 1}
      nextDisabled={sessionTotal > 0 && playbackState.currentAyah >= sessionTotal}
      onRepeat={toggleRepeat}
      onPrevious={() => prevAyah(surahState.selected.nomor, sessionTotal)}
      onPlayPause={() => togglePlayback(playbackState.sessionSurah || surahState.selected.nomor, sessionTotal)}
      onNext={() => nextAyah(surahState.selected.nomor, sessionTotal)}
      onClose={() => void endPlayback()}
      onRetry={() => void retryPlayback()}
    />
  </div>
{/if}
