<script lang="ts">
  import { onDestroy } from 'svelte';

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
  <span class="custom-select-label">{label}</span>
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
    <svg class:open viewBox="0 0 20 20" class="custom-select-chevron" aria-hidden="true">
      <path d="M5.25 7.75 10 12.5l4.75-4.75" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round" />
    </svg>
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
            <span class="custom-select-check">✓</span>
          {/if}
        </button>
      {/each}
    </div>
  {/if}
</div>
