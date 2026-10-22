<script lang="ts">
  import { design, setDesign, DESIGNS } from "$lib/stores/design";
</script>

<div class="design-switch" role="group" aria-label="Design">
  {#each DESIGNS as option}
    <button
      type="button"
      aria-pressed={$design === option.id}
      aria-label={option.label}
      title={option.label}
      on:click={() => setDesign(option.id)}
    >
      <span class="full">{option.label}</span>
      <span class="short">{option.short}</span>
    </button>
  {/each}
</div>

<style>
  .design-switch {
    display: inline-flex;
    align-items: center;
    height: 32px;
    padding: 2px;
    gap: 2px;
    border: 1px solid var(--line);
    border-radius: var(--radius-sm);
    background: transparent;
  }

  button {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    height: 100%;
    padding: 0 8px;
    border: 0;
    border-radius: var(--radius-xs);
    background: transparent;
    color: var(--ink-3);
    font-family: var(--font-mono);
    font-size: 11px;
    letter-spacing: 0.03em;
    line-height: 1;
    white-space: nowrap;
    cursor: pointer;
    transition:
      background-color var(--dur-fast) var(--ease),
      color var(--dur-fast) var(--ease);
  }

  button:hover {
    color: var(--ink);
    background: var(--hover-wash);
  }

  button[aria-pressed="true"] {
    background: var(--ink);
    color: var(--ink-inverse);
  }

  button[aria-pressed="true"]:hover {
    background: var(--ink);
    color: var(--ink-inverse);
  }

  .short {
    display: none;
  }

  @media (max-width: 640px) {
    .full {
      display: none;
    }
    .short {
      display: inline;
    }
    button {
      width: 28px;
      padding: 0;
    }
  }
</style>
