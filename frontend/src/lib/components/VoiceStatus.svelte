<script lang="ts">
  import Icon from '$lib/components/Icon.svelte';
  import type { IconName } from '$lib/components/Icon.svelte';
  import { t } from '$lib/i18n';
  import { playbackState } from '$lib/stores/playback.svelte';

  let isReady = $derived(playbackState.ttsVoiceStatus === 'ready');

  let stateClass = $derived(
    playbackState.ttsVoiceStatus === 'ready'
      ? 'is-ready'
      : playbackState.ttsVoiceStatus === 'checking'
        ? ''
        : 'is-failed'
  );

  let icon = $derived.by((): IconName => {
    if (playbackState.ttsVoiceStatus === 'ready') return 'check';
    if (playbackState.ttsVoiceStatus === 'checking') return 'loader';
    return 'alert';
  });

  let copy = $derived(
    playbackState.ttsVoiceStatus === 'ready'
      ? t('settings.voiceReady')
      : playbackState.ttsVoiceStatus === 'checking'
        ? t('settings.voiceChecking')
        // The backend names the real cause (interpreter version, missing files);
        // the generic hint is only a fallback when it has nothing to say.
        : playbackState.ttsVoiceMessage.trim() || t('settings.voiceFailedHint')
  );

  /** The voice row only earns space when it says something actionable. */
  let visible = $derived(!isReady);
</script>

{#if visible}
  <div class="voice-status {stateClass}">
    <span class="voice-status-icon">
      <Icon name={icon} class={playbackState.ttsVoiceStatus === 'checking' ? 'animate-spin' : ''} />
    </span>
    <div>
      <h3 class="voice-status-title">{t('settings.voice')}</h3>
      <p class="voice-status-copy">{copy}</p>
    </div>
  </div>
{/if}

<style>
  /* Status notice: a quiet surface, not an amber warning card */
  .voice-status {
    display: flex;
    align-items: flex-start;
    gap: 12px;
    padding: 12px;
    border: 1px solid var(--border);
    border-radius: var(--r-control);
    background: var(--surface);
  }
  
  .voice-status-icon {
    display: grid;
    width: 26px;
    height: 26px;
    flex: 0 0 auto;
    place-items: center;
    color: var(--text-muted);
  }
  
  .voice-status-icon :global(svg) {
    width: 18px;
    height: 18px;
    fill: none;
    stroke: currentColor;
    stroke-width: 1.5;
    stroke-linecap: round;
    stroke-linejoin: round;
  }

  .voice-status.is-ready .voice-status-icon {
    color: var(--accent);
  }
  
  .voice-status.is-failed .voice-status-icon {
    color: var(--mark);
  }
  
  .voice-status-title {
    font-size: 14px;
    font-weight: 500;
    color: var(--text);
  }
  
  .voice-status-copy {
    margin-top: 3px;
    font-size: 12px;
    line-height: 1.45;
    color: var(--text-muted);
  }
</style>
