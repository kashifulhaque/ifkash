<script lang="ts">
  // The Latticefall HUD: DOM over the canvas, drawn with the site's tokens so it follows the
  // design and theme (docs/game/07-presentation.md, "HUD"). It only reads: the engine pushes
  // state into the controller's stores and nothing here writes simulation state.
  import type { Readable } from 'svelte/store';
  import { barLength, isQuiet, STATUS_NAMES, type HudState, type QuickSlot } from './hudState';
  import type { HudSettings } from './hudSettings';
  import type { HudController } from './hudStore';
  import CouriersLetter from '../ending/CouriersLetter.svelte';

  export let hud: HudController;
  export let settings: Readable<HudSettings>;

  const { state, notices, lock } = hud;

  // One token per status, so the meters can be told apart without a new color.
  const STATUS_TINT = ['--moss', '--clay', '--fjord', '--danger', '--falu', '--fjord-strong'];
  // And one shape per status (WP-082), so color is never the only cue: circle, diamond,
  // triangle, square, cross, hexagon. The shapes are drawn with clip-path in the styles.
  const STATUS_SHAPE = ['circle', 'diamond', 'triangle', 'square', 'cross', 'hexagon'];

  // What Interact would do, by the core's `interact_prompt` code (1, the Lantern, has its own line).
  const PROMPTS: Record<number, string> = {
    2: 'Take',
    3: 'Pull the lever',
    4: 'Open the door',
    5: 'Read the message',
    6: 'Climb the ladder',
    7: 'Kick the ladder down',
    8: 'Speak',
    9: 'Look closer',
    30: 'Ride the cart',
    31: 'Push the cart',
    32: 'Push the hatch open',
    40: 'Turn the mirror',
    55: 'Light the brazier',
    56: 'Touch the sigil'
  };

  let innerWidth = 1280;

  $: s = $state as HudState;
  $: scale = $settings.scale / 100;
  $: hidden = $settings.visibility === 'hidden';
  $: faded = $settings.visibility === 'auto' && isQuiet(s) && $notices.length === 0;
  $: building = STATUS_NAMES.map((name, i) => ({ name, value: s.status[i], tint: STATUS_TINT[i], shape: STATUS_SHAPE[i] })).filter(
    (m) => m.value > 0
  );
  $: bars = [
    { key: 'hp', label: 'Health', bar: s.hp, color: '--falu' },
    { key: 'fp', label: 'Focus', bar: s.fp, color: '--fjord' },
    { key: 'st', label: 'Stamina', bar: s.stamina, color: '--moss' }
  ];

  // The boss bar centers between the bottom corner groups when it fits, at its own width.
  // When they leave too little room (high scale, narrow window) it rides above them instead.
  const EDGE = 24;
  const BOSS_MAX = 560;
  const BOSS_MIN = 300;
  const LEFT_REACH = 270;
  const RIGHT_REACH = 190;
  const CORNER_HEIGHT = 200;
  $: free = innerWidth - 2 * (EDGE + Math.max(LEFT_REACH, RIGHT_REACH) * scale + 8);
  $: lifted = free < BOSS_MIN * scale;
  // Width and bottom are in unscaled units: the boss block is scaled by `--hs` about its bottom center.
  $: bossWidth = Math.max(240, Math.min(BOSS_MAX, (lifted ? innerWidth - 2 * EDGE : free) / scale));
  $: bossBottom = lifted ? EDGE + CORNER_HEIGHT * scale + 8 : 32;

  function pct(value: number, max: number): number {
    return max > 0 ? Math.max(0, Math.min(100, (value / max) * 100)) : 0;
  }

  function slotLabel(q: QuickSlot, empty: string): string {
    if (q.num === 0) return empty;
    return q.count === null ? q.name : `${q.name}, ${q.count}`;
  }
</script>

<svelte:window bind:innerWidth />

{#if !hidden}
  <div class="hud" class:faded style="--hs: {scale}" aria-label="Heads-up display" role="group">
    <section class="tl" aria-label="Vitals">
      {#if building.length}
        <div class="statuses" aria-label="Status buildup">
          {#each building as m (m.name)}
            <div
              class="status"
              role="meter"
              aria-label={m.name}
              aria-valuemin="0"
              aria-valuemax="100"
              aria-valuenow={Math.round(m.value * 100)}
              style="--tint: var({m.tint})"
            >
              <span class="status-name"><span class="glyph {m.shape}" aria-hidden="true"></span>{m.name}</span>
              <span class="status-track"><span class="status-fill" style="width: {m.value * 100}%"></span></span>
            </div>
          {/each}
        </div>
      {/if}
      {#each bars as b (b.key)}
        <div
          class="bar {b.key}"
          role="meter"
          aria-label={b.label}
          aria-valuemin="0"
          aria-valuemax={Math.round(b.bar.max)}
          aria-valuenow={Math.round(b.bar.value)}
          style="width: {barLength(b.bar.max)}px; --fill: var({b.color})"
        >
          <span class="bar-fill" style="width: {pct(b.bar.value, b.bar.max)}%"></span>
        </div>
      {/each}
    </section>

    <section class="bl" aria-label="Quick slots">
      <div class="cross">
        <div class="slot spell" aria-label={`Spell: ${slotLabel(s.spell, 'none')}`}>
          <span class="slot-key">Spell</span>
          <span class="slot-name">{s.spell.name || '—'}</span>
        </div>
        <div class="slot left" aria-label={`Left hand: ${slotLabel(s.left, 'empty')}`}>
          <span class="slot-key">Left</span>
          <span class="slot-name">{s.left.name || '—'}</span>
        </div>
        <div class="slot right" aria-label={`Right hand: ${slotLabel(s.right, 'empty')}`}>
          <span class="slot-key">Right</span>
          <span class="slot-name">{s.right.name || '—'}</span>
        </div>
        <div class="slot item" aria-label={`Item: ${slotLabel(s.item, 'none')}`}>
          <span class="slot-key">Item</span>
          <span class="slot-name">{s.item.name || '—'}</span>
          {#if s.item.count !== null}<span class="slot-count">{s.item.count}</span>{/if}
        </div>
      </div>
      {#if s.vigil.max + s.lucid.max > 0}
        <p class="flasks" aria-label="Flasks">
          <span>Vigil <b>{s.vigil.value}</b>/{s.vigil.max}</span>
          <span>Lucid <b>{s.lucid.value}</b>/{s.lucid.max}</span>
        </p>
      {/if}
    </section>

    <section class="br" aria-label="Glimmer">
      <span class="glimmer-label">Glimmer</span>
      <span class="glimmer-value">{Math.round(s.glimmer).toLocaleString('en-US')}</span>
    </section>

    {#if s.boss}
      <section
        class="boss"
        aria-label="Boss"
        style="width: {bossWidth}px; margin-left: {-bossWidth / 2}px; bottom: {bossBottom}px"
      >
        <p class="boss-name">{s.boss.name}</p>
        <div
          class="boss-bar"
          role="meter"
          aria-label={`${s.boss.name} health`}
          aria-valuemin="0"
          aria-valuemax="100"
          aria-valuenow={Math.round(s.boss.frac * 100)}
        >
          <span class="boss-fill" style="width: {s.boss.frac * 100}%"></span>
        </div>
      </section>
    {/if}

    <section class="center">
      {#if $lock}
        <span class="lock" style="left: {$lock.x}px; top: {$lock.y}px" aria-hidden="true"></span>
      {/if}
      <div
        class="stack"
        class:above-boss={lifted && s.boss !== null}
        style={lifted && s.boss ? `bottom: ${bossBottom + 70 * scale}px` : ''}
      >
        <div class="notices" aria-live="polite">
          {#each $notices as n (n.id)}
            <p class="notice" class:message={n.message}>{n.text}</p>
          {/each}
        </div>
        {#if s.prompt === 1}
          <p class="prompt"><kbd>E</kbd> Rest at the Lantern</p>
        {:else if s.prompt > 1 && PROMPTS[s.prompt]}
          <p class="prompt"><kbd>E</kbd> {PROMPTS[s.prompt]}</p>
        {/if}
      </div>
    </section>
  </div>
{/if}

<!-- The ending (WP-100): the Courier's Letter after B12 falls. -->
<CouriersLetter />

<style>
  .hud {
    position: absolute;
    inset: 0;
    pointer-events: none;
    user-select: none;
    font-family: var(--font-sans);
    color: var(--ink);
    transition: opacity 0.4s ease;
  }

  .hud.faded {
    opacity: 0;
  }

  /* Each corner scales from its own corner, so 150% grows toward the middle. */
  .tl,
  .bl,
  .br,
  .boss {
    position: absolute;
    transform: scale(var(--hs));
  }

  .tl {
    top: calc(var(--s-6) + 28px);
    left: var(--s-5);
    transform-origin: top left;
    display: flex;
    flex-direction: column;
    gap: 6px;
  }

  .bl {
    bottom: var(--s-5);
    left: var(--s-5);
    transform-origin: bottom left;
  }

  .br {
    bottom: var(--s-5);
    right: var(--s-5);
    transform-origin: bottom right;
    display: flex;
    align-items: baseline;
    gap: var(--s-3);
    padding: var(--s-2) var(--s-4);
    background: color-mix(in srgb, var(--surface) 80%, transparent);
    border: 1px solid var(--line-strong);
    border-radius: 3px;
  }

  /* Bars: thin, hairline-bordered, over a wash so they read on any scene. */
  .bar {
    height: 12px;
    box-sizing: border-box;
    border: 1px solid var(--line-strong);
    border-radius: 2px;
    background: color-mix(in srgb, var(--surface) 72%, transparent);
    overflow: hidden;
  }

  .bar-fill {
    display: block;
    height: 100%;
    background: var(--fill);
    transition: width 0.12s linear;
  }

  /* Status meters sit above the bars and only exist while a status builds. */
  .statuses {
    position: absolute;
    bottom: 100%;
    left: 0;
    margin-bottom: 8px;
    display: flex;
    flex-wrap: wrap;
    gap: 4px var(--s-3);
    width: max-content;
    max-width: 420px;
  }

  .status {
    width: 72px;
  }

  .status-name {
    display: block;
    width: max-content;
    padding: 0 4px;
    margin-bottom: 2px;
    font-size: calc(11px * var(--lf-text, 1));
    line-height: 1.3;
    color: var(--ink);
    background: color-mix(in srgb, var(--surface) 84%, transparent);
    border-radius: 2px;
  }

  .status-track {
    display: block;
    height: 5px;
    box-sizing: border-box;
    border: 1px solid var(--line-strong);
    border-radius: 2px;
    background: color-mix(in srgb, var(--surface) 72%, transparent);
    overflow: hidden;
  }

  .status-fill {
    display: block;
    height: 100%;
    background: var(--tint);
  }

  /* The shape that goes with each status's color. */
  .glyph {
    display: inline-block;
    width: 8px;
    height: 8px;
    margin-right: 4px;
    background: var(--tint);
    vertical-align: -1px;
  }

  .glyph.circle {
    border-radius: 50%;
  }

  .glyph.diamond {
    clip-path: polygon(50% 0, 100% 50%, 50% 100%, 0 50%);
  }

  .glyph.triangle {
    clip-path: polygon(50% 0, 100% 100%, 0 100%);
  }

  .glyph.cross {
    clip-path: polygon(35% 0, 65% 0, 65% 35%, 100% 35%, 100% 65%, 65% 65%, 65% 100%, 35% 100%, 35% 65%, 0 65%, 0 35%, 35% 35%);
  }

  .glyph.hexagon {
    clip-path: polygon(25% 0, 75% 0, 100% 50%, 75% 100%, 25% 100%, 0 50%);
  }

  /* The bars differ by fill pattern as well as color: health is solid, focus is hatched, stamina
     is dotted (WP-082). */
  .bar.fp .bar-fill {
    background-image: repeating-linear-gradient(135deg, rgb(0 0 0 / 0.3) 0 2px, transparent 2px 5px);
    background-color: var(--fill);
  }

  .bar.st .bar-fill {
    background-image: radial-gradient(circle, rgb(0 0 0 / 0.38) 1px, transparent 1.5px);
    background-size: 5px 5px;
    background-color: var(--fill);
  }

  /* Quick slots in a cross: spell above, hands left and right, item below. */
  .cross {
    display: grid;
    grid-template-columns: repeat(3, 84px);
    grid-template-rows: repeat(3, 52px);
    gap: 4px;
  }

  .slot {
    position: relative;
    box-sizing: border-box;
    padding: 4px 6px;
    display: flex;
    flex-direction: column;
    justify-content: space-between;
    overflow: hidden;
    background: color-mix(in srgb, var(--surface) 80%, transparent);
    border: 1px solid var(--line-strong);
    border-radius: 3px;
  }

  .spell {
    grid-column: 2;
    grid-row: 1;
  }

  .left {
    grid-column: 1;
    grid-row: 2;
  }

  .right {
    grid-column: 3;
    grid-row: 2;
  }

  .item {
    grid-column: 2;
    grid-row: 3;
  }

  .slot-key {
    font-family: var(--font-mono);
    font-size: 10px;
    letter-spacing: 0.06em;
    text-transform: uppercase;
    color: var(--ink-3);
  }

  .slot-name {
    font-size: var(--text-xs);
    line-height: 1.15;
    color: var(--ink);
    display: -webkit-box;
    -webkit-line-clamp: 2;
    line-clamp: 2;
    -webkit-box-orient: vertical;
    overflow: hidden;
  }

  .slot-count {
    position: absolute;
    top: 3px;
    right: 6px;
    font-family: var(--font-mono);
    font-size: var(--text-sm);
    font-variant-numeric: tabular-nums;
    color: var(--ink);
  }

  .flasks {
    margin: 6px 0 0;
    padding: 2px var(--s-2);
    width: max-content;
    display: flex;
    gap: var(--s-3);
    font-size: var(--text-xs);
    color: var(--ink-2);
    background: color-mix(in srgb, var(--surface) 84%, transparent);
    border-radius: 3px;
  }

  .flasks b {
    font-family: var(--font-mono);
    font-weight: 500;
    color: var(--ink);
  }

  .glimmer-label {
    font-size: var(--text-xs);
    letter-spacing: 0.08em;
    text-transform: uppercase;
    color: var(--ink-3);
  }

  .glimmer-value {
    font-family: var(--font-mono);
    font-size: var(--text-lg);
    font-variant-numeric: tabular-nums;
    color: var(--ink);
  }

  /* Boss bar: bottom center, with the boss's name in the display font. */
  .boss {
    left: 50%;
    transform-origin: bottom center;
  }

  .boss-name {
    margin: 0 auto 6px;
    width: max-content;
    max-width: 100%;
    padding: 2px var(--s-3);
    font-family: var(--font-display);
    font-size: var(--text-md);
    text-align: center;
    color: var(--ink);
    background: color-mix(in srgb, var(--surface) 84%, transparent);
    border: 1px solid var(--line);
    border-radius: 3px;
  }

  .boss-bar {
    height: 10px;
    box-sizing: border-box;
    border: 1px solid var(--line-strong);
    border-radius: 2px;
    background: color-mix(in srgb, var(--surface) 72%, transparent);
    overflow: hidden;
  }

  .boss-fill {
    display: block;
    height: 100%;
    background: var(--falu);
    transition: width 0.15s linear;
  }

  .center {
    position: absolute;
    inset: 0;
  }

  .lock {
    position: absolute;
    width: 18px;
    height: 18px;
    margin: -9px 0 0 -9px;
    box-sizing: border-box;
    border: 2px solid var(--falu);
    border-radius: 2px;
    transform: rotate(45deg);
    background: color-mix(in srgb, var(--surface) 30%, transparent);
  }

  /* Prompts and notifications sit just below the middle of the screen. */
  .stack {
    position: absolute;
    left: 50%;
    top: 48%;
    transform: translateX(-50%) scale(var(--hs));
    transform-origin: top center;
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: var(--s-2);
  }

  /* A boss bar riding above the corners takes the space below the middle; the stack moves up. */
  .stack.above-boss {
    top: auto;
    transform-origin: bottom center;
  }

  .notices {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 4px;
    min-height: 0;
  }

  .notice,
  .prompt {
    margin: 0;
    padding: 4px var(--s-3);
    font-size: calc(var(--text-sm) * var(--lf-text, 1));
    color: var(--ink);
    background: color-mix(in srgb, var(--surface) 84%, transparent);
    border: 1px solid var(--line-strong);
    border-radius: 3px;
    white-space: nowrap;
  }

  .notice {
    animation: rise 0.25s ease-out;
  }

  /* An authored message: a few sentences, so it wraps instead of running off the screen. */
  .notice.message {
    max-width: min(34rem, 92vw);
    white-space: normal;
    text-align: center;
    line-height: 1.45;
    padding: var(--s-2) var(--s-4);
  }

  kbd {
    font-family: var(--font-mono);
    font-size: var(--text-xs);
    padding: 0 6px;
    margin-right: 6px;
    border: 1px solid var(--ink-3);
    border-radius: 2px;
  }

  @keyframes rise {
    from {
      opacity: 0;
      transform: translateY(4px);
    }
  }

  @media (prefers-reduced-motion: reduce) {
    .hud,
    .bar-fill,
    .boss-fill {
      transition: none;
    }

    .notice {
      animation: none;
    }
  }
</style>
