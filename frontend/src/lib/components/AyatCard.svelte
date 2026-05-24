<script lang="ts">
  import { playbackState, startPlayback, stopPlayback } from '$lib/stores/playback.svelte';
  import { surahState } from '$lib/stores/surah.svelte';
  import { bookmarkState, toggleBookmark } from '$lib/stores/bookmarks.svelte';
  import { focusState } from '$lib/stores/focus.svelte';
  import { loadTafsir, playTafsirVoice, tafsirForAyah, tafsirState } from '$lib/stores/tafsir.svelte';
  import type { Ayat } from '$lib/stores/surah.svelte';

  let { ayah }: { ayah: Ayat } = $props();

  let isActive = $derived(playbackState.currentAyah === ayah.nomorAyat && playbackState.playing);
  let isBookmarked = $derived(bookmarkState.list.some((b) => b.surah === surahState.selected.nomor && b.ayah === ayah.nomorAyat));
  let translation = $derived(playbackState.lang === 'en' && ayah.teksEnglish ? ayah.teksEnglish : ayah.teksIndonesia);
  let copied = $state(false);
  let showTafsir = $state(false);
  let copyTimer: ReturnType<typeof setTimeout> | undefined;
  let tafsir = $derived(tafsirForAyah(surahState.selected.nomor, ayah.nomorAyat));
  let tafsirLoading = $derived(tafsirState.loadingSurah === surahState.selected.nomor && showTafsir && !tafsir);
  let tafsirSpeaking = $derived(tafsirState.speakingKey === `${surahState.selected.nomor}:${ayah.nomorAyat}`);

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
    if (copyTimer) {
      clearTimeout(copyTimer);
    }
    copyTimer = setTimeout(() => {
      copied = false;
    }, 1200);
  }

  async function toggleTafsirDetail() {
    showTafsir = !showTafsir;
    if (showTafsir && !tafsir) {
      await loadTafsir(surahState.selected.nomor);
    }
  }
</script>

<article class="rounded-2xl border p-6 shadow-sm transition-all duration-200 {focusState.enabled ? 'focus-ayah px-8 py-9 md:px-12' : ''} {isActive ? 'border-[#2f9b62] bg-[#f6fff0] shadow-md shadow-[#2f9b62]/15 ring-1 ring-[#2f9b62]/30' : 'border-[#d8c08a] bg-[#fff8e8] hover:shadow-md'}">
  <div class="mb-5 flex items-start justify-between gap-6">
    <div class="relative shrink-0">
      <svg viewBox="0 0 64 64" class="h-16 w-16" xmlns="http://www.w3.org/2000/svg">
        <polygon
          points="32,2 45,9 58,9 58,22 64,32 58,42 58,55 45,55 32,62 19,55 6,55 6,42 0,32 6,22 6,9 19,9"
          fill={isActive ? '#0b5a43' : '#fffdf2'}
          stroke={isActive ? '#2f9b62' : '#d0a34e'}
          stroke-width="2"
        />
      </svg>
      <span class="absolute inset-0 flex items-center justify-center text-lg font-bold {isActive ? 'text-white' : 'text-[#0b5a43]'}">{ayah.nomorAyat}</span>
    </div>

    {#if isActive}
      <span class="mt-2 flex items-center gap-1 text-sm text-[#2f9b62]">
        <span class="inline-block h-2 w-2 animate-pulse rounded-full bg-[#2f9b62]"></span>
        Playing
      </span>
    {/if}

    <p class="arabic flex-1 text-right leading-[2.2] text-black {focusState.enabled ? 'text-5xl md:text-6xl' : 'text-4xl'}">{ayah.teksArab}</p>
  </div>

  <p class="mb-2 font-serif italic leading-relaxed text-[#315949] {focusState.enabled ? 'text-2xl' : 'text-xl'}">{ayah.teksLatin}</p>
  <p class="leading-relaxed text-[#1f1b17] {focusState.enabled ? 'text-xl' : 'text-lg'}">{translation}</p>

  <div class="mt-5 flex items-center justify-end gap-4">
    <button
      aria-label="Play ayah {ayah.nomorAyat}"
      class="grid h-10 w-10 place-items-center rounded-full border transition {isActive ? 'border-[#2f9b62] bg-[#2f9b62] text-white' : 'border-[#d0a34e] bg-[#fffdf2] text-[#0b5a43] hover:bg-[#f0e8c8]'}"
      onclick={() => isActive ? stopPlayback() : startPlayback(surahState.selected.nomor, ayah.nomorAyat, ayah.nomorAyat)}
    >
      {#if isActive}
        <svg viewBox="0 0 24 24" class="h-[1.125rem] w-[1.125rem] fill-current" aria-hidden="true">
          <rect x="6.5" y="5" width="4" height="14" rx="1" />
          <rect x="13.5" y="5" width="4" height="14" rx="1" />
        </svg>
      {:else}
        <svg viewBox="0 0 24 24" class="ml-0.5 h-[1.125rem] w-[1.125rem] fill-current" aria-hidden="true">
          <polygon points="7,5 19,12 7,19" />
        </svg>
      {/if}
    </button>
    <button
      aria-label="Bookmark ayah {ayah.nomorAyat}"
      class="grid h-10 w-10 place-items-center rounded-full border transition {isBookmarked ? 'border-[#e74c3c] bg-[#fdf0ef] text-[#e74c3c]' : 'border-[#d0a34e] bg-[#fffdf2] text-[#0b5a43] hover:bg-[#f0e8c8]'}"
      onclick={() => toggleBookmark(surahState.selected.nomor, ayah.nomorAyat, surahState.selected.namaLatin, ayah.teksArab.slice(0, 50))}
    >
      <svg viewBox="0 0 24 24" class="h-5 w-5 {isBookmarked ? 'fill-current' : 'fill-none'}" aria-hidden="true">
        <path d="M7 4.75A2.75 2.75 0 0 1 9.75 2h4.5A2.75 2.75 0 0 1 17 4.75v16l-5-3.15L7 20.75z" stroke="currentColor" stroke-width="1.8" stroke-linejoin="round" />
      </svg>
    </button>
    <button
      aria-label="Copy ayah {ayah.nomorAyat}"
      class="grid h-10 w-10 place-items-center rounded-full border transition {copied ? 'border-[#2f9b62] bg-[#f0fff4] text-[#2f9b62]' : 'border-[#d0a34e] bg-[#fffdf2] text-[#0b5a43] hover:bg-[#f0e8c8]'}"
      onclick={copyAyah}
    >
      {copied ? '✓' : '⧉'}
    </button>
    <button
      aria-expanded={showTafsir}
      aria-label="Tampilkan tafsir ayah {ayah.nomorAyat}"
      class="inline-flex h-10 items-center gap-2 rounded-full border px-4 text-sm font-semibold transition {showTafsir ? 'border-[#2f9b62] bg-[#f0fff4] text-[#2f9b62]' : 'border-[#d0a34e] bg-[#fffdf2] text-[#0b5a43] hover:bg-[#f0e8c8]'}"
      onclick={toggleTafsirDetail}
    >
      Tafsir {showTafsir ? '▴' : '▾'}
    </button>
  </div>

  {#if showTafsir}
    <section class="tafsir-panel mt-5 rounded-2xl border border-[#ead8a8] bg-[#fff2cf]/70 p-5 text-left">
      <div class="mb-3 flex items-center justify-between gap-3">
        <h3 class="tafsir-title text-base font-bold text-[#0b4a39]">Tafsir Ayat {ayah.nomorAyat}</h3>
        {#if tafsir}
          <button
            aria-label="Bacakan tafsir ayat {ayah.nomorAyat}"
            class="tafsir-voice-button inline-flex items-center gap-2 rounded-full border border-[#d8c08a] bg-[#fffdf2] px-3 py-1 text-xs font-semibold text-[#0b5a43] transition hover:bg-[#f0e8c8] disabled:cursor-wait disabled:opacity-70"
            disabled={tafsirSpeaking}
            onclick={() => playTafsirVoice(surahState.selected.nomor, ayah.nomorAyat, tafsir.teks)}
          >
            {tafsirSpeaking ? 'Membacakan...' : '🔊 Voice'}
          </button>
        {:else}
          <span class="tafsir-detail-badge rounded-full border border-[#d8c08a] px-3 py-1 text-xs font-semibold text-[#9a6f2a]">Detail</span>
        {/if}
      </div>
      {#if tafsirLoading}
        <p class="tafsir-status text-sm text-[#5a5345]">Memuat tafsir...</p>
      {:else if tafsir}
        <p class="tafsir-body whitespace-pre-line text-sm leading-7 text-[#1f1b17]">{tafsir.teks}</p>
      {:else}
        <p class="tafsir-status text-sm text-[#8a7a5a]">{tafsirState.error || 'Tafsir belum tersedia untuk ayat ini.'}</p>
      {/if}
    </section>
  {/if}
</article>
