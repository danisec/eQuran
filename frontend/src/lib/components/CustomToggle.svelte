<script lang="ts">
  let {
    label,
    description,
    checked,
    disabled = false,
    onChange
  }: {
    label: string;
    description?: string;
    checked: boolean;
    disabled?: boolean;
    onChange: (checked: boolean) => void;
  } = $props();

  function toggle() {
    if (disabled) return;
    onChange(!checked);
  }
</script>

<button
  type="button"
  class="custom-toggle"
  class:checked
  class:disabled
  role="switch"
  aria-checked={checked}
  aria-disabled={disabled}
  aria-label={label}
  onclick={toggle}
>
  <span class="custom-toggle-copy">
    <span class="custom-toggle-label">{label}</span>
    {#if description}
      <span class="custom-toggle-description">{description}</span>
    {/if}
  </span>
  <span class="custom-toggle-track" aria-hidden="true">
    <span class="custom-toggle-knob">
      {#if checked}
        <svg viewBox="0 0 16 16" class="custom-toggle-check">
          <path d="M3.25 8.1 6.4 11.25 12.9 4.75" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" />
        </svg>
      {/if}
    </span>
  </span>
</button>

<style>
  .custom-toggle {
    display: flex;
    align-items: flex-start;
    justify-content: space-between;
    gap: 16px;
    padding: 12px;
    border: 1px solid var(--border);
    border-radius: var(--r-control);
    background: var(--surface);
    text-align: left;
    transition: border-color var(--speed-fast) var(--ease), background-color var(--speed-fast) var(--ease);
  }
  
  .custom-toggle:hover {
    border-color: var(--border-strong);
    background: var(--raised);
  }
  
  .custom-toggle.disabled {
    opacity: 0.55;
  }
  
  .custom-toggle-copy {
    min-width: 0;
  }
  
  .custom-toggle-label {
    display: block;
    font-size: 14px;
    font-weight: 500;
    color: var(--text);
    line-height: 1.3;
  }
  
  .custom-toggle-description {
    display: block;
    margin-top: 3px;
    font-size: 12px;
    color: var(--text-muted);
    line-height: 1.4;
  }
  
  .custom-toggle-track {
    position: relative;
    width: 38px;
    height: 22px;
    flex: 0 0 auto;
    margin-top: 1px;
    border-radius: 11px;
    background: var(--track);
    transition: background-color var(--speed) var(--ease);
  }
  
  .custom-toggle-knob {
    position: absolute;
    top: 3px;
    left: 3px;
    width: 16px;
    height: 16px;
    border-radius: 50%;
    background: var(--knob);
    box-shadow: var(--shadow-card);
    transition: transform var(--speed) var(--ease);
  }
  
  .custom-toggle.checked .custom-toggle-track {
    background: var(--accent);
  }
  
  /* On the accent, the knob takes the accent's own ink. Light accent is dark, so
     the knob stays pale; dark accent is a light fill, so the knob goes dark and
     the switch reads like a physical toggle instead of a white dot on white. */
  .custom-toggle.checked .custom-toggle-knob {
    transform: translateX(16px);
    background: var(--on-accent);
  }
  
  .custom-toggle-check {
    display: none;
  }
</style>
