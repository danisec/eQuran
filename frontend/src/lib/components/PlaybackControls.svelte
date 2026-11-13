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
