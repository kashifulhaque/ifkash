<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import { name, tagline } from '$lib/content';
  import { pickDailyMutators } from '$lib/game/mutators';
  import { dayString, dailySeed, seededRng } from '$lib/game/rng';

  export let isTouch: boolean;
  export let resumed = false; // true when pointer lock was lost mid-game
  export let muted = false;
  export let dayNightCycle = false;

  const dispatch = createEventDispatcher();
  const today = dayString();
  // Preview today's mutators deterministically (matches the seed the Game uses).
  const dailyMutators = pickDailyMutators(seededRng(dailySeed()));
</script>

<div class="start">
  <div class="grid" aria-hidden="true"></div>
  <div class="radar radar-one" aria-hidden="true"></div>
  <div class="radar radar-two" aria-hidden="true"></div>
  <div class="corner corner-tl" aria-hidden="true"></div>
  <div class="corner corner-br" aria-hidden="true"></div>
  <div class="inner">
    <div class="status-line"><span class="status-dot"></span> PORTFOLIO INSTANCE ONLINE <span class="status-divider">//</span> BUILD 2026.07</div>
    {#if !resumed}
      <p class="eyebrow">WELCOME TO THE FIELD</p>
      <h1 class="title"><span>{name.split(' ')[0]}</span> {name.split(' ').slice(1).join(' ')}</h1>
      <p class="tagline">{tagline}</p>

      <div class="mission-grid">
        <div class="mission-card">
          <span class="mission-number">01</span>
          <span class="mission-icon">⌾</span>
          <strong>EXPLORE</strong>
          <small>6 portfolio sectors</small>
        </div>
        <div class="mission-card hot">
          <span class="mission-number">02</span>
          <span class="mission-icon">✦</span>
          <strong>ENGAGE</strong>
          <small>Clear enemy waves</small>
        </div>
        <div class="mission-card">
          <span class="mission-number">03</span>
          <span class="mission-icon">▣</span>
          <strong>UNLOCK</strong>
          <small>Loot the intel caches</small>
        </div>
      </div>

      <div class="how">
        {#if isTouch}
          <p>LEFT THUMB · MOVE <b>—</b> RIGHT THUMB · LOOK <b>—</b> FIRE · SHOOT <b>—</b> JUMP / DUCK</p>
        {:else}
          <p>WASD · MOVE <b>—</b> SHIFT · RUN <b>—</b> SPACE · JUMP <b>—</b> C · CROUCH <b>—</b> CLICK · SHOOT <b>—</b> E · LOOT</p>
        {/if}
        <p class="goal">THE TARGETS ARE ARMED. YOUR PORTFOLIO IS THE PRIZE.</p>
      </div>
    {/if}

    <button class="enter" on:click|stopPropagation={() => dispatch('start')}>
      <span class="enter-bracket">[</span>{resumed ? 'RESUME RUN' : isTouch ? 'TAP TO DEPLOY' : 'CLICK TO DEPLOY'}<span class="enter-bracket">]</span>
    </button>
    {#if !resumed}
      <div class="daily">
        <button class="daily-btn" on:click|stopPropagation={() => dispatch('startdaily')}>
          <span>DAILY CHALLENGE</span> <em>· {today}</em>
        </button>
        <p class="daily-mods">
          {#each dailyMutators as m, i}<span class="mod">{m.label}</span>{#if i < dailyMutators.length - 1}<span class="plus">+</span>{/if}{/each}
        </p>
      </div>
    {/if}
    {#if resumed}
      <div class="menu">
        <button class="menu-item" on:click|stopPropagation={() => dispatch('togglecycle')}>
          DAY/NIGHT CYCLE · {dayNightCycle ? 'ON' : 'OFF'}
        </button>
        <button class="menu-item" on:click|stopPropagation={() => dispatch('mute')}>
          SOUND · {muted ? 'OFF' : 'ON'}
        </button>
      </div>
    {/if}
    <button class="text-link" on:click|stopPropagation={() => dispatch('textmode')}>
      exit to the regular site
    </button>
  </div>
</div>

<style>
  .start {
    position: absolute;
    inset: 0;
    z-index: 25;
    display: flex;
    align-items: center;
    justify-content: center;
    isolation: isolate;
    overflow: hidden;
    background:
      radial-gradient(circle at 50% 42%, rgba(62, 135, 176, 0.18), transparent 34%),
      linear-gradient(115deg, rgba(4, 10, 20, 0.86), rgba(8, 8, 14, 0.62) 52%, rgba(25, 7, 20, 0.8));
    backdrop-filter: blur(5px) saturate(1.15);
  }

  .grid {
    position: absolute;
    z-index: -1;
    inset: -30%;
    opacity: 0.24;
    background-image:
      linear-gradient(rgba(122, 220, 255, 0.22) 1px, transparent 1px),
      linear-gradient(90deg, rgba(122, 220, 255, 0.22) 1px, transparent 1px);
    background-size: 42px 42px;
    transform: perspective(450px) rotateX(61deg) translateY(21%);
    transform-origin: center bottom;
    animation: grid-drift 9s linear infinite;
  }

  .radar {
    position: absolute;
    z-index: -1;
    width: min(70vw, 720px);
    aspect-ratio: 1;
    border: 1px solid rgba(88, 224, 255, 0.16);
    border-radius: 50%;
    background: repeating-radial-gradient(circle, transparent 0 18%, rgba(88, 224, 255, 0.08) 18.2% 18.5%);
    box-shadow: inset 0 0 80px rgba(24, 152, 223, 0.09), 0 0 80px rgba(28, 150, 255, 0.08);
  }

  .radar::after {
    content: '';
    position: absolute;
    inset: 50%;
    width: 50%;
    height: 1px;
    transform-origin: left center;
    background: linear-gradient(90deg, rgba(115, 255, 227, 0.6), transparent);
    box-shadow: 0 0 12px rgba(115, 255, 227, 0.7);
    animation: sweep 5s linear infinite;
  }

  .radar-one { top: -28%; left: -16%; }
  .radar-two { right: -28%; bottom: -42%; opacity: 0.56; transform: scale(0.78); }

  .corner {
    position: absolute;
    width: 82px;
    height: 82px;
    border-color: rgba(255, 210, 63, 0.76);
    pointer-events: none;
  }
  .corner-tl { top: 22px; left: 22px; border-top: 2px solid; border-left: 2px solid; }
  .corner-br { right: 22px; bottom: 22px; border-right: 2px solid; border-bottom: 2px solid; }

  .inner {
    position: relative;
    text-align: center;
    padding: 30px 24px 24px;
    max-width: 850px;
    animation: boot-in 0.7s var(--ease-out-expo, cubic-bezier(0.16, 1, 0.3, 1)) both;
  }

  .status-line,
  .eyebrow {
    font-family: var(--font-mono, monospace);
    font-size: 0.65rem;
    letter-spacing: 0.18em;
    color: rgba(190, 237, 255, 0.68);
  }

  .status-line { display: flex; align-items: center; justify-content: center; gap: 9px; }
  .status-dot { width: 7px; height: 7px; border-radius: 50%; background: #65f7b4; box-shadow: 0 0 10px #65f7b4; animation: signal 1.2s ease-in-out infinite; }
  .status-divider { color: rgba(255, 255, 255, 0.25); }
  .eyebrow { margin-top: clamp(22px, 4vh, 42px); color: #ffd23f; }

  .title {
    font-family: var(--font-display, monospace);
    font-size: clamp(2.6rem, 8vw, 5.5rem);
    line-height: 0.9;
    letter-spacing: 0.03em;
    text-transform: uppercase;
    color: #f6f5eb;
    text-shadow: 4px 4px 0 rgba(0, 0, 0, 0.45), 0 0 32px rgba(77, 196, 255, 0.22);
    margin-top: 8px;
  }
  .title span { color: #ffd23f; }

  .tagline {
    font-family: var(--font-body, serif);
    font-size: clamp(0.95rem, 1.6vw, 1.1rem);
    line-height: 1.5;
    color: rgba(255, 255, 255, 0.85);
    margin: 14px auto 0;
    max-width: 610px;
  }

  .mission-grid {
    display: grid;
    grid-template-columns: repeat(3, 1fr);
    gap: 10px;
    margin: 25px auto 0;
    max-width: 680px;
  }

  .mission-card {
    position: relative;
    min-height: 106px;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    border: 1px solid rgba(177, 224, 255, 0.28);
    background: linear-gradient(135deg, rgba(10, 25, 42, 0.74), rgba(15, 11, 23, 0.5));
    box-shadow: inset 0 1px rgba(255, 255, 255, 0.05);
    transition: transform 160ms ease, border-color 160ms ease, background 160ms ease;
  }
  .mission-card:hover { transform: translateY(-4px); border-color: rgba(106, 232, 255, 0.75); background: rgba(13, 37, 59, 0.82); }
  .mission-card.hot { border-color: rgba(255, 210, 63, 0.48); }
  .mission-number { position: absolute; top: 8px; left: 10px; font: 0.58rem var(--font-mono, monospace); color: rgba(255, 255, 255, 0.35); }
  .mission-icon { color: #72dcff; font-family: var(--font-display, monospace); font-size: 1.45rem; line-height: 1; }
  .hot .mission-icon { color: #ffd23f; }
  .mission-card strong { margin-top: 7px; font: 1.15rem var(--font-display, monospace); letter-spacing: 0.13em; color: #fff; }
  .mission-card small { margin-top: 2px; font: 0.56rem var(--font-mono, monospace); letter-spacing: 0.08em; color: rgba(255, 255, 255, 0.55); }

  .how {
    margin-top: 18px;
    font-family: var(--font-mono, monospace);
    font-size: 0.7rem;
    letter-spacing: 0.12em;
    color: rgba(255, 255, 255, 0.65);
  }
  .how b { font-weight: 400; color: #72dcff; }

  .goal {
    margin-top: 8px;
    color: rgba(255, 255, 255, 0.5);
  }

  .enter {
    display: inline-block;
    margin-top: 26px;
    background: linear-gradient(90deg, #ffd23f, #fff1a6, #ffd23f);
    background-size: 200% 100%;
    border: 1px solid #fff3a6;
    font-family: var(--font-display, monospace);
    font-size: 1.45rem;
    letter-spacing: 0.12em;
    color: #10111a;
    padding: 12px 27px 10px;
    cursor: var(--cursor-pointer);
    box-shadow: 0 0 0 4px rgba(255, 210, 63, 0.11), 0 12px 38px rgba(0, 0, 0, 0.32);
    animation: shimmer 2.6s linear infinite;
    transition: transform 150ms ease, box-shadow 150ms ease;
  }
  .enter:hover { transform: translateY(-3px) scale(1.03); box-shadow: 0 0 0 6px rgba(255, 210, 63, 0.17), 0 16px 45px rgba(0, 0, 0, 0.42); }
  .enter-bracket { color: rgba(16, 17, 26, 0.55); padding: 0 7px; }

  .daily {
    margin-top: 22px;
  }

  .daily-btn {
    background: rgba(7, 13, 24, 0.52);
    border: 1px solid rgba(255, 210, 63, 0.72);
    font-family: var(--font-display, monospace);
    font-size: 1.05rem;
    letter-spacing: 0.1em;
    color: #ffd23f;
    padding: 8px 18px;
    cursor: var(--cursor-pointer);
  }

  .daily-btn:hover {
    background: rgba(255, 210, 63, 0.18);
  }
  .daily-btn em { color: rgba(255, 255, 255, 0.66); font-style: normal; }

  .daily-mods {
    margin-top: 10px;
    font-family: var(--font-mono, monospace);
    font-size: 0.68rem;
    letter-spacing: 0.1em;
    color: rgba(255, 255, 255, 0.65);
  }

  .daily-mods .mod {
    color: #ffd23f;
  }

  .daily-mods .plus {
    margin: 0 6px;
    color: rgba(255, 255, 255, 0.4);
  }

  .menu {
    display: flex;
    flex-direction: column;
    gap: 10px;
    margin-top: 26px;
    align-items: center;
  }

  .menu-item {
    background: rgba(0, 0, 0, 0.35);
    border: 1px solid rgba(255, 255, 255, 0.4);
    font-family: var(--font-mono, monospace);
    font-size: 0.78rem;
    letter-spacing: 0.12em;
    color: rgba(255, 255, 255, 0.85);
    padding: 9px 22px;
    min-width: 240px;
    cursor: var(--cursor-pointer);
  }

  .menu-item:hover {
    border-color: #ffd23f;
    color: #ffd23f;
  }

  .text-link {
    display: block;
    margin: 22px auto 0;
    background: none;
    border: none;
    font-family: var(--font-mono, monospace);
    font-size: 0.7rem;
    letter-spacing: 0.1em;
    color: rgba(255, 255, 255, 0.5);
    text-decoration: underline;
    cursor: var(--cursor-pointer);
  }

  .text-link:hover {
    color: #fff;
  }

  @keyframes blink {
    0%, 100% { opacity: 1; }
    50% { opacity: 0.55; }
  }

  @keyframes boot-in { from { opacity: 0; transform: translateY(16px) scale(0.98); } to { opacity: 1; transform: none; } }
  @keyframes signal { 50% { opacity: 0.35; transform: scale(0.75); } }
  @keyframes sweep { to { transform: rotate(360deg); } }
  @keyframes grid-drift { to { background-position: 0 42px, 42px 0; } }
  @keyframes shimmer { to { background-position: -200% 0; } }

  @media (prefers-reduced-motion: reduce) {
    .grid, .radar::after, .status-dot, .inner, .enter { animation: none; }
  }

  @media (max-width: 600px) {
    .corner { width: 42px; height: 42px; }
    .corner-tl { top: 12px; left: 12px; }
    .corner-br { right: 12px; bottom: 12px; }
    .status-line { font-size: 0.52rem; letter-spacing: 0.1em; }
    .status-divider { display: none; }
    .mission-grid { gap: 6px; margin-top: 20px; }
    .mission-card { min-height: 88px; }
    .mission-card strong { font-size: 0.95rem; }
    .mission-card small { font-size: 0.49rem; letter-spacing: 0.03em; }
    .mission-icon { font-size: 1.15rem; }
    .how { font-size: 0.55rem; line-height: 1.65; }
    .enter { margin-top: 21px; font-size: 1.25rem; }
  }
</style>
