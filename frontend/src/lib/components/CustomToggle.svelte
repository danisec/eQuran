<script lang="ts">
  let {
    label,
    description,
    checked,
    onChange
  }: {
    label: string;
    description?: string;
    checked: boolean;
    onChange: (checked: boolean) => void;
  } = $props();

  function toggle() {
    onChange(!checked);
  }
</script>

<button
  type="button"
  class="custom-toggle"
  class:checked
  role="switch"
  aria-checked={checked}
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
