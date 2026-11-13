<script lang="ts">
  import { onDestroy } from 'svelte';
  import Icon from './Icon.svelte';

  export type SelectOption = {
    value: string;
    label: string;
    description?: string;
  };

  let {
    label,
    value,
    options,
    onChange
  }: {
    label: string;
    value: string;
    options: SelectOption[];
    onChange: (value: string) => void;
  } = $props();

  let open = $state(false);
  let root: HTMLDivElement;
  let selected = $derived(options.find((option) => option.value === value) ?? options[0]);

  function close() {
    open = false;
    document.removeEventListener('click', handleOutsideClick);
  }

  function toggle(event: MouseEvent) {
    event.stopPropagation();
    open = !open;
    if (open) {
      document.addEventListener('click', handleOutsideClick);
    } else {
      document.removeEventListener('click', handleOutsideClick);
    }
  }

  function handleOutsideClick(event: MouseEvent) {
    if (!root?.contains(event.target as Node)) {
      close();
    }
  }

  function handleKeydown(event: KeyboardEvent) {
    if (event.key === 'Escape') {
      close();
    }
  }

  function choose(nextValue: string) {
    onChange(nextValue);
    close();
  }

  onDestroy(() => {
    document.removeEventListener('click', handleOutsideClick);
  });
</script>

<svelte:window onkeydown={handleKeydown} />

<div bind:this={root} class="custom-select">
  <span class="field-label">{label}</span>
  <button
    type="button"
    class="custom-select-trigger"
    aria-haspopup="listbox"
    aria-expanded={open}
    onclick={toggle}
  >
    <span>
      <span class="custom-select-value">{selected.label}</span>
      {#if selected.description}
        <span class="custom-select-description">{selected.description}</span>
      {/if}
    </span>
    <Icon name="chevron-down" size={16} class="custom-select-chevron {open ? 'open' : ''}" />
  </button>

  {#if open}
    <div class="custom-select-menu" role="listbox" aria-label={label}>
      {#each options as option}
        <button
          type="button"
          role="option"
          aria-selected={option.value === value}
          class="custom-select-option"
          class:selected={option.value === value}
          onclick={() => choose(option.value)}
        >
          <span>
            <span class="custom-select-option-label">{option.label}</span>
            {#if option.description}
              <span class="custom-select-option-description">{option.description}</span>
            {/if}
          </span>
          {#if option.value === value}
            <span class="custom-select-check"><Icon name="check" size={16} /></span>
          {/if}
        </button>
      {/each}
    </div>
  {/if}
</div>

<style>
  .field-label {
    display: block;
    margin-bottom: 8px;
    font-size: 12px;
    font-weight: 500;
    color: var(--text-muted);
  }
  
  .custom-select {
    position: relative;
  }
  
  .custom-select-trigger {
    display: flex;
    width: 100%;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
    padding: 10px 12px;
    border: 1px solid var(--border-strong);
    border-radius: var(--r-control);
    background: var(--surface);
    color: var(--text);
    text-align: left;
    transition: border-color var(--speed-fast) var(--ease), background-color var(--speed-fast) var(--ease);
  }
  
  .custom-select-trigger:hover,
  .custom-select-trigger[aria-expanded='true'] {
    border-color: var(--accent);
    background: var(--raised);
  }
  
  .custom-select-value {
    display: block;
    font-size: 14px;
    font-weight: 500;
    line-height: 1.25;
  }
  
  .custom-select-description,
  .custom-select-option-description {
    display: block;
    margin-top: 2px;
    color: var(--text-muted);
    font-size: 12px;
  }
  
  /* The chevron is an Icon child, so its svg carries the class from outside this
     component's scope: keep the rule global rather than lose it to scoping. */
  :global(.custom-select-chevron) {
    width: 16px;
    height: 16px;
    flex: 0 0 auto;
    color: var(--text-muted);
    transition: transform var(--speed-fast) var(--ease);
  }
  
  :global(.custom-select-chevron.open) {
    transform: rotate(180deg);
  }
  
  .custom-select-menu {
    position: absolute;
    z-index: 30;
    top: calc(100% + 6px);
    left: 0;
    right: 0;
    overflow: hidden;
    border: 1px solid var(--border-strong);
    border-radius: var(--r-card);
    background: var(--surface);
    box-shadow: var(--shadow-overlay);
  }
  
  .custom-select-option {
    display: flex;
    width: 100%;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
    padding: 10px 12px;
    border: 0;
    border-bottom: 1px solid var(--border);
    background: transparent;
    color: var(--text);
    text-align: left;
    transition: background-color var(--speed-fast) var(--ease);
  }
  
  .custom-select-option:last-child {
    border-bottom: 0;
  }
  
  .custom-select-option:hover,
  .custom-select-option.selected {
    background: var(--accent-soft);
  }
  
  .custom-select-option-label {
    display: block;
    font-size: 14px;
    font-weight: 500;
    line-height: 1.25;
  }
  
  .custom-select-check {
    display: grid;
    width: 18px;
    height: 18px;
    flex: 0 0 auto;
    place-items: center;
    color: var(--accent);
  }
  
  .custom-select-check :global(svg) {
    width: 16px;
    height: 16px;
    fill: none;
    stroke: currentColor;
    stroke-width: 2;
    stroke-linecap: round;
    stroke-linejoin: round;
  }
</style>
