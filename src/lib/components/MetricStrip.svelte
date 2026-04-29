<script lang="ts">
  type Metric = { value: string; label: string; suffix?: string };
  export let metrics: Metric[] = [];
</script>

<div class="strip">
  {#each metrics as m, i}
    <div class="cell stagger" style="--i: {i}">
      <div class="value">
        {m.value}
        {#if m.suffix}<span class="suffix">{m.suffix}</span>{/if}
      </div>
      <div class="label">{m.label}</div>
    </div>
  {/each}
</div>

<style>
  .strip {
    display: grid;
    grid-template-columns: repeat(2, 1fr);
    gap: 0;
    border: 1px solid var(--border);
    border-radius: var(--radius-lg);
    background: var(--surface-glass);
    backdrop-filter: blur(14px);
    -webkit-backdrop-filter: blur(14px);
    overflow: hidden;
  }

  @media (min-width: 720px) {
    .strip {
      grid-template-columns: repeat(4, 1fr);
    }
  }

  .cell {
    padding: 1.5rem 1.25rem;
    border-right: 1px solid var(--border-subtle);
    border-bottom: 1px solid var(--border-subtle);
    display: flex;
    flex-direction: column;
    gap: 0.35rem;
  }

  @media (min-width: 720px) {
    .cell {
      border-bottom: none;
    }
    .cell:last-child {
      border-right: none;
    }
  }

  @media (max-width: 719px) {
    .cell:nth-child(2n) {
      border-right: none;
    }
    .cell:nth-last-child(-n+2) {
      border-bottom: none;
    }
  }

  .value {
    font-family: var(--font-sans);
    font-size: clamp(1.75rem, 3vw + 0.5rem, 2.5rem);
    font-weight: 400;
    letter-spacing: -0.04em;
    color: var(--text-primary);
    line-height: 1;
    font-variant-numeric: tabular-nums;
  }

  .suffix {
    font-size: 0.65em;
    color: var(--accent);
    font-weight: 500;
    margin-left: 0.05em;
  }

  .label {
    font-family: var(--font-mono);
    font-size: 0.6875rem;
    text-transform: uppercase;
    letter-spacing: 0.16em;
    color: var(--text-tertiary);
  }
</style>
