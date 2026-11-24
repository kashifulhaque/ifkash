<script lang="ts">
  /**
   * A red hanko (name seal). It reads "カシ", the start of Kashiful in
   * katakana. The rough edge comes from the `#japandi-hanko` filter, which
   * `Shell.svelte` defines once.
   */
  export let size = 40;
</script>

<svg
  class="seal"
  width={size}
  height={size}
  viewBox="0 0 48 48"
  aria-hidden="true"
  focusable="false"
>
  <g filter="url(#japandi-hanko)">
    <rect x="3" y="3" width="42" height="42" rx="3" fill="var(--falu)" />
    <rect
      x="7"
      y="7"
      width="34"
      height="34"
      rx="1.5"
      fill="none"
      stroke="var(--bg)"
      stroke-width="1.3"
      opacity="0.85"
    />
    <text
      x="24"
      y="23"
      text-anchor="middle"
      font-family="'Shippori Mincho', serif"
      font-weight="700"
      font-size="16"
      fill="var(--bg)">カ</text
    >
    <text
      x="24"
      y="39"
      text-anchor="middle"
      font-family="'Shippori Mincho', serif"
      font-weight="700"
      font-size="16"
      fill="var(--bg)">シ</text
    >
  </g>
</svg>

<style>
  .seal {
    display: block;
    flex-shrink: 0;
    transform: rotate(-3deg);
  }
</style>
