<script lang="ts">
  import {
    installNaturalIndonesianVoice,
    playbackState,
    refreshCacheStatus,
    refreshNaturalIndonesianVoiceStatus,
    setLang,
    setPrefetch,
    setQari,
    setTtsEnabled
  } from '$lib/stores/playback.svelte';
  import { surahState } from '$lib/stores/surah.svelte';
  import CustomSelect from './CustomSelect.svelte';
  import CustomToggle from './CustomToggle.svelte';
  import { onMount } from 'svelte';

  onMount(() => {
    void refreshCacheStatus(surahState.selected.nomor);
    void refreshNaturalIndonesianVoiceStatus();
  });

  let cachePercent = $derived(
    playbackState.cacheTotal > 0 ? (playbackState.cacheReady / playbackState.cacheTotal) * 100 : 0
  );
  let voiceProgressPercent = $derived(playbackState.naturalIndonesianProgress ?? 0);
  let voiceProgressLabel = $derived(`${Math.round(voiceProgressPercent)}% downloaded`);
  let isNaturalIndonesianReady = $derived(playbackState.naturalIndonesianStatus === 'ready');
  let showVoiceAction = $derived(
    playbackState.naturalIndonesianStatus === 'missing' || playbackState.naturalIndonesianStatus === 'failed'
  );
  let voiceStatusIcon = $derived.by(() => {
    if (playbackState.naturalIndonesianStatus === 'ready') return '✓';
    if (playbackState.naturalIndonesianStatus === 'failed') return '⚠';
    if (playbackState.naturalIndonesianStatus === 'installing') return '↧';
    return '○';
  });
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

    {#if !isNaturalIndonesianReady}
      <div class="translation-voice-card mb-5 rounded-2xl border border-[#d8c08a] bg-[#fffdf4]/85 p-4 shadow-sm">
        <div class="mb-3 flex items-start gap-3">
          <div class="flex h-9 w-9 shrink-0 items-center justify-center rounded-full bg-[#173f33] text-sm font-bold text-[#f8efd9]">
            {voiceStatusIcon}
          </div>
          <div>
            <h3 class="translation-voice-title text-sm font-semibold text-[#173f33]">Translation Voice</h3>
            <p class="translation-voice-copy mt-1 text-xs leading-relaxed text-[#5d4a2f]">{playbackState.naturalIndonesianMessage}</p>
          </div>
        </div>

        {#if playbackState.naturalIndonesianStatus === 'checking'}
          <p class="translation-voice-note rounded-xl bg-[#f8efd9] px-3 py-2 text-xs font-medium text-[#5d4a2f]">Checking Natural Indonesian Voice...</p>
        {:else if playbackState.naturalIndonesianStatus === 'installing'}
          <div class="space-y-2">
            <p class="translation-voice-copy text-xs font-medium text-[#5d4a2f]">{voiceProgressLabel}</p>
            <div class="translation-voice-track h-2 overflow-hidden rounded-full bg-[#ead9ad]">
              <div class="h-full rounded-full bg-[#173f33] transition-all" style={`width: ${voiceProgressPercent}%`}></div>
            </div>
          </div>
        {:else if showVoiceAction}
          <button
            type="button"
            class="mt-2 w-full rounded-xl bg-[#173f33] px-4 py-2.5 text-sm font-semibold text-[#f8efd9] shadow-sm transition hover:bg-[#215947] disabled:cursor-not-allowed disabled:opacity-60"
            disabled={playbackState.naturalIndonesianInstalling || !playbackState.naturalIndonesianCanDownload}
            onclick={() => void installNaturalIndonesianVoice()}
          >
            {playbackState.naturalIndonesianStatus === 'failed' ? 'Retry Download' : 'Download Voice'}
          </button>
        {/if}
      </div>
    {/if}

    {#if isNaturalIndonesianReady}
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
    {/if}

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
