<script lang="ts">
  import Icon from '$lib/components/Icon.svelte';
  import { t } from '$lib/i18n';

  let {
    playing,
    repeat = false,
    showRepeat = false,
    playLabel,
    prevDisabled = false,
    nextDisabled = false,
    onPrevious,
    onPlayPause,
    onNext,
    onRepeat
  }: {
    playing: boolean;
    repeat?: boolean;
    showRepeat?: boolean;
    /** Overrides the play/pause label (finished sessions say "Play again"). */
    playLabel?: string;
    prevDisabled?: boolean;
    nextDisabled?: boolean;
    onPrevious: () => void;
    onPlayPause: () => void;
    onNext: () => void;
    onRepeat?: () => void;
  } = $props();
</script>

<div class="playback-controls" role="group" aria-label={t('play.controls')}>
  {#if showRepeat}
    <button
      type="button"
      class="playback-control-button playback-control-secondary {repeat ? 'is-active' : ''}"
      aria-label={repeat ? t('play.repeatOff') : t('play.repeatOn')}
      aria-pressed={repeat}
      onclick={onRepeat}
    >
      <Icon name="repeat" />
    </button>
  {/if}

  <button
    type="button"
    class="playback-control-button playback-control-secondary"
    aria-label={t('play.previous')}
    aria-disabled={prevDisabled}
    disabled={prevDisabled}
    onclick={onPrevious}
  >
    <Icon name="previous" />
  </button>

  <button
    type="button"
    class="playback-control-button playback-control-primary"
    aria-label={playLabel ?? (playing ? t('play.pause') : t('play.play'))}
    onclick={onPlayPause}
  >
    {#if playing}
      <Icon name="pause" filled />
    {:else}
      <Icon name="play" filled class="playback-icon-play" />
    {/if}
  </button>

  <button
    type="button"
    class="playback-control-button playback-control-secondary"
    aria-label={t('play.next')}
    aria-disabled={nextDisabled}
    disabled={nextDisabled}
    onclick={onNext}
  >
    <Icon name="next" />
  </button>
</div>

<style>
.playback-controls {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 10px;
}

.playback-control-button {
  display: grid;
  place-items: center;
  border: 1px solid var(--border-strong);
  border-radius: var(--r-control);
  background: var(--surface);
  color: var(--text-secondary);
  transition:
    background-color var(--speed-fast) var(--ease),
    border-color var(--speed-fast) var(--ease),
    color var(--speed-fast) var(--ease),
    transform var(--speed-fast) var(--ease);
}

.playback-control-button:disabled {
  opacity: 0.4;
  cursor: default;
}

.playback-control-button:hover {
  background: var(--raised);
  color: var(--text);
}

.playback-control-button:active {
  transform: scale(0.97);
}

.playback-control-button :global(svg) {
  width: 18px;
  height: 18px;
  fill: none;
  stroke: currentColor;
  stroke-width: 1.6;
  stroke-linecap: round;
  stroke-linejoin: round;
}

.playback-control-secondary {
  width: 36px;
  height: 36px;
}

.playback-control-primary {
  width: 44px;
  height: 44px;
  border-color: transparent;
  background: var(--accent);
  color: var(--on-accent);
}

.playback-control-primary:hover {
  background: var(--accent-hover);
  color: var(--on-accent);
}

.playback-control-primary :global(svg) {
  fill: currentColor;
  stroke: none;
  width: 20px;
  height: 20px;
}

:global(.playback-icon-play) {
  margin-left: 2px;
}
</style>
