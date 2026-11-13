<script lang="ts">
  import { tick } from 'svelte';
  import { clearPendingScrollAyah, surahState } from '$lib/stores/surah.svelte';
  import { playbackState } from '$lib/stores/playback.svelte';
  import SurahHeader from './SurahHeader.svelte';
  import AyatCard from './AyatCard.svelte';

  let scrollContainer: HTMLDivElement;
  let observer: IntersectionObserver | null = null;

  $effect(() => {
    const ayah = playbackState.currentAyah;
    if (ayah && scrollContainer && (playbackState.playing || playbackState.phase === 'paused')) {
      const el = scrollContainer.querySelector(`[data-ayah="${ayah}"]`);
      el?.scrollIntoView({ behavior: 'smooth', block: 'center' });
    }
  });

  $effect(() => {
    const ayah = surahState.pendingScrollAyah;
    const surahNumber = surahState.selected.nomor;
    if (ayah && surahNumber && scrollContainer) {
      void tick().then(() => {
        const el = scrollContainer.querySelector(`[data-ayah="${ayah}"]`);
        el?.scrollIntoView({ behavior: 'smooth', block: 'center' });
        clearPendingScrollAyah();
      });
    }
  });

  $effect(() => {
    void surahState.selected.nomor;
    if (!scrollContainer) return;
    observer?.disconnect();
    observer = new IntersectionObserver(
      (entries) => {
        const visible = entries
          .filter((e) => e.isIntersecting)
          .sort((a, b) => a.boundingClientRect.top - b.boundingClientRect.top);
        if (visible[0]) {
          const ayah = Number(visible[0].target.getAttribute('data-ayah'));
          if (ayah) surahState.visibleAyah = ayah;
        }
      },
      { root: scrollContainer, threshold: 0, rootMargin: '-20% 0px -60% 0px' }
    );
    void tick().then(() => {
      for (const child of scrollContainer!.querySelectorAll('[data-ayah]')) {
        observer?.observe(child);
      }
    });
    return () => observer?.disconnect();
  });

</script>

<section class="flex min-h-0 flex-col overflow-hidden px-8 pb-4 pt-6">
  <SurahHeader />

  <div
    bind:this={scrollContainer}
    class="flex-1 space-y-5 overflow-y-auto pr-2 {playbackState.playing || playbackState.phase === 'paused' || playbackState.phase === 'finished'
      ? 'pb-32'
      : 'pb-8'}"
  >
    {#each surahState.selected.ayat as ayah}
      <div data-ayah={ayah.nomorAyat}>
        <AyatCard {ayah} />
      </div>
    {/each}
  </div>
</section>
