<script lang="ts">
  import { tick } from 'svelte';
  import { clearPendingScrollAyah, surahState } from '$lib/stores/surah.svelte';
  import { playbackState } from '$lib/stores/playback.svelte';
  import SurahHeader from './SurahHeader.svelte';
  import AyatCard from './AyatCard.svelte';

  let scrollContainer: HTMLDivElement;

  $effect(() => {
    const ayah = playbackState.currentAyah;
    if (ayah && scrollContainer) {
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

</script>

<section class="min-h-0 overflow-hidden px-8 pb-4 pt-6">
  <SurahHeader />

  <div bind:this={scrollContainer} class="h-[calc(100vh-300px)] space-y-5 overflow-y-auto pb-8 pr-2">
    {#each surahState.selected.ayat as ayah}
      <div data-ayah={ayah.nomorAyat}>
        <AyatCard {ayah} />
      </div>
    {/each}
  </div>
</section>
