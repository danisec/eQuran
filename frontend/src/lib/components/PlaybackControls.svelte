<script lang="ts">
  type PlaybackControlsVariant = 'compact' | 'focus';

  let {
    playing,
    repeat = false,
    showRepeat = false,
    variant = 'compact',
    onPrevious,
    onPlayPause,
    onNext,
    onRepeat
  }: {
    playing: boolean;
    repeat?: boolean;
    showRepeat?: boolean;
    variant?: PlaybackControlsVariant;
    onPrevious: () => void;
    onPlayPause: () => void;
    onNext: () => void;
    onRepeat?: () => void;
  } = $props();
</script>

<div class="playback-controls playback-controls-{variant}" aria-label="Playback controls">
  {#if showRepeat}
    <button
      type="button"
      class="playback-control-button playback-control-secondary {repeat ? 'is-active' : ''}"
      aria-label={repeat ? 'Disable repeat' : 'Enable repeat'}
      aria-pressed={repeat}
      onclick={onRepeat}
    >
      <svg viewBox="0 0 24 24" aria-hidden="true">
        <path d="M7 7h8.6a3.4 3.4 0 0 1 3.4 3.4v.6" />
        <path d="m16 4 3 3-3 3" />
        <path d="M17 17H8.4A3.4 3.4 0 0 1 5 13.6V13" />
        <path d="m8 20-3-3 3-3" />
      </svg>
    </button>
  {/if}

  <button type="button" class="playback-control-button playback-control-secondary" aria-label="Previous ayah" onclick={onPrevious}>
    <svg viewBox="0 0 24 24" aria-hidden="true">
      <path d="M6 5v14" />
      <path d="m18 6-9 6 9 6V6Z" />
    </svg>
  </button>

  <button type="button" class="playback-control-button playback-control-primary" aria-label={playing ? 'Pause' : 'Play'} onclick={onPlayPause}>
    {#if playing}
      <svg viewBox="0 0 24 24" aria-hidden="true">
        <rect x="7" y="5" width="3.8" height="14" rx="1" />
        <rect x="13.2" y="5" width="3.8" height="14" rx="1" />
      </svg>
    {:else}
      <svg viewBox="0 0 24 24" aria-hidden="true" class="playback-icon-play">
        <path d="M8 5.5v13l10-6.5-10-6.5Z" />
      </svg>
    {/if}
  </button>

  <button type="button" class="playback-control-button playback-control-secondary" aria-label="Next ayah" onclick={onNext}>
    <svg viewBox="0 0 24 24" aria-hidden="true">
      <path d="M18 5v14" />
      <path d="m6 6 9 6-9 6V6Z" />
    </svg>
  </button>
</div>
