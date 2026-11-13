<script lang="ts">
  import Icon from '$lib/components/Icon.svelte';
  import PlaybackControls from '$lib/components/PlaybackControls.svelte';
  import { t } from '$lib/i18n';
  import { playbackState } from '$lib/stores/playback.svelte';

  /**
   * The one player bar.
   *
   * Baca and Mode fokus both mount this, so the controls, type, spacing and
   * progress can no longer drift apart the way two hand-rolled bars did.
   * `ayah` is the verse currently shown (in focus it may not be the one playing
   * yet) and `total` is the surah length, so the figure reads the same
   * everywhere.
   *
   * The surah name is passed in rather than read from the browsing state: the
   * bar describes the session, and navigating the mushaf must not relabel it.
   */
  let {
    ayah,
    total,
    playing,
    repeat = false,
    showRepeat = false,
    qariLabel,
    langLabel,
    surahName,
    surahArabic,
    prevDisabled = false,
    nextDisabled = false,
    onPrevious,
    onPlayPause,
    onNext,
    onRepeat,
    onClose,
    onRetry
  }: {
    ayah: number;
    total: number;
    playing: boolean;
    repeat?: boolean;
    showRepeat?: boolean;
    qariLabel: string;
    langLabel: string;
    surahName: string;
    surahArabic: string;
    prevDisabled?: boolean;
    nextDisabled?: boolean;
    onPrevious: () => void;
    onPlayPause: () => void;
    onNext: () => void;
    onRepeat?: () => void;
    /** Ends the session and dismisses the bar. */
    onClose: () => void;
    onRetry?: () => void;
  } = $props();

  let progressPercent = $derived(total > 0 ? (ayah / total) * 100 : 0);
  let phaseNote = $derived(
    playbackState.phase === 'finished'
      ? t('play.finished')
      : playbackState.phase === 'translation'
        ? t('play.phase.translation')
        : playbackState.phase === 'tafsir'
          ? t('play.phase.tafsir')
          : ''
  );
  let playLabel = $derived(
    playing
      ? t('play.pause')
      : playbackState.phase === 'finished'
        ? t('play.replay')
        : t('play.play')
  );
</script>

<footer class="player-bar">
  <div class="player-now-playing">
    <div class="player-thumb arabic">{surahArabic}</div>
    <div class="player-meta">
      <p class="player-title truncate">
        {surahName} · {t('play.ayahLabel', ayah)}
      </p>
      <p
        class="player-sub"
        class:is-error={Boolean(playbackState.errorMessage)}
        aria-live="polite"
      >
        {#if playbackState.errorMessage}
          {playbackState.errorMessage}
        {:else}
          {qariLabel} · {langLabel}{phaseNote ? ` · ${phaseNote}` : ''}
          {#if playbackState.notice}
            · {playbackState.notice}
          {/if}
        {/if}
      </p>
    </div>
  </div>

  <div class="player-center">
    <PlaybackControls
      {playing}
      {repeat}
      {showRepeat}
      onPrevious={onPrevious}
      onPlayPause={onPlayPause}
      onNext={onNext}
      onRepeat={onRepeat}
      playLabel={playLabel}
      {prevDisabled}
      {nextDisabled}
    />

    <div
      class="player-progress"
      role="progressbar"
      aria-label={t('play.progress')}
      aria-valuemin={1}
      aria-valuemax={total}
      aria-valuenow={ayah}
      aria-valuetext={t('play.progressValue', ayah, total)}
    >
      <span class="tnum">{ayah}</span>
      <div class="player-progress-track">
        <div class="player-progress-fill" style={`width: ${progressPercent}%`}></div>
      </div>
      <span class="tnum">{total}</span>
    </div>
  </div>

  <div class="player-trailing">
    {#if playbackState.errorMessage && onRetry}
      <button class="button is-quiet" type="button" onclick={onRetry}>
        {t('play.retry')}
      </button>
    {/if}
    <button
      class="icon-button"
      type="button"
      aria-label={t('play.endSession')}
      onclick={onClose}
    >
      <Icon name="close" />
    </button>
  </div>
</footer>

<style>
.player-bar {
  display: grid;
  grid-template-columns: minmax(0, 1fr) auto minmax(0, 1fr);
  align-items: center;
  gap: 24px;
  min-height: 80px;
  padding: 12px 32px;
  background: var(--surface);
  border-top: 1px solid var(--border);
  box-shadow: var(--shadow-raised);
}

.player-now-playing {
  display: flex;
  min-width: 0;
  align-items: center;
  gap: 16px;
}

.player-thumb {
  display: grid;
  width: 52px;
  height: 52px;
  flex: 0 0 auto;
  place-items: center;
  border: 1px solid var(--border);
  border-radius: var(--r-control);
  background: var(--accent);
  color: var(--on-accent);
  font-size: 20px;
}

.player-meta {
  min-width: 0;
}

.player-title {
  font-size: 15px;
  font-weight: 500;
  color: var(--text);
}

.player-sub {
  margin-top: 2px;
  font-size: 12px;
  color: var(--text-muted);
  overflow-wrap: anywhere;
}

.player-sub.is-error {
  color: var(--mark);
}

.player-center {
  display: flex;
  width: min(28rem, 100%);
  flex-direction: column;
  justify-self: center;
  gap: 8px;
}

.player-trailing {
  display: flex;
  align-items: center;
  justify-content: flex-end;
  gap: 8px;
}

.player-progress {
  display: flex;
  align-items: center;
  gap: 12px;
  font-size: 12px;
  font-variant-numeric: tabular-nums;
  color: var(--text-muted);
}

.player-progress-track {
  position: relative;
  height: 4px;
  flex: 1;
  overflow: hidden;
  border-radius: 2px;
  background: var(--track);
}

.player-progress-fill {
  position: absolute;
  inset-block: 0;
  left: 0;
  border-radius: 2px;
  background: var(--accent);
  transition: width var(--speed) var(--ease);
}
</style>
