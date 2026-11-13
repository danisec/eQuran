<script lang="ts">
  import Icon from '$lib/components/Icon.svelte';
  import { t } from '$lib/i18n';

  type Props = {
    open: boolean;
    title: string;
    subtitle?: string;
    onClose: () => void;
    /** id used by `aria-labelledby`; must be unique per page. */
    labelledBy: string;
    children: import('svelte').Snippet;
  };

  let { open, title, subtitle, onClose, labelledBy, children }: Props = $props();

  let dialog = $state<HTMLDivElement | undefined>(undefined);
  let closeButton = $state<HTMLButtonElement | undefined>(undefined);

  /** Moves focus into the dialog when it opens, so Escape and Tab land here. */
  $effect(() => {
    if (open) {
      closeButton?.focus();
    }
  });

  /** Keeps Tab and Shift+Tab inside the dialog while it is open. */
  function trapFocus(event: KeyboardEvent) {
    if (event.key !== 'Tab' || !dialog) return;
    const focusable = dialog.querySelectorAll<HTMLElement>(
      'button:not([disabled]), input:not([disabled]), select, textarea, [href], [tabindex]:not([tabindex="-1"])'
    );
    if (focusable.length === 0) return;
    const first = focusable[0];
    const last = focusable[focusable.length - 1];
    if (event.shiftKey && document.activeElement === first) {
      event.preventDefault();
      last.focus();
    } else if (!event.shiftKey && document.activeElement === last) {
      event.preventDefault();
      first.focus();
    }
  }
</script>

<svelte:window
  onkeydown={(event) => {
    if (open && event.key === 'Escape') {
      event.preventDefault();
      onClose();
    }
  }}
/>

{#if open}
  <!-- The backdrop is a button so a pointer click and the keyboard both close it -->
  <div class="settings-layer">
    <button
      class="settings-backdrop"
      type="button"
      aria-label={t('settings.close')}
      tabindex="-1"
      onclick={onClose}
    ></button>

    <div
      bind:this={dialog}
      class="settings-dialog"
      role="dialog"
      aria-modal="true"
      aria-labelledby={labelledBy}
      tabindex="-1"
      onkeydown={trapFocus}
    >
      <header class="mb-5 flex items-start justify-between gap-4">
        <div>
          <h2 id={labelledBy} class="text-[18px] font-medium text-[var(--text)]">{title}</h2>
          {#if subtitle}
            <p class="meta mt-1">{subtitle}</p>
          {/if}
        </div>
        <button
          bind:this={closeButton}
          class="icon-button"
          type="button"
          aria-label={t('settings.close')}
          onclick={onClose}
        >
          <Icon name="close" />
        </button>
      </header>

      {@render children()}
    </div>
  </div>
{/if}

<style>
.settings-layer {
  position: fixed;
  inset: 0;
  z-index: 50;
  display: grid;
  place-items: center;
  padding: 24px;
}

.settings-backdrop {
  position: absolute;
  inset: 0;
  border: 0;
  background: rgba(10, 18, 27, 0.56);
  cursor: default;
}

.settings-dialog {
  position: relative;
  width: min(520px, 100%);
  max-height: min(760px, 100%);
  overflow-y: auto;
  border: 1px solid var(--border);
  border-radius: var(--r-panel);
  background: var(--surface);
  padding: 24px;
  box-shadow: var(--shadow-overlay);
}
</style>
