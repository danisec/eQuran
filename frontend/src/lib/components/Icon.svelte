<script lang="ts">
  /**
   * One icon set, one stroke, one grid. Every glyph in the interface comes from
   * here so nothing drifts back to emoji or a second stroke weight.
   */
  export type IconName =
    | 'home'
    | 'book'
    | 'bookmark'
    | 'bookmark-filled'
    | 'search'
    | 'play'
    | 'pause'
    | 'stop'
    | 'previous'
    | 'next'
    | 'repeat'
    | 'copy'
    | 'check'
    | 'close'
    | 'chevron-down'
    | 'chevron-left'
    | 'chevron-right'
    | 'maximize'
    | 'sun'
    | 'moon'
    | 'settings'
    | 'volume'
    | 'alert'
    | 'loader'
    | 'arrow-right';

  let {
    name,
    size = 18,
    stroke = 1.5,
    filled = false,
    class: className = ''
  }: {
    name: IconName;
    size?: number;
    stroke?: number;
    filled?: boolean;
    class?: string;
  } = $props();
</script>

<svg
  class={className}
  width={size}
  height={size}
  viewBox="0 0 24 24"
  aria-hidden="true"
  fill={filled ? 'currentColor' : 'none'}
  stroke={filled ? 'none' : 'currentColor'}
  stroke-width={stroke}
  stroke-linecap="round"
  stroke-linejoin="round"
>
  {#if name === 'home'}
    <path d="M4 10.5 12 4l8 6.5V19a1 1 0 0 1-1 1h-4v-6h-6v6H5a1 1 0 0 1-1-1z" />
  {:else if name === 'book'}
    <path d="M5 4.5A1.5 1.5 0 0 1 6.5 3H18a1 1 0 0 1 1 1v14a1 1 0 0 1-1 1H6.5A1.5 1.5 0 0 0 5 20.5z" />
    <path d="M5 18.5A1.5 1.5 0 0 1 6.5 17H19" />
  {:else if name === 'bookmark'}
    <path d="M7 4.75A1.75 1.75 0 0 1 8.75 3h6.5A1.75 1.75 0 0 1 17 4.75V20l-5-3.2L7 20z" />
  {:else if name === 'bookmark-filled'}
    <path d="M7 4.75A1.75 1.75 0 0 1 8.75 3h6.5A1.75 1.75 0 0 1 17 4.75V20l-5-3.2L7 20z" />
  {:else if name === 'search'}
    <circle cx="11" cy="11" r="6.5" />
    <path d="m16 16 4 4" />
  {:else if name === 'play'}
    <path d="M8 5.2v13.6l11-6.8z" />
  {:else if name === 'pause'}
    <rect x="7" y="5" width="3.5" height="14" rx="1" />
    <rect x="13.5" y="5" width="3.5" height="14" rx="1" />
  {:else if name === 'stop'}
    <rect x="6" y="6" width="12" height="12" rx="1.5" />
  {:else if name === 'previous'}
    <path d="M7 5v14" />
    <path d="M18 5.6v12.8L9.5 12z" />
  {:else if name === 'next'}
    <path d="M17 5v14" />
    <path d="M6 5.6v12.8L14.5 12z" />
  {:else if name === 'repeat'}
    <path d="M6.5 8.5h10A2.5 2.5 0 0 1 19 11v1" />
    <path d="m14 6 2.5 2.5L14 11" />
    <path d="M17.5 15.5h-10A2.5 2.5 0 0 1 5 13v-1" />
    <path d="m10 18-2.5-2.5L10 13" />
  {:else if name === 'copy'}
    <rect x="9" y="9" width="11" height="11" rx="2" />
    <path d="M15 9V6a2 2 0 0 0-2-2H6a2 2 0 0 0-2 2v7a2 2 0 0 0 2 2h3" />
  {:else if name === 'check'}
    <path d="m5 12.5 4.5 4.5L19 7" />
  {:else if name === 'close'}
    <path d="m6 6 12 12M18 6 6 18" />
  {:else if name === 'chevron-down'}
    <path d="m6 9.5 6 6 6-6" />
  {:else if name === 'chevron-left'}
    <path d="m14 6-6 6 6 6" />
  {:else if name === 'chevron-right'}
    <path d="m10 6 6 6-6 6" />
  {:else if name === 'maximize'}
    <path d="M4 9V5a1 1 0 0 1 1-1h4M20 9V5a1 1 0 0 0-1-1h-4M4 15v4a1 1 0 0 0 1 1h4M20 15v4a1 1 0 0 1-1 1h-4" />
  {:else if name === 'sun'}
    <circle cx="12" cy="12" r="4" />
    <path d="M12 3v2M12 19v2M3 12h2M19 12h2M5.6 5.6l1.4 1.4M17 17l1.4 1.4M18.4 5.6 17 7M7 17l-1.4 1.4" />
  {:else if name === 'moon'}
    <path d="M20 15.6A8.5 8.5 0 1 1 8.4 4a6.8 6.8 0 0 0 11.6 11.6z" />
  {:else if name === 'settings'}
    <circle cx="12" cy="12" r="3" />
    <path d="M19.4 15a1.7 1.7 0 0 0 .3 1.8l.1.1a2 2 0 1 1-2.8 2.8l-.1-.1a1.7 1.7 0 0 0-1.8-.3 1.7 1.7 0 0 0-1 1.5v.2a2 2 0 1 1-4 0v-.1a1.7 1.7 0 0 0-1.1-1.5 1.7 1.7 0 0 0-1.8.3l-.1.1a2 2 0 1 1-2.8-2.8l.1-.1a1.7 1.7 0 0 0 .3-1.8 1.7 1.7 0 0 0-1.5-1H3a2 2 0 1 1 0-4h.1a1.7 1.7 0 0 0 1.5-1.1 1.7 1.7 0 0 0-.3-1.8l-.1-.1a2 2 0 1 1 2.8-2.8l.1.1a1.7 1.7 0 0 0 1.8.3H9a1.7 1.7 0 0 0 1-1.5V3a2 2 0 1 1 4 0v.1a1.7 1.7 0 0 0 1 1.5 1.7 1.7 0 0 0 1.8-.3l.1-.1a2 2 0 1 1 2.8 2.8l-.1.1a1.7 1.7 0 0 0-.3 1.8V9a1.7 1.7 0 0 0 1.5 1H21a2 2 0 1 1 0 4h-.1a1.7 1.7 0 0 0-1.5 1z" />
  {:else if name === 'volume'}
    <path d="M11 4.7a.7.7 0 0 0-1.2-.5L6.4 7.6a1.4 1.4 0 0 1-1 .4H3.5a1 1 0 0 0-1 1v6a1 1 0 0 0 1 1H5.4a1.4 1.4 0 0 1 1 .4l3.4 3.4a.7.7 0 0 0 1.2-.5z" />
    <path d="M15.5 9.5a4 4 0 0 1 0 5" />
    <path d="M18.5 6.5a8 8 0 0 1 0 11" />
  {:else if name === 'alert'}
    <circle cx="12" cy="12" r="8.5" />
    <path d="M12 8v4.5M12 15.8v.4" />
  {:else if name === 'loader'}
    <path d="M12 3.5v3M12 17.5v3M3.5 12h3M17.5 12h3M6 6l2 2M16 16l2 2M18 6l-2 2M8 16l-2 2" />
  {:else if name === 'arrow-right'}
    <path d="M4 12h15" />
    <path d="m13 6 6 6-6 6" />
  {/if}
</svg>

<style>
  svg {
    flex: 0 0 auto;
    display: block;
  }
</style>
