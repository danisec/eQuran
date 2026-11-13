<script lang="ts">
  /**
   * The verse rosette: eight-pointed star of a mushaf margin. One shape at
   * three sizes, so the reader finds the same mark on the home hero, in the
   * sidebar and on the verse that is playing. This is the product's signature.
   */
  let {
    number,
    size = 40,
    active = false
  }: {
    number: number | string;
    size?: number;
    active?: boolean;
  } = $props();

  // Octagram with 8 outer and 8 inner points, drawn once on a 64 grid.
  const points =
    '32,3 40,10 50,8 54,17 63,21 60,31 64,40 56,47 57,57 47,58 40,64 32,58 24,64 17,57 7,57 8,47 0,40 4,31 1,21 10,17 14,8 24,10';
</script>

<svg
  class="rosette {active ? 'is-active' : ''}"
  width={size}
  height={size}
  viewBox="0 0 64 64"
  role="img"
  aria-label={String(number)}
>
  <polygon points={points} />
  <text class="rosette-number" x="32" y="32" text-anchor="middle" dominant-baseline="central">{number}</text>
</svg>

<style>
  .rosette-number {
    font-family: 'IBM Plex Mono', ui-monospace, monospace;
  }
  text {
    font-size: 22px;
    font-weight: 500;
  }
  svg[width='28'] text {
    font-size: 26px;
  }
  svg[width='40'] text {
    font-size: 22px;
  }
</style>
