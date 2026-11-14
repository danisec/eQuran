<script lang="ts">
  import Icon from '$lib/components/Icon.svelte';
  import type { IconName } from '$lib/components/Icon.svelte';
  import { t } from '$lib/i18n';
  import { playbackState, setupTtsRuntime } from '$lib/stores/playback.svelte';

  /** Transient "copied" acknowledgement for the manual-command fallback. */
  let copied = $state(false);
  let copyTimer: ReturnType<typeof setTimeout> | null = null;

  let isReady = $derived(playbackState.ttsVoiceStatus === 'ready');

  // An error from the last setup attempt, so a failure stays visible after the
  // log collapses instead of silently resetting to the generic notice.
  let setupError = $state('');
  // The tail of the failed run's output. The live log disappears when the run
  // ends, but a failure is only diagnosable if the reason stays on screen.
  let failedLog = $state<string[]>([]);

  let stateClass = $derived(
    playbackState.ttsVoiceStatus === 'ready'
      ? 'is-ready'
      : playbackState.ttsVoiceStatus === 'checking'
        ? ''
        : 'is-failed'
  );

  let icon = $derived.by((): IconName => {
    if (playbackState.ttsVoiceStatus === 'ready') return 'check';
    if (playbackState.ttsVoiceStatus === 'checking' || playbackState.ttsSettingUp) return 'loader';
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

  /** The last few log lines, so a long pip run does not grow without bound. */
  let logTail = $derived(playbackState.ttsSetupLog.slice(-8));
  let canRunSetup = $derived(playbackState.ttsCanSetup && !playbackState.ttsSettingUp);
  let showManual = $derived(!playbackState.ttsSettingUp && playbackState.ttsSetupCommand.trim() !== '');

  async function onSetup() {
    setupError = '';
    failedLog = [];
    copied = false;
    const error = await setupTtsRuntime();
    // The backend detail and the notice copy are often the same sentence; the
    // error line only earns its space when it says something new.
    if (error && error !== copy) {
      setupError = error;
      failedLog = playbackState.ttsSetupLog.slice(-8);
    }
  }

  async function onCopy() {
    if (!navigator.clipboard) return;
    try {
      await navigator.clipboard.writeText(playbackState.ttsSetupCommand);
      copied = true;
      if (copyTimer) clearTimeout(copyTimer);
      copyTimer = setTimeout(() => (copied = false), 1500);
    } catch {
      copied = false;
    }
  }
</script>

{#if visible}
  <div class="voice-status {stateClass}">
    <span class="voice-status-icon">
      <Icon
        name={icon}
        class={playbackState.ttsVoiceStatus === 'checking' || playbackState.ttsSettingUp
          ? 'animate-spin'
          : ''}
      />
    </span>
    <div class="voice-status-body">
      <h3 class="voice-status-title">{t('settings.voice')}</h3>
      <p class="voice-status-copy">{copy}</p>

      {#if playbackState.ttsSettingUp}
        <p class="voice-status-setup-hint">{t('settings.voiceSetupHint')}</p>
        {#if logTail.length > 0}
          <pre class="voice-status-log">{logTail.join('\n')}</pre>
        {/if}
      {:else}
        {#if canRunSetup}
          <button type="button" class="voice-status-action" onclick={onSetup}>
            <Icon name="arrow-right" size={15} />
            {playbackState.ttsVoiceStatus === 'failed'
              ? t('settings.voiceSetupAgain')
              : t('settings.voiceSetup')}
          </button>
        {/if}

        {#if setupError}
          <p class="voice-status-error">{setupError}</p>
          {#if failedLog.length > 0}
            <pre class="voice-status-log">{failedLog.join('\n')}</pre>
          {/if}
        {/if}

        {#if showManual}
          <details class="voice-status-manual">
            <summary>{t('settings.voiceManualHint')}</summary>
            <div class="voice-status-command">
              <code>{playbackState.ttsSetupCommand}</code>
              <button type="button" class="voice-status-copy" onclick={onCopy}>
                <Icon name={copied ? 'check' : 'copy'} size={14} />
                {copied ? t('settings.voiceCopied') : t('settings.voiceCopyCommand')}
              </button>
            </div>
          </details>
        {/if}
      {/if}
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

  .voice-status-body {
    flex: 1 1 auto;
    min-width: 0;
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

  .voice-status-setup-hint {
    margin-top: 8px;
    font-size: 12px;
    line-height: 1.45;
    color: var(--text-muted);
  }

  .voice-status-log {
    margin-top: 8px;
    max-height: 132px;
    overflow: auto;
    padding: 8px 10px;
    border-radius: var(--r-control);
    background: var(--raised);
    border: 1px solid var(--border);
    font-family: 'IBM Plex Mono', ui-monospace, monospace;
    font-size: 11px;
    line-height: 1.5;
    color: var(--text-muted);
    white-space: pre-wrap;
    word-break: break-word;
  }

  .voice-status-action {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    margin-top: 10px;
    padding: 7px 12px;
    border: 1px solid var(--border);
    border-radius: var(--r-control);
    background: var(--surface);
    color: var(--text);
    font-size: 12px;
    font-weight: 500;
    cursor: pointer;
    transition: border-color var(--speed-fast) var(--ease), background-color var(--speed-fast) var(--ease);
  }

  .voice-status-action:hover {
    border-color: var(--border-strong);
    background: var(--raised);
  }

  .voice-status-error {
    margin-top: 8px;
    font-size: 12px;
    line-height: 1.45;
    color: var(--mark);
  }

  .voice-status-manual {
    margin-top: 10px;
  }

  .voice-status-manual summary {
    font-size: 12px;
    color: var(--text-muted);
    cursor: pointer;
  }

  .voice-status-command {
    display: flex;
    align-items: flex-start;
    gap: 8px;
    margin-top: 6px;
  }

  .voice-status-command code {
    flex: 1 1 auto;
    min-width: 0;
    padding: 6px 8px;
    border-radius: var(--r-control);
    border: 1px solid var(--border);
    background: var(--raised);
    font-family: 'IBM Plex Mono', ui-monospace, monospace;
    font-size: 11px;
    line-height: 1.5;
    color: var(--text-muted);
    white-space: pre-wrap;
    word-break: break-all;
  }

  .voice-status-copy {
    display: inline-flex;
    align-items: center;
    gap: 5px;
    flex: 0 0 auto;
    padding: 6px 9px;
    border: 1px solid var(--border);
    border-radius: var(--r-control);
    background: var(--surface);
    color: var(--text);
    font-size: 11px;
    cursor: pointer;
    transition: border-color var(--speed-fast) var(--ease), background-color var(--speed-fast) var(--ease);
  }

  .voice-status-copy:hover {
    border-color: var(--border-strong);
    background: var(--raised);
  }
</style>
