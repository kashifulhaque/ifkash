<script lang="ts">
  // The Latticefall HUD: DOM over the canvas (docs/game/07-presentation.md, "HUD"), drawn in
  // the style of a 2010s action RPG: thin framed bars, a cross of quick slots, and text over
  // dark bands, in its own colors rather than the site's. Color-blind modes still remap the
  // bar colors through the site tokens. It only reads: the engine pushes state into the
  // controller's stores and nothing here writes simulation state.
  import type { Readable } from 'svelte/store';
  import { barLength, isQuiet, STATUS_NAMES, type HudState, type QuickSlot } from './hudState';
  import type { HudSettings } from './hudSettings';
  import type { HudController } from './hudStore';
  import CouriersLetter from '../ending/CouriersLetter.svelte';
  import TutorialEnd from '../ending/TutorialEnd.svelte';
  import { loadSettings } from '../../input/bindings';
  import { actionGlyph, lastDevice } from '../../input/device';
  import { promptParts } from '../../input/prompts';

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
    10: 'Pass through the fog',
    30: 'Ride the cart',
    31: 'Push the cart',
    32: 'Push the hatch open',
    40: 'Turn the mirror',
    55: 'Light the brazier',
    56: 'Touch the sigil'
  };

  let innerWidth = 1280;

  // The Interact glyph for the device used last: Y on a pad, E on the keyboard by default.
  // Read from the saved bindings when the prompt or the device changes, so a remap shows.
  $: interactKey = s.prompt > 0 ? actionGlyph(loadSettings().bindings, 'interact', $lastDevice) || 'E' : 'E';
  // Authored messages name controls as placeholders (`{interact}`), filled in from the saved
  // bindings for the device used last, so the text switches the moment the player does.
  $: bindings = readBindings($notices.length, $lastDevice);
  $: pad = $lastDevice === 'pad';

  // The arguments only make Svelte read the bindings again when a notice or the device changes.
  // eslint-disable-next-line @typescript-eslint/no-unused-vars
  function readBindings(..._deps: unknown[]) {
    return loadSettings().bindings;
  }

  /** Xbox face buttons get their own round, colored cap; everything else is a key cap or pill. */
  function faceButton(glyph: string): string | null {
    return /^[ABXY]$/.test(glyph) ? glyph : null;
  }

  $: s = $state as HudState;
  $: scale = $settings.scale / 100;
  $: hidden = $settings.visibility === 'hidden';
  $: faded = $settings.visibility === 'auto' && isQuiet(s) && $notices.length === 0;
  $: building = STATUS_NAMES.map((name, i) => ({ name, value: s.status[i], tint: STATUS_TINT[i], shape: STATUS_SHAPE[i] })).filter(
    (m) => m.value > 0
  );
  $: bars = [
    { key: 'hp', label: 'Health', bar: s.hp },
    { key: 'fp', label: 'Focus', bar: s.fp },
    { key: 'st', label: 'Stamina', bar: s.stamina }
  ];

  // The boss bar centers between the bottom corner groups when it fits, at its own width.
  // When they leave too little room (high scale, narrow window) it rides above them instead.
  const EDGE = 24;
  const BOSS_MAX = 560;
  const BOSS_MIN = 300;
  const LEFT_REACH = 200;
  const RIGHT_REACH = 170;
  const CORNER_HEIGHT = 210;
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
          style="width: {barLength(b.bar.max)}px"
        >
          <span class="bar-trail" style="width: {pct(b.bar.value, b.bar.max)}%"></span>
          <span class="bar-fill" style="width: {pct(b.bar.value, b.bar.max)}%"></span>
        </div>
      {/each}
    </section>

    <section class="bl" aria-label="Quick slots">
      <div class="cross">
        <div class="slot spell" class:empty={!s.spell.num} aria-label={`Spell: ${slotLabel(s.spell, 'none')}`}>
          <span class="slot-name" aria-hidden="true">{s.spell.name}</span>
        </div>
        <div class="slot left" class:empty={!s.left.num} aria-label={`Left hand: ${slotLabel(s.left, 'empty')}`}>
          <span class="slot-name" aria-hidden="true">{s.left.name}</span>
        </div>
        <div
          class="slot right"
          class:empty={!s.right.num}
          class:two-handing={s.twoHanding}
          aria-label={`Right hand: ${slotLabel(s.right, 'empty')}${s.twoHanding ? ', two-handed' : ''}`}
        >
          <span class="slot-name" aria-hidden="true">{s.right.name}</span>
          {#if s.twoHanding}<span class="slot-mark" aria-hidden="true">2H</span>{/if}
        </div>
        <div class="slot item" class:empty={!s.item.num} aria-label={`Item: ${slotLabel(s.item, 'none')}`}>
          <span class="slot-name" aria-hidden="true">{s.item.name}</span>
          {#if s.item.count !== null}<span class="slot-count" aria-hidden="true">{s.item.count}</span>{/if}
        </div>
      </div>
      {#if s.vigil.max + s.lucid.max > 0}
        <p class="flasks" aria-label={`Flasks: Vigil ${s.vigil.value} of ${s.vigil.max}, Lucid ${s.lucid.value} of ${s.lucid.max}`}>
          <span class="flask vigil" aria-hidden="true"><i></i><b>{s.vigil.value}</b></span>
          <span class="flask lucid" aria-hidden="true"><i></i><b>{s.lucid.value}</b></span>
        </p>
      {/if}
    </section>

    <section class="br" aria-label={`Glimmer: ${Math.round(s.glimmer)}`}>
      <span class="glimmer-mark" aria-hidden="true"></span>
      <span class="glimmer-value" aria-hidden="true">{Math.round(s.glimmer).toLocaleString('en-US')}</span>
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
          <span class="bar-trail" style="width: {s.boss.frac * 100}%"></span>
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
            <p class="band notice" class:message={n.message}>
              {#each promptParts(n.text, bindings, $lastDevice) as part}{#if 'text' in part}{part.text}{:else}<kbd
                    class:pad
                    class:face={faceButton(part.control)}
                    data-btn={faceButton(part.control)}>{part.control}</kbd
                  >{/if}{/each}
            </p>
          {/each}
        </div>
        {#if s.prompt === 1 || (s.prompt > 1 && PROMPTS[s.prompt])}
          <p class="band prompt">
            <kbd class:pad class:face={faceButton(interactKey)} data-btn={faceButton(interactKey)}>{interactKey}</kbd>
            {s.prompt === 1 ? 'Rest at the Lantern' : PROMPTS[s.prompt]}
          </p>
        {/if}
      </div>
    </section>
  </div>
{/if}

<!-- The ending (WP-100): the Courier's Letter after B12 falls. -->
<CouriersLetter />
<!-- The end of the tutorial while only Waking Hollow is open: after B01 falls. -->
<TutorialEnd />

<style>
  .hud {
    /* The HUD's own palette: pale parchment text and bronze frames over dark washes. */
    --hud-ink: #ece5d3;
    --hud-dim: rgb(236 229 211 / 0.66);
    --hud-frame: rgb(196 170 118 / 0.62);
    --hud-frame-dim: rgb(196 170 118 / 0.3);
    --hud-back: rgb(10 8 6 / 0.62);
    --hud-shadow: 0 1px 2px rgb(0 0 0 / 0.9), 0 0 6px rgb(0 0 0 / 0.6);
    --hud-hp: #a8231e;
    --hud-fp: #2f56b8;
    --hud-st: #3f8a3c;
    --hud-trail: #d9c27a;
    position: absolute;
    inset: 0;
    pointer-events: none;
    user-select: none;
    font-family: var(--font-sans);
    color: var(--hud-ink);
    transition: opacity 0.4s ease;
  }

  /* Color-blind modes (WP-082) remap these site tokens; the bars follow them there. */
  :global(.lf-root[data-cb]) .hud {
    --hud-hp: var(--falu);
    --hud-fp: var(--fjord);
    --hud-st: var(--moss);
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
    top: var(--s-5);
    left: var(--s-5);
    transform-origin: top left;
    display: flex;
    flex-direction: column;
    gap: 5px;
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
    align-items: center;
    gap: 10px;
    min-width: 120px;
    justify-content: flex-end;
    padding: 5px 4px 5px 28px;
    /* A hairline above and below that fades out toward the middle of the screen. */
    border-top: 1px solid;
    border-bottom: 1px solid;
    border-image: linear-gradient(90deg, transparent, var(--hud-frame) 40%) 1;
    background: linear-gradient(90deg, transparent, rgb(0 0 0 / 0.45) 40%);
  }

  /* Touch mode's buttons fill the bottom right, so Glimmer moves under the Menu button. */
  :global(.lf-root[data-touch]) .br {
    top: calc(var(--s-3) + 52px + env(safe-area-inset-top, 0px));
    bottom: auto;
    transform-origin: top right;
  }

  /* Bars: thin, framed in bronze over a dark wash, with no panel behind them. */
  .bar,
  .boss-bar {
    position: relative;
    box-sizing: border-box;
    border: 1px solid var(--hud-frame);
    background: var(--hud-back);
    box-shadow: 0 0 0 1px rgb(0 0 0 / 0.55);
    overflow: hidden;
  }

  .bar {
    height: 8px;
  }

  .bar.hp {
    height: 10px;
  }

  .bar-fill,
  .bar-trail,
  .boss-fill {
    position: absolute;
    inset: 0 auto 0 0;
    display: block;
  }

  .bar-fill {
    background: linear-gradient(180deg, rgb(255 255 255 / 0.18), transparent 55%, rgb(0 0 0 / 0.25)), var(--fill);
    transition: width 0.12s linear;
  }

  /* Damage leaves a pale trail that catches up a moment later. */
  .bar-trail {
    background: var(--hud-trail);
    opacity: 0.75;
    transition: width 0.5s ease-in 0.5s;
  }

  .bar.hp {
    --fill: var(--hud-hp);
  }

  .bar.fp {
    --fill: var(--hud-fp);
  }

  .bar.st {
    --fill: var(--hud-st);
  }

  /* Focus and stamina never trail; only health and the boss do. */
  .bar.fp .bar-trail,
  .bar.st .bar-trail {
    display: none;
  }

  /* Status meters sit above the bars and only exist while a status builds. */
  .statuses {
    position: absolute;
    top: 100%;
    left: 0;
    margin-top: 10px;
    display: flex;
    flex-wrap: wrap;
    gap: 6px var(--s-3);
    width: max-content;
    max-width: 420px;
  }

  .status {
    width: 72px;
  }

  .status-name {
    display: block;
    width: max-content;
    margin-bottom: 3px;
    font-size: calc(11px * var(--lf-text, 1));
    line-height: 1.3;
    color: var(--hud-ink);
    text-shadow: var(--hud-shadow);
  }

  .status-track {
    display: block;
    height: 4px;
    box-sizing: border-box;
    border: 1px solid var(--hud-frame-dim);
    background: var(--hud-back);
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

  /* The bars differ by fill texture as well as color (WP-082): health is solid, focus is
     faintly hatched, stamina faintly dotted. */
  .bar.fp .bar-fill {
    background:
      repeating-linear-gradient(135deg, rgb(0 0 0 / 0.22) 0 2px, transparent 2px 5px),
      linear-gradient(180deg, rgb(255 255 255 / 0.18), transparent 55%, rgb(0 0 0 / 0.25)),
      var(--fill);
  }

  .bar.st .bar-fill {
    background:
      radial-gradient(circle, rgb(0 0 0 / 0.3) 1px, transparent 1.5px) 0 0 / 5px 5px,
      linear-gradient(180deg, rgb(255 255 255 / 0.18), transparent 55%, rgb(0 0 0 / 0.25)),
      var(--fill);
  }

  /* Quick slots in a cross: spell above, hands left and right, item below. */
  .cross {
    display: grid;
    grid-template-columns: repeat(3, 58px);
    grid-template-rows: repeat(3, 58px);
    gap: 3px;
  }

  .slot {
    position: relative;
    box-sizing: border-box;
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 4px;
    overflow: hidden;
    background: radial-gradient(circle at 50% 40%, rgb(40 34 26 / 0.7), rgb(6 5 4 / 0.78));
    border: 1px solid var(--hud-frame);
    box-shadow:
      inset 0 0 0 1px rgb(0 0 0 / 0.6),
      inset 0 0 10px rgb(0 0 0 / 0.7);
  }

  .slot.empty {
    background: rgb(6 5 4 / 0.35);
    border-color: var(--hud-frame-dim);
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

  /* Two-handing: the right weapon is held in both hands. */
  .right.two-handing {
    border-color: var(--hud-trail);
  }

  .slot-name {
    font-family: var(--font-serif);
    font-size: 11px;
    line-height: 1.1;
    text-align: center;
    color: var(--hud-ink);
    text-shadow: var(--hud-shadow);
    display: -webkit-box;
    -webkit-line-clamp: 3;
    line-clamp: 3;
    -webkit-box-orient: vertical;
    overflow: hidden;
  }

  .slot-count,
  .slot-mark {
    position: absolute;
    right: 4px;
    bottom: 2px;
    font-family: var(--font-mono);
    font-size: 12px;
    font-weight: 600;
    font-variant-numeric: tabular-nums;
    color: var(--hud-ink);
    text-shadow: var(--hud-shadow);
  }

  .slot-mark {
    top: 2px;
    bottom: auto;
    font-size: 9px;
    color: var(--hud-trail);
  }

  .flasks {
    margin: 6px 0 0;
    display: flex;
    gap: var(--s-3);
    font-size: 13px;
  }

  .flask {
    display: inline-flex;
    align-items: center;
    gap: 5px;
  }

  /* A small flask shape in the flask's color: Vigil heals (red), Lucid restores focus (blue). */
  .flask i {
    width: 8px;
    height: 12px;
    background: var(--hud-hp);
    border: 1px solid var(--hud-frame);
    border-radius: 2px 2px 4px 4px;
    box-shadow: 0 0 4px rgb(0 0 0 / 0.8);
  }

  .flask.lucid i {
    background: var(--hud-fp);
  }

  .flask b {
    font-family: var(--font-mono);
    font-weight: 600;
    color: var(--hud-ink);
    text-shadow: var(--hud-shadow);
  }

  /* Glimmer: a pale diamond and the count, nothing else. */
  .glimmer-mark {
    width: 9px;
    height: 9px;
    transform: rotate(45deg);
    background: radial-gradient(circle, #fff6d8, var(--hud-trail));
    box-shadow: 0 0 6px rgb(217 194 122 / 0.7);
  }

  .glimmer-value {
    font-family: var(--font-serif);
    font-size: 19px;
    font-variant-numeric: tabular-nums lining-nums;
    letter-spacing: 0.02em;
    color: var(--hud-ink);
    text-shadow: var(--hud-shadow);
  }

  /* Boss bar: bottom center, the boss's name above it on the left. */
  .boss {
    left: 50%;
    transform-origin: bottom center;
  }

  .boss-name {
    margin: 0 0 4px 2px;
    font-family: var(--font-serif);
    font-size: 17px;
    letter-spacing: 0.02em;
    color: var(--hud-ink);
    text-shadow: var(--hud-shadow);
  }

  .boss-bar {
    height: 9px;
  }

  .boss-fill {
    background: linear-gradient(180deg, rgb(255 255 255 / 0.16), transparent 55%, rgb(0 0 0 / 0.3)), var(--hud-hp);
    transition: width 0.15s linear;
  }

  .center {
    position: absolute;
    inset: 0;
  }

  /* The lock-on mark: a small bright point with a soft ring, over the target. */
  .lock {
    position: absolute;
    width: 9px;
    height: 9px;
    margin: -4.5px 0 0 -4.5px;
    border-radius: 50%;
    background: radial-gradient(circle, #fff 0 35%, rgb(255 246 220 / 0.85) 55%, transparent 72%);
    box-shadow:
      0 0 0 2px rgb(0 0 0 / 0.35),
      0 0 8px 2px rgb(255 240 200 / 0.55);
  }

  /* Prompts and notifications sit just below the middle of the screen. */
  .stack {
    position: absolute;
    left: 50%;
    top: 56%;
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

  /* Text over a dark band that fades out at both ends, with bronze hairlines above and below. */
  .band {
    position: relative;
    margin: 0;
    padding: 6px 56px;
    font-size: calc(var(--text-sm) * var(--lf-text, 1));
    color: var(--hud-ink);
    text-shadow: var(--hud-shadow);
    white-space: nowrap;
    background: linear-gradient(90deg, transparent, rgb(0 0 0 / 0.72) 22%, rgb(0 0 0 / 0.72) 78%, transparent);
  }

  .band::before,
  .band::after {
    content: '';
    position: absolute;
    left: 0;
    right: 0;
    height: 1px;
    background: linear-gradient(90deg, transparent, var(--hud-frame) 30%, var(--hud-frame) 70%, transparent);
  }

  .band::before {
    top: 0;
  }

  .band::after {
    bottom: 0;
  }

  .notice {
    animation: rise 0.25s ease-out;
  }

  /* An authored message: a few sentences, so it wraps instead of running off the screen. */
  .notice.message {
    max-width: min(36rem, 92vw);
    white-space: normal;
    text-align: center;
    font-family: var(--font-serif);
    font-size: calc(16px * var(--lf-text, 1));
    line-height: 1.55;
    padding: var(--s-3) 64px;
  }

  .prompt {
    display: flex;
    align-items: center;
    gap: 8px;
    font-size: calc(15px * var(--lf-text, 1));
  }

  /* A key cap. */
  kbd {
    display: inline-block;
    min-width: 1.6em;
    box-sizing: border-box;
    margin: 0 2px;
    padding: 0 5px;
    font-family: var(--font-mono);
    font-size: 0.78em;
    font-weight: 600;
    line-height: 1.55;
    text-align: center;
    text-shadow: none;
    color: var(--hud-ink);
    background: rgb(255 255 255 / 0.08);
    border: 1px solid rgb(236 229 211 / 0.7);
    border-bottom-width: 2px;
    border-radius: 3px;
    vertical-align: 0.08em;
  }

  /* A pad button: shoulders, triggers, and sticks are pills; face buttons are round. */
  kbd.pad {
    border-width: 1px;
    border-radius: 999px;
    padding: 0 7px;
    background: rgb(0 0 0 / 0.55);
  }

  kbd.pad.face {
    width: 1.75em;
    height: 1.75em;
    padding: 0;
    line-height: calc(1.75em - 2px);
    font-family: var(--font-sans);
    font-weight: 700;
    border: 1px solid var(--btn);
    color: var(--btn);
  }

  kbd[data-btn='A'] {
    --btn: #7fcf6a;
  }

  kbd[data-btn='B'] {
    --btn: #f0716a;
  }

  kbd[data-btn='X'] {
    --btn: #6aa8f0;
  }

  kbd[data-btn='Y'] {
    --btn: #f0d060;
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
    .bar-trail,
    .boss-fill {
      transition: none;
    }

    .notice {
      animation: none;
    }
  }
</style>
