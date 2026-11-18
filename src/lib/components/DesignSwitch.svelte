<script lang="ts">
  import { design, setDesign, DESIGNS, isDesign } from "$lib/stores/design";

  function onChange(event: Event) {
    const value = (event.currentTarget as HTMLSelectElement).value;
    if (isDesign(value)) setDesign(value);
  }
</script>

<label class="design-switch">
  <span class="sr-only">Design</span>
  <select value={$design} on:change={onChange} title="Design">
    {#each DESIGNS as option}
      <option value={option.id}>{option.label}</option>
    {/each}
  </select>
  <svg class="chevron" viewBox="0 0 12 12" aria-hidden="true">
    <path d="M3 4.5 6 7.5 9 4.5" fill="none" stroke="currentColor" stroke-width="1.25" />
  </svg>
</label>

<style>
  .design-switch {
    position: relative;
    display: inline-flex;
    align-items: center;
    height: 32px;
  }

  select {
    appearance: none;
    -webkit-appearance: none;
    height: 100%;
    padding: 0 26px 0 10px;
    border: 1px solid var(--line);
    border-radius: var(--radius-sm);
    background: transparent;
    color: var(--ink-2);
    font-family: var(--font-mono);
    font-size: 11px;
    letter-spacing: 0.03em;
    line-height: 1;
    cursor: pointer;
    transition:
      border-color var(--dur-fast) var(--ease),
      color var(--dur-fast) var(--ease),
      background-color var(--dur-fast) var(--ease);
  }

  select:hover {
    color: var(--ink);
    border-color: var(--line-strong);
    background: var(--hover-wash);
  }

  select:focus-visible {
    outline: 1.5px solid var(--ink);
    outline-offset: 2px;
  }

  /* The open list uses the page colors instead of the OS defaults. */
  option {
    background: var(--surface);
    color: var(--ink);
  }

  .chevron {
    position: absolute;
    right: 8px;
    width: 12px;
    height: 12px;
    color: var(--ink-3);
    pointer-events: none;
  }
</style>
