<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import { seedLink } from '../seed';

  export let isTouch = false;
  export let found = 0;
  export let total = 7;
  export let seed = '';
  export let depth = 0;

  const dispatch = createEventDispatcher();

  let copyState: 'idle' | 'copied' | 'failed' = 'idle';
  let copyTimer: ReturnType<typeof setTimeout> | undefined;

  $: isEarth = seed === 'earth';
  $: link = seed ? seedLink(seed, undefined, depth) : '';

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
    <h2>Seven wonders, one way into deep space</h2>
    <p class="lead">
      Explore the planet and open its glowing wonders. Found so far:
      <strong>{found} of {total}</strong>.
    </p>

    <section class="journey" aria-labelledby="journey-title">
      <p class="kicker" id="journey-title">Your journey</p>
      <ol>
        <li><strong>Recover five ship parts</strong><span>Search the planet for the missing components.</span></li>
        <li><strong>Craft at the landing pad</strong><span>Bring all five parts back and assemble your ship.</span></li>
        <li><strong>Launch into deep space</strong><span>Choose one of three generated planets, then keep travelling deeper without a final limit.</span></li>
      </ol>
      <p class="fuel-note"><strong>Fuel:</strong> starlight shards refill the ship up to its meter's limit. Every route shows its cost; routes you cannot afford stay locked.</p>
      <p class="earth-note"><strong>Earth recall:</strong> while away, the home control or star map teleports you to Earth for free.</p>
    </section>

    <dl>
      {#if isTouch}
        <div><dt>Walk</dt><dd>Drag the stick in the bottom-left corner, or tap a spot to walk there.</dd></div>
        <div><dt>Look</dt><dd>Drag anywhere else to orbit the camera. Pinch to zoom.</dd></div>
        <div><dt>Hop</dt><dd>Tap the hop button.</dd></div>
        <div><dt>Open a wonder</dt><dd>Walk up to a marker and tap the card that appears.</dd></div>
        <div><dt>Animals</dt><dd>Walk up to a sheep, a fox, or a polar bear and tap the card to pet it. Pet it twice and it follows you for a while.</dd></div>
        <div><dt>Shards</dt><dd>A dozen starlight shards glow purple across each planet. Walk over one to collect it.</dd></div>
        <div><dt>Ship parts</dt><dd>Walk into each missing part to recover it. Once all five are found, return to the landing pad and tap the card to craft, then launch.</dd></div>
        <div><dt>Deep space</dt><dd>Pick an affordable planet on the star map. Every arrival creates three routes one depth farther out.</dd></div>
        <div><dt>Earth</dt><dd>When away from home, tap the home control or choose the free Earth recall on the star map.</dd></div>
        <div><dt>Journal</dt><dd>Tap the book button to see the biomes, animals, and milestones you have collected across every planet.</dd></div>
        <div><dt>Boats</dt><dd>Stand at the shore and tap the card to launch a boat. Steer with the stick. Reach land to dock.</dd></div>
        <div><dt>The sky</dt><dd>It is always night here. Look up for the moon, the planets, the constellations, and the odd shooting star.</dd></div>
        <div><dt>Photo mode</dt><dd>Tap the camera button to hide the interface. Save a picture or leave from the corner buttons.</dd></div>
      {:else}
        <div><dt><kbd>W</kbd><kbd>A</kbd><kbd>S</kbd><kbd>D</kbd></dt><dd>Walk. Arrow keys work too.</dd></div>
        <div><dt><kbd>Shift</kbd></dt><dd>Run.</dd></div>
        <div><dt><kbd>Space</kbd></dt><dd>Hop.</dd></div>
        <div><dt><kbd>E</kbd></dt><dd>Interact with wonders and animals. At the landing pad, craft after recovering all five ship parts, then launch.</dd></div>
        <div><dt><kbd>M</kbd></dt><dd>Pull back to see the whole planet. Click a spot to walk there.</dd></div>
        <div><dt><kbd>J</kbd></dt><dd>Open the journal: biomes visited, animals befriended, milestones, and a dozen starlight shards to find on each planet.</dd></div>
        <div><dt>Deep space</dt><dd>Choose an affordable route on the star map. Every arrival generates three planets one depth farther out, with no final depth.</dd></div>
        <div><dt>Earth recall</dt><dd>When away from home, use the home control or the free Earth route on the star map.</dd></div>
        <div><dt>The sky</dt><dd>It is always night here. Look up for the moon, the planets, the constellations, auroras, and the odd shooting star.</dd></div>
        <div><dt><kbd>P</kbd></dt><dd>Photo mode: hides the interface. Press again or <kbd>Esc</kbd> to leave, or save a picture from the corner.</dd></div>
        <div><dt>Mouse</dt><dd>Drag to orbit the camera, scroll to zoom, click the ground to walk.</dd></div>
        <div><dt>Boats</dt><dd>Stand at the shore and press <kbd>E</kbd> to launch a boat. Steer with the walk keys. Reach land to dock.</dd></div>
      {/if}
    </dl>

    {#if seed}
      <div class="seed">
        <div class="seed-text">
          <p class="kicker">{isEarth ? 'Homeworld' : `Planet seed · depth ${depth}`}</p>
          <p class="seed-label"><code>{seed}</code></p>
          <p class="seed-help">
            {#if copyState === 'failed'}
              Copy this link by hand: <span class="seed-url">{link}</span>
            {:else}
              Share this link to show someone this exact world.
            {/if}
          </p>
        </div>
        <div class="seed-actions">
          <button class="chip" on:click={copyLink}>{copyState === 'copied' ? 'Copied' : 'Copy link'}</button>
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
  .journey {
    margin: 0 0 20px;
    padding: 14px 15px;
    border: 1px solid rgba(233, 196, 106, 0.2);
    border-radius: 12px;
    background: rgba(233, 196, 106, 0.045);
  }
  .journey ol {
    margin: 11px 0 12px;
    padding: 0;
    display: grid;
    gap: 8px;
    counter-reset: journey;
    list-style: none;
  }
  .journey li {
    display: grid;
    grid-template-columns: 21px 1fr;
    column-gap: 9px;
    counter-increment: journey;
  }
  .journey li::before {
    grid-row: 1 / span 2;
    width: 19px;
    height: 19px;
    display: grid;
    place-items: center;
    border: 1px solid rgba(233, 196, 106, 0.4);
    border-radius: 50%;
    color: #e9c46a;
    content: counter(journey);
    font-size: 0.6rem;
  }
  .journey li strong {
    color: rgba(242, 239, 230, 0.88);
    font-size: 0.76rem;
    font-weight: 500;
  }
  .journey li span {
    margin-top: 2px;
    color: rgba(242, 239, 230, 0.58);
    font-size: 0.69rem;
    line-height: 1.45;
  }
  .fuel-note,
  .earth-note {
    margin: 5px 0 0;
    color: rgba(242, 239, 230, 0.62);
    font-size: 0.69rem;
    line-height: 1.5;
  }
  .fuel-note strong,
  .earth-note strong {
    color: #e9c46a;
    font-weight: 500;
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
    .overlay {
      align-items: flex-end;
      padding: 12px;
    }
    .panel {
      max-height: min(92vh, 92dvh);
      padding: 21px 18px 18px;
    }
    dl div {
      grid-template-columns: 1fr;
      gap: 4px;
    }
  }
</style>
