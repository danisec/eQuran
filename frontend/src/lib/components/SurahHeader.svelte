<script lang="ts">
  import { enableFocusMode, setFocusAyah } from '$lib/stores/focus.svelte';
  import { playbackState } from '$lib/stores/playback.svelte';
  import { surahState } from '$lib/stores/surah.svelte';

  function handleFocusMode() {
    setFocusAyah(playbackState.currentAyah || 1);
    enableFocusMode();
  }
</script>

<header class="relative mb-8 text-center">
  <div class="absolute left-1/2 top-1/2 -z-10 h-40 w-[600px] -translate-x-1/2 -translate-y-1/2 opacity-10">
    <svg viewBox="0 0 600 160" fill="none" xmlns="http://www.w3.org/2000/svg" class="h-full w-full">
      <path d="M300 10 L320 30 L340 20 L360 35 L380 15 L400 40 L420 25 L440 45 L460 30 L480 50 L500 35 L520 55 L540 40 L560 60 L580 45 L600 65" stroke="#c8a951" stroke-width="1.5" fill="none" />
      <path d="M300 10 L280 30 L260 20 L240 35 L220 15 L200 40 L180 25 L160 45 L140 30 L120 50 L100 35 L80 55 L60 40 L40 60 L20 45 L0 65" stroke="#c8a951" stroke-width="1.5" fill="none" />
      <path d="M300 150 L320 130 L340 140 L360 125 L380 145 L400 120 L420 135 L440 115 L460 130 L480 110 L500 125 L520 105 L540 120 L560 100 L580 115 L600 95" stroke="#c8a951" stroke-width="1.5" fill="none" />
      <path d="M300 150 L280 130 L260 140 L240 125 L220 145 L200 120 L180 135 L160 115 L140 130 L120 110 L100 125 L80 105 L60 120 L40 100 L20 115 L0 95" stroke="#c8a951" stroke-width="1.5" fill="none" />
    </svg>
  </div>

  <div class="mb-2 flex items-center justify-center gap-4">
    <span class="text-2xl text-[#c8a951]">❊</span>
    <h2 class="font-serif text-5xl font-bold text-[#0b4a39]">Surah {surahState.selected.namaLatin}</h2>
  </div>

  <button
    aria-label="Masuk ke Focus Mode"
    class="absolute right-0 top-2 inline-flex h-10 items-center gap-2 rounded-lg border border-[#d0a34e] bg-[#fffdf2] px-3 text-sm font-bold text-[#0b5a43] transition hover:bg-[#f0e8c8]"
    type="button"
    onclick={handleFocusMode}
  >
    <span aria-hidden="true">◉</span>
    Focus
  </button>

  <p class="text-lg tracking-wide text-[#9a6f2a]">
    <span class="font-medium">{surahState.selected.tempatTurun === 'Madinah' ? 'Madaniyah' : 'Makkiyah'}</span>
    <span class="mx-2">•</span>
    <span>{surahState.selected.jumlahAyat} Ayah</span>
  </p>

  <div class="mx-auto mt-4 flex w-64 items-center gap-2">
    <span class="h-px flex-1 bg-gradient-to-r from-transparent to-[#c8a951]"></span>
    <span class="text-[#c8a951]">✦</span>
    <span class="h-px flex-1 bg-gradient-to-l from-transparent to-[#c8a951]"></span>
  </div>
</header>
