<script lang="ts">
  import { playbackState, setQari, setLang, setPrefetch, setTtsEnabled, refreshCacheStatus } from '$lib/stores/playback.svelte';
  import { surahState } from '$lib/stores/surah.svelte';
  import CustomSelect from './CustomSelect.svelte';
  import CustomToggle from './CustomToggle.svelte';
  import { onMount } from 'svelte';

  onMount(() => {
    void refreshCacheStatus(surahState.selected.nomor);
  });

  let cachePercent = $derived(
    playbackState.cacheTotal > 0 ? (playbackState.cacheReady / playbackState.cacheTotal) * 100 : 0
  );
  let showAdvanced = $state(false);

  const qariOptions = [
    { value: 'misyari', label: 'Mishary Rashid Alafasy', description: 'Balanced recitation' },
    { value: 'sudais', label: 'Abdurrahman As-Sudais', description: 'Makkah imam' },
    { value: 'dossari', label: 'Ibrahim Al-Dossari', description: 'Clear cadence' },
    { value: 'juhany', label: 'Abdullah Al-Juhany', description: 'Haram reciter' },
    { value: 'qasim', label: 'Abdul Muhsin Al-Qasim', description: 'Madinah imam' },
    { value: 'yasser', label: 'Yasser Al-Dosari', description: 'Warm tone' }
  ];

  const languageOptions = [
    { value: 'id', label: 'Indonesian', description: 'Terjemahan Indonesia' },
    { value: 'en', label: 'English', description: 'English translation' }
  ];
</script>

<aside class="row-span-1 border-l border-[#d7c294] bg-[#f8efd9]/85 p-5 overflow-y-auto">
  <section class="rounded-2xl border border-[#d8c08a] bg-[#fff8e8]/80 p-5 shadow-sm">
    <h2 class="mb-6 text-center text-xl font-semibold text-[#173f33]">Playback Settings</h2>

    <div class="mb-5">
      <CustomSelect label="Qari" value={playbackState.qari} options={qariOptions} onChange={setQari} />
    </div>

    <div class="mb-5">
      <CustomSelect label="Language (Translation)" value={playbackState.lang} options={languageOptions} onChange={setLang} />
    </div>

    <div class="mb-4">
      <CustomToggle
        label="TTS Translation"
        description="Read translation after recitation"
        checked={playbackState.ttsEnabled}
        onChange={setTtsEnabled}
      />
    </div>

    <div class="mb-6">
      <CustomToggle
        label="Prefetch Translation Audio"
        description="Prepare the next ayah voice early"
        checked={playbackState.prefetch}
        onChange={setPrefetch}
      />
    </div>

    <div class="advanced-settings">
      <button
        type="button"
        class="advanced-settings-trigger"
        aria-expanded={showAdvanced}
        onclick={() => showAdvanced = !showAdvanced}
      >
        <span>
          <span class="advanced-settings-title">Advanced</span>
          <span class="advanced-settings-summary">Audio preparation details</span>
        </span>
        <svg class:open={showAdvanced} viewBox="0 0 20 20" class="advanced-settings-chevron" aria-hidden="true">
          <path d="M5.25 7.75 10 12.5l4.75-4.75" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round" />
        </svg>
      </button>

      {#if showAdvanced}
        <div class="cache-status-panel">
          <h3 class="cache-status-title">🪙 Voice Audio Cache</h3>
          <p class="cache-status-copy">Prepared {playbackState.cacheReady} of {playbackState.cacheTotal} ayah translation audio files.</p>
          <div class="cache-status-track">
            <div class="cache-status-fill" style={`width: ${cachePercent}%`}></div>
          </div>
          {#if playbackState.playing && playbackState.phase === 'cache'}
            <p class="cache-status-live">
              <span></span>
              Preparing ayah {playbackState.currentAyah} translation voice
            </p>
          {/if}
        </div>
      {/if}
    </div>
  </section>
</aside>
