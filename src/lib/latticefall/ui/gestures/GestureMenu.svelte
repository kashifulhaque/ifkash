<script lang="ts">
  // The gesture menu (WP-088): wave, bow, point forward, sit, shrug, and beckon. Choosing one
  // queues `Command::Gesture(n)` through the Marks host, so it goes into the core's command list
  // like every other menu action and replays. The core decides whether the gesture can start
  // (the player must be standing free). The game pauses while the menu is open.
  import { onDestroy, tick } from 'svelte';
  import { GESTURES } from '../marks/marksData';

  export let marks: { perform(n: number): void };
  export let engine: { setPaused(paused: boolean): void };
  /** True while the menu is open. Bind to it to hold the rest of the interface still. */
  export let open = false;

  let root: HTMLElement | undefined;
  let index = 0;

  /** Opens the menu if it is closed, and closes it if it is open. */
  export function toggle(): void {
    if (open) close();
    else void show();
  }

  async function show(): Promise<void> {
    if (open) return;
    open = true;
    index = 0;
    engine.setPaused(true);
    await tick();
    focus();
  }

  function close(): void {
    if (!open) return;
    open = false;
    engine.setPaused(false);
  }

  onDestroy(() => {
    if (open) engine.setPaused(false);
  });

  function focus(): void {
    root?.querySelectorAll<HTMLElement>('button')[index]?.focus();
  }

  function choose(n: number): void {
    // The gesture goes in while the world is paused (one tick with no input), then the game resumes.
    marks.perform(n);
    close();
  }

  function onKey(e: KeyboardEvent): void {
    if (!open) return;
    if (e.code === 'Escape' || e.code === 'KeyG') {
      e.preventDefault();
      e.stopPropagation();
      close();
      return;
    }
    const digit = /^Digit([1-6])$/.exec(e.code);
    if (digit) {
      e.preventDefault();
      e.stopPropagation();
      choose(Number(digit[1]) - 1);
      return;
    }
    const step = e.code === 'ArrowDown' || e.code === 'ArrowRight' ? 1 : e.code === 'ArrowUp' || e.code === 'ArrowLeft' ? -1 : 0;
    if (step !== 0) {
      e.preventDefault();
      e.stopPropagation();
      index = (index + step + GESTURES.length) % GESTURES.length;
      focus();
    }
  }
</script>

<svelte:window on:keydown|capture={onKey} />

{#if open}
  <div class="scrim">
    <div class="panel" role="dialog" aria-modal="true" aria-labelledby="lf-gestures-title" bind:this={root}>
      <h2 id="lf-gestures-title">Gestures</h2>
      <ul>
        {#each GESTURES as g, i (g.id)}
          <li>
            <button type="button" on:click={() => choose(g.id)} on:focus={() => (index = i)}>
              <kbd>{i + 1}</kbd>
              <span class="name">{g.name}</span>
              <span class="hint">{g.hint}</span>
            </button>
          </li>
        {/each}
      </ul>
      <p class="foot"><kbd>1</kbd> to <kbd>6</kbd> choose · <kbd>Esc</kbd> close. Moving or acting ends a gesture.</p>
    </div>
  </div>
{/if}

<style>
  .scrim {
    position: absolute;
    inset: 0;
    z-index: 20;
    display: grid;
    place-items: center;
    padding: var(--s-4);
    background: var(--scrim);
    font-family: var(--font-sans);
    color: var(--ink);
  }

  .panel {
    width: min(26rem, 100%);
    padding: var(--s-4);
    background: var(--surface);
    border: 1px solid var(--line-strong);
    border-radius: var(--radius-md);
    box-shadow: 0 18px 48px var(--shadow-color);
  }

  h2 {
    margin: 0 0 var(--s-3);
    font-family: var(--font-display);
    font-size: 1.375rem;
    font-weight: 500;
  }

  ul {
    display: grid;
    gap: var(--s-2);
    margin: 0;
    padding: 0;
    list-style: none;
  }

  button {
    display: grid;
    grid-template-columns: 1.75rem 1fr;
    grid-template-areas: 'key name' '. hint';
    column-gap: var(--s-2);
    width: 100%;
    padding: var(--s-2) var(--s-3);
    border: 1px solid var(--line);
    border-radius: var(--radius-sm);
    background: var(--surface);
    color: var(--ink);
    font: inherit;
    text-align: left;
    cursor: pointer;
  }

  button:hover,
  button:focus-visible {
    border-color: var(--fjord);
    box-shadow: inset 3px 0 0 var(--fjord);
    outline: none;
  }

  kbd {
    grid-area: key;
    font-family: var(--font-mono);
    color: var(--ink-3);
  }

  .name {
    grid-area: name;
  }

  .hint {
    grid-area: hint;
    color: var(--ink-2);
    font-size: var(--text-sm);
  }

  .foot {
    margin: var(--s-3) 0 0;
    color: var(--ink-3);
    font-size: var(--text-xs);
  }

  .foot kbd {
    margin: 0 2px;
  }
</style>
