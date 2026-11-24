<script lang="ts">
  /** The moon as a small engraved disc: an outline, a lit shape, and hatching on the dark side. */
  import { moonPath } from "./astro";

  export let fraction = 0;
  export let size = 22;

  $: shape = moonPath(fraction, 9);
</script>

<svg
  class="moon"
  width={size}
  height={size}
  viewBox="-11 -11 22 22"
  aria-hidden="true"
  focusable="false"
>
  <defs>
    <clipPath id="moon-disc"><circle r="9" /></clipPath>
  </defs>
  <g clip-path="url(#moon-disc)" stroke="currentColor" stroke-width="0.6" opacity="0.5">
    {#each [-8, -5.5, -3, -0.5, 2, 4.5, 7] as y}
      <path d={`M-10 ${y + 10}L10 ${y - 10}`} />
    {/each}
  </g>
  <path
    d={shape.d}
    fill="var(--bg)"
    stroke="currentColor"
    stroke-width="0.8"
    transform={shape.mirror ? "scale(-1 1)" : undefined}
  />
  <circle r="9" fill="none" stroke="currentColor" stroke-width="0.9" />
</svg>

<style>
  .moon {
    display: block;
    flex: none;
  }
</style>
