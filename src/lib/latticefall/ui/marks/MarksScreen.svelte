<script lang="ts">
  // The Marks list (WP-088): every Mark, what it asks, and whether it is earned. Read-only; the
  // core decides what is earned (`lf-core` `world_state::marks`). Shown as a screen of the pause
  // menu. Earned Marks are also kept locally by the codex store, and in the cloud when signed in.
  import type { MarksHost } from './marksHost';

  export let marks: Pick<MarksHost, 'list' | 'earned'>;
  export let announce: (text: string) => void = () => {};

  const earned = marks.earned;
  const list = marks.list();

  $: total = list.length;
  $: count = $earned.size;
  $: announce(`${count} of ${total} Marks earned`);

  /** Nothing to step back out of. */
  export function back(): boolean {
    return false;
  }
</script>

<div class="marks">
  <p class="summary">
    <strong>{count}</strong> of {total} earned
  </p>
  <ul aria-label="Marks">
    {#each list as m (m.slug)}
      {@const got = $earned.has(m.slug)}
      <li class="mark" class:earned={got} tabindex="0" data-nav="body" aria-label={`${m.name}: ${got ? 'earned' : 'not yet earned'}. ${m.how}`}>
        <span class="seal" aria-hidden="true">{got ? '✓' : ''}</span>
        <span class="text">
          <span class="name">{m.name}</span>
          <span class="how">{m.how}</span>
        </span>
        <span class="state">{got ? 'Earned' : 'Not yet'}</span>
      </li>
    {/each}
  </ul>
</div>

<style>
  .marks {
    display: grid;
    gap: 0.75em;
  }

  .summary {
    margin: 0;
    color: var(--ink-2);
    font-size: 0.875em;
  }

  ul {
    display: grid;
    gap: 0.4em;
    margin: 0;
    padding: 0;
    list-style: none;
  }

  .mark {
    display: grid;
    grid-template-columns: 1.75em 1fr auto;
    align-items: center;
    gap: 0.75em;
    padding: 0.55em 0.75em;
    border: 1px solid var(--line);
    border-radius: var(--radius-sm);
    background: var(--surface);
    color: var(--ink-3);
  }

  .mark:focus-visible {
    outline: 2px solid var(--fjord);
    outline-offset: 1px;
  }

  .mark.earned {
    border-color: var(--moss);
    color: var(--ink);
  }

  .seal {
    display: grid;
    place-items: center;
    width: 1.5em;
    height: 1.5em;
    border: 1px solid var(--line-strong);
    border-radius: 50%;
    font-size: 0.875em;
  }

  .earned .seal {
    border-color: var(--moss);
    background: var(--moss);
    color: var(--ink-inverse);
  }

  .text {
    display: grid;
    gap: 0.1em;
  }

  .name {
    font-family: var(--font-display);
    font-size: 1.0625em;
  }

  .how {
    font-size: 0.8125em;
    color: var(--ink-2);
  }

  .state {
    font-family: var(--font-mono);
    font-size: 0.6875em;
    letter-spacing: 0.06em;
    text-transform: uppercase;
  }
</style>
