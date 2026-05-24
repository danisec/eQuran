<script lang="ts">
  import PlaybackControls from '$lib/components/PlaybackControls.svelte';
  import { playbackState, togglePlayback, nextAyah, prevAyah, toggleRepeat } from '$lib/stores/playback.svelte';
  import { surahState } from '$lib/stores/surah.svelte';
</script>

<footer class="col-span-3 grid grid-cols-[430px_1fr] items-center gap-6 border-t border-[#c9a85e] bg-[#fff5df]/95 px-8 shadow-2xl">
  <div class="flex items-center gap-4">
    <div class="grid h-20 w-20 place-items-center rounded-xl border-2 border-[#d0a34e] bg-[#0d5b43] arabic text-2xl text-[#f3d376]">{surahState.selected.nama}</div>
    <div>
      <p class="text-lg font-semibold text-[#0b4a39]">Now Playing: {surahState.selected.namaLatin} • Ayah {playbackState.currentAyah}</p>
      <p class="mt-1 text-sm text-[#5a5345]">🎙 {playbackState.qariLabel} • {playbackState.langLabel} translation</p>
    </div>
  </div>

  <div class="text-center">
    <PlaybackControls
      playing={playbackState.playing}
      repeat={playbackState.repeat}
      showRepeat
      variant="compact"
      onRepeat={toggleRepeat}
      onPrevious={() => prevAyah(surahState.selected.nomor, surahState.selected.jumlahAyat)}
      onPlayPause={() => togglePlayback(surahState.selected.nomor)}
      onNext={() => nextAyah(surahState.selected.nomor, surahState.selected.jumlahAyat)}
    />
    <div class="flex items-center gap-3 text-xs text-[#5a5345]">
      <span>{playbackState.current}/{playbackState.total}</span>
      <div class="relative h-2 flex-1 overflow-hidden rounded-full bg-[#e8d9b0]">
        <div class="absolute inset-y-0 left-0 rounded-full bg-[#c29b43] transition-all" style={`width: ${playbackState.total > 0 ? (playbackState.current / playbackState.total) * 100 : 0}%`}></div>
      </div>
      <span>{playbackState.total} ayah</span>
    </div>
  </div>
</footer>
