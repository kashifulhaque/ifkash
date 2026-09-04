<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import { dailySeedLabel, randomSeedLabel, seedLink } from '../seed';

  export let isTouch = false;
  export let found = 0;
  export let total = 7;
  export let seed = '';

  const dispatch = createEventDispatcher();

  let copyState: 'idle' | 'copied' | 'failed' = 'idle';
  let copyTimer: ReturnType<typeof setTimeout> | undefined;

  $: isDaily = seed === dailySeedLabel();
  $: link = seed ? seedLink(seed) : '';
  // Pick the "another planet" destination once, so re-renders don't re-roll it.
  const anotherLink = seedLink(randomSeedLabel(), '');

  async function copyLink() {
    clearTimeout(copyTimer);
    try {
      await navigator.clipboard.writeText(link);
      copyState = 'copied';
    } catch {
      copyState = 'failed';
    }
    copyTimer = setTimeout(() => (copyState = 'idle'), 2200);
  }
</script>

<div class="overlay" role="dialog" aria-modal="true" aria-label="How to play">
  <button class="backdrop" on:click={() => dispatch('close')} aria-label="Close"></button>
  <div class="panel">
    <p class="kicker">How to wander</p>
    <h2>A tiny planet, seven small wonders</h2>
    <p class="lead">
      Walk the planet, find the glowing markers, and open each one to read a chapter of the
      portfolio. Found so far: <strong>{found} of {total}</strong>.
    </p>

    <dl>
      {#if isTouch}
        <div><dt>Walk</dt><dd>Drag the stick in the bottom-left corner, or tap a spot to walk there.</dd></div>
        <div><dt>Look</dt><dd>Drag anywhere else to orbit the camera. Pinch to zoom.</dd></div>
        <div><dt>Hop</dt><dd>Tap the hop button.</dd></div>
        <div><dt>Open a wonder</dt><dd>Walk up to a marker and tap the card that appears.</dd></div>
        <div><dt>Animals</dt><dd>Walk up to a sheep, a fox, or a polar bear and tap the card to pet it.</dd></div>
        <div><dt>Boats</dt><dd>Stand at the shore and tap the card to launch a boat. Steer with the stick. Reach land to dock.</dd></div>
        <div><dt>Day and night</dt><dd>A full day passes every four minutes. Tap the sun or moon button to pause it. After dark, look up.</dd></div>
        <div><dt>Photo mode</dt><dd>Tap the camera button to hide the interface. Save a picture or leave from the corner buttons.</dd></div>
      {:else}
        <div><dt><kbd>W</kbd><kbd>A</kbd><kbd>S</kbd><kbd>D</kbd></dt><dd>Walk. Arrow keys work too.</dd></div>
        <div><dt><kbd>Shift</kbd></dt><dd>Run.</dd></div>
        <div><dt><kbd>Space</kbd></dt><dd>Hop.</dd></div>
        <div><dt><kbd>E</kbd></dt><dd>Open the wonder you are standing next to, or pet the animal beside you.</dd></div>
        <div><dt><kbd>M</kbd></dt><dd>Pull back to see the whole planet. Click a spot to walk there.</dd></div>
        <div><dt><kbd>T</kbd></dt><dd>Pause or resume the day. A full day passes every four minutes. After dark, look up: auroras, constellations, and the odd shooting star.</dd></div>
        <div><dt><kbd>P</kbd></dt><dd>Photo mode: hides the interface. Press again or <kbd>Esc</kbd> to leave, or save a picture from the corner.</dd></div>
        <div><dt>Mouse</dt><dd>Drag to orbit the camera, scroll to zoom, click the ground to walk.</dd></div>
        <div><dt>Boats</dt><dd>Stand at the shore and press <kbd>E</kbd> to launch a boat. Steer with the walk keys. Reach land to dock.</dd></div>
      {/if}
    </dl>

    {#if seed}
      <div class="seed">
        <div class="seed-text">
          <p class="kicker">{isDaily ? "Today's planet" : 'Planet seed'}</p>
          <p class="seed-label"><code>{seed}</code></p>
          <p class="seed-help">
            {#if copyState === 'failed'}
              Copy this link by hand: <span class="seed-url">{link}</span>
            {:else}
              Share the link to show someone this exact world. A new planet appears every day.
            {/if}
          </p>
        </div>
        <div class="seed-actions">
          <button class="chip" on:click={copyLink}>{copyState === 'copied' ? 'Copied' : 'Copy link'}</button>
          <a class="chip ghost" href={anotherLink} data-sveltekit-reload>Another planet</a>
        </div>
      </div>
    {/if}

    <div class="actions">
      <button class="continue" on:click={() => dispatch('close')}>Back to the planet</button>
      <a class="text-link" href="/">Prefer plain text? Open the site</a>
    </div>
  </div>
</div>

<style>
  .overlay {
    position: absolute;
    inset: 0;
    z-index: 30;
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 24px;
    font-family: var(--planet-sans);
    color: #f2efe6;
  }
  .backdrop {
    position: absolute;
    inset: 0;
    border: none;
    background: rgba(6, 16, 24, 0.55);
    backdrop-filter: blur(4px);
  }
  .panel {
    position: relative;
    width: min(520px, 100%);
    max-height: min(82vh, 82dvh);
    overflow-y: auto;
    padding: 26px 28px 22px;
    background: rgba(14, 30, 40, 0.94);
    border: 1px solid rgba(255, 255, 255, 0.12);
    border-radius: 18px;
    box-shadow: 0 30px 80px rgba(0, 0, 0, 0.5);
    animation: pop 0.3s cubic-bezier(0.2, 0.8, 0.2, 1);
  }
  .kicker {
    margin: 0;
    font-size: 0.62rem;
    letter-spacing: 0.2em;
    text-transform: uppercase;
    color: #e9c46a;
  }
  h2 {
    margin: 8px 0 10px;
    font-family: var(--planet-serif);
    font-weight: 400;
    font-size: 1.8rem;
    line-height: 1.1;
    color: #fbf8f0;
  }
  .lead {
    margin: 0 0 18px;
    font-size: 0.86rem;
    line-height: 1.55;
    color: rgba(242, 239, 230, 0.72);
  }
  .lead strong {
    color: #e9c46a;
    font-weight: 600;
  }
  dl {
    margin: 0;
    display: grid;
    gap: 10px;
  }
  dl div {
    display: grid;
    grid-template-columns: 120px 1fr;
    gap: 12px;
    align-items: baseline;
    font-size: 0.82rem;
  }
  dt {
    display: flex;
    gap: 4px;
    flex-wrap: wrap;
    color: rgba(242, 239, 230, 0.85);
    font-weight: 500;
  }
  dd {
    margin: 0;
    color: rgba(242, 239, 230, 0.66);
    line-height: 1.5;
  }
  kbd {
    font-family: var(--planet-sans);
    font-size: 0.62rem;
    line-height: 1;
    padding: 4px 6px;
    min-width: 20px;
    text-align: center;
    border: 1px solid rgba(255, 255, 255, 0.22);
    border-radius: 4px;
    background: rgba(255, 255, 255, 0.04);
  }
  .seed {
    margin-top: 20px;
    padding: 14px 16px;
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 14px;
    flex-wrap: wrap;
    border: 1px solid rgba(255, 255, 255, 0.1);
    border-radius: 12px;
    background: rgba(255, 255, 255, 0.03);
  }
  .seed-text {
    min-width: 0;
    flex: 1 1 200px;
  }
  .seed-label {
    margin: 4px 0 4px;
  }
  .seed-label code {
    font-family: var(--font-mono, ui-monospace, monospace);
    font-size: 0.9rem;
    color: #fbf8f0;
    letter-spacing: 0.02em;
  }
  .seed-help {
    margin: 0;
    font-size: 0.72rem;
    line-height: 1.5;
    color: rgba(242, 239, 230, 0.55);
  }
  .seed-url {
    word-break: break-all;
    color: rgba(242, 239, 230, 0.85);
  }
  .seed-actions {
    display: flex;
    gap: 8px;
    flex-wrap: wrap;
  }
  .chip {
    padding: 7px 13px;
    border-radius: 999px;
    border: 1px solid rgba(233, 196, 106, 0.6);
    background: rgba(233, 196, 106, 0.12);
    color: #fbf1d3;
    font-size: 0.72rem;
    font-weight: 500;
    text-decoration: none;
    white-space: nowrap;
  }
  .chip:hover {
    background: rgba(233, 196, 106, 0.22);
  }
  .chip.ghost {
    border-color: rgba(255, 255, 255, 0.18);
    background: rgba(255, 255, 255, 0.04);
    color: rgba(242, 239, 230, 0.8);
  }
  .chip.ghost:hover {
    background: rgba(255, 255, 255, 0.1);
  }
  .actions {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
    margin-top: 22px;
    flex-wrap: wrap;
  }
  .continue {
    padding: 10px 18px;
    border-radius: 999px;
    border: 1px solid rgba(233, 196, 106, 0.6);
    background: rgba(233, 196, 106, 0.12);
    color: #fbf1d3;
    font-size: 0.8rem;
    font-weight: 500;
  }
  .continue:hover {
    background: rgba(233, 196, 106, 0.22);
  }
  .text-link {
    font-size: 0.72rem;
    color: rgba(242, 239, 230, 0.55);
    text-decoration: underline;
    text-underline-offset: 3px;
  }
  .text-link:hover {
    color: rgba(242, 239, 230, 0.9);
  }
  @keyframes pop {
    from {
      opacity: 0;
      transform: translateY(14px) scale(0.97);
    }
    to {
      opacity: 1;
      transform: none;
    }
  }
  @media (max-width: 600px) {
    dl div {
      grid-template-columns: 1fr;
      gap: 4px;
    }
  }
</style>
