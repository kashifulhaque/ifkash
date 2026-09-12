<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import type { BiomeCaption } from '../store';

  export let found = 0;
  export let total = 7;
  /** Starlight shards collected on this planet. */
  export let shards: { found: number; total: number } = { found: 0, total: 0 };
  export let biome: BiomeCaption | null = null;
  export let prompt: { id: string; action: string; found: boolean; kicker?: string } | null = null;
  export let muted = false;
  export let globeView = false;
  export let intro = true;
  export let isTouch = false;

  const dispatch = createEventDispatcher();
</script>

<div class="hud" aria-live="polite">
  <div class="brand">
    <svg viewBox="0 0 32 32" aria-hidden="true">
      <circle cx="16" cy="16" r="7" />
      <ellipse cx="16" cy="16" rx="14" ry="4.5" transform="rotate(-25 16 16)" />
      <circle cx="26.5" cy="9" r="1.4" class="dot" />
    </svg>
    <div>
      <p class="brand-name">Tiny Planet</p>
      <p class="brand-sub">A pocket portfolio</p>
    </div>
  </div>

  <div class="counter" class:complete={found >= total}>
    <svg viewBox="0 0 24 24" aria-hidden="true">
      <path d="M4 5.5A1.5 1.5 0 0 1 5.5 4H11v16H5.5A1.5 1.5 0 0 1 4 18.5z" />
      <path d="M13 4h5.5A1.5 1.5 0 0 1 20 5.5v13a1.5 1.5 0 0 1-1.5 1.5H13z" />
    </svg>
    {#key found}
      <strong class="count">{found}</strong>
    {/key}
    <span class="of">/ {total}</span>
    <span class="label">small wonders</span>
    {#if shards.total > 0}
      <span class="divider"></span>
      <span class="shards" class:complete={shards.found >= shards.total} title="Starlight shards on this planet">
        <span class="shard-icon">✦</span>
        {#key shards.found}
          <strong class="count">{shards.found}</strong>
        {/key}
        <span class="of">/ {shards.total}</span>
      </span>
    {/if}
  </div>

  <div class="actions">
    <button class="round" class:active={globeView} on:click={() => dispatch('globe')} aria-label={globeView ? 'Return to the ground' : 'View the whole planet'} title="Globe view (M)">
      <svg viewBox="0 0 24 24" aria-hidden="true">
        <circle cx="12" cy="12" r="9" />
        <path d="M3 12h18M12 3c3 3.5 3 14.5 0 18M12 3c-3 3.5-3 14.5 0 18" />
      </svg>
    </button>
    <button class="round" on:click={() => dispatch('mute')} aria-label={muted ? 'Unmute' : 'Mute'} title={muted ? 'Unmute' : 'Mute'}>
      {#if muted}
        <svg viewBox="0 0 24 24" aria-hidden="true">
          <path d="M4 9v6h4l5 4V5L8 9z" />
          <path d="M17 9l4 6M21 9l-4 6" />
        </svg>
      {:else}
        <svg viewBox="0 0 24 24" aria-hidden="true">
          <path d="M4 9v6h4l5 4V5L8 9z" />
          <path d="M16.5 8.5a5 5 0 0 1 0 7M19 6a8.5 8.5 0 0 1 0 12" />
        </svg>
      {/if}
    </button>
    <button class="round" on:click={() => dispatch('photo')} aria-label="Photo mode" title="Photo mode (P)">
      <svg viewBox="0 0 24 24" aria-hidden="true">
        <path d="M4 8.5A1.5 1.5 0 0 1 5.5 7H8l1.5-2h5L16 7h2.5A1.5 1.5 0 0 1 20 8.5v9a1.5 1.5 0 0 1-1.5 1.5h-13A1.5 1.5 0 0 1 4 17.5z" />
        <circle cx="12" cy="13" r="3.2" />
      </svg>
    </button>
    <button class="round" on:click={() => dispatch('journal')} aria-label="Journal" title="Journal (J)">
      <svg viewBox="0 0 24 24" aria-hidden="true">
        <path d="M5 4.5A1.5 1.5 0 0 1 6.5 3H18a1 1 0 0 1 1 1v14.5a1.5 1.5 0 0 1-1.5 1.5H6.5A1.5 1.5 0 0 1 5 18.5z" />
        <path d="M5 17.5A1.5 1.5 0 0 1 6.5 16H19M9 7.5h6M9 10.5h4" />
      </svg>
    </button>
    <button class="round" on:click={() => dispatch('help')} aria-label="Help" title="Help (H)">
      <svg viewBox="0 0 24 24" aria-hidden="true">
        <circle cx="12" cy="12" r="9" />
        <path d="M9.5 9.5a2.5 2.5 0 1 1 3.6 2.2c-.8.5-1.1 1-1.1 1.8M12 17h.01" />
      </svg>
    </button>
  </div>

  {#if biome && !intro}
    {#key biome.name}
      <div class="caption">
        <p class="kicker">{biome.kind} · {biome.index}</p>
        <h2>{biome.name}</h2>
        <p class="tagline">{biome.tagline}</p>
      </div>
    {/key}
  {/if}

  <div class="bottom">
    {#if prompt}
      <button class="prompt" on:click={() => dispatch('interact')}>
        <span class="key">E</span>
        <span class="prompt-text">
          <span class="kicker">{prompt.kicker ?? (prompt.found ? 'Found · visit again' : 'A small wonder')}</span>
          <span class="action">{prompt.action}</span>
        </span>
        <svg viewBox="0 0 24 24" aria-hidden="true"><path d="M9 6l6 6-6 6" /></svg>
      </button>
    {/if}

    {#if intro}
      {#if isTouch}
        <p class="hint-line"><span class="spark">✦</span> Tap to wander <span class="sep">·</span> drag to orbit <span class="sep">·</span> pinch to zoom</p>
      {:else}
        <p class="hint-line"><span class="spark">✦</span> Click anywhere to wander <span class="sep">|</span> Drag to orbit <span class="sep">·</span> Scroll to zoom</p>
      {/if}
    {/if}

    {#if !isTouch}
      <div class="hints">
        <span class="group"><kbd>W</kbd><kbd>A</kbd><kbd>S</kbd><kbd>D</kbd> walk</span>
        <span class="group"><kbd>⇧</kbd> run</span>
        <span class="group"><kbd>space</kbd> hop</span>
        <span class="group"><kbd>E</kbd> interact</span>
        <span class="group"><kbd>M</kbd> globe</span>
        <span class="group"><kbd>J</kbd> journal</span>
      </div>
    {:else if !intro}
      <p class="hint-line small">Tap a spot to wander · drag to look around</p>
    {/if}
  </div>
</div>

<style>
  .hud {
    position: absolute;
    inset: 0;
    pointer-events: none;
    color: #f2efe6;
    font-family: var(--planet-sans);
    --glass: rgba(10, 24, 34, 0.55);
    --glass-border: rgba(255, 255, 255, 0.12);
  }

  .hud > * {
    pointer-events: auto;
  }

  .kicker {
    font-size: 0.62rem;
    letter-spacing: 0.2em;
    text-transform: uppercase;
    color: rgba(242, 239, 230, 0.6);
    margin: 0;
  }

  /* Brand */
  .brand {
    position: absolute;
    top: 18px;
    left: 22px;
    display: flex;
    align-items: center;
    gap: 10px;
  }
  .brand svg {
    width: 34px;
    height: 34px;
    fill: none;
    stroke: rgba(242, 239, 230, 0.9);
    stroke-width: 1.4;
  }
  .brand svg .dot {
    fill: #e9c46a;
    stroke: none;
  }
  .brand-name {
    margin: 0;
    font-size: 0.72rem;
    letter-spacing: 0.26em;
    text-transform: uppercase;
    font-weight: 600;
  }
  .brand-sub {
    margin: 2px 0 0;
    font-size: 0.58rem;
    letter-spacing: 0.2em;
    text-transform: uppercase;
    color: rgba(242, 239, 230, 0.55);
  }

  /* Counter */
  .counter {
    position: absolute;
    top: 18px;
    left: 50%;
    transform: translateX(-50%);
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 8px 14px;
    background: var(--glass);
    border: 1px solid var(--glass-border);
    border-radius: 999px;
    backdrop-filter: blur(10px);
    font-size: 0.8rem;
  }
  .counter svg {
    width: 16px;
    height: 16px;
    fill: none;
    stroke: rgba(242, 239, 230, 0.75);
    stroke-width: 1.5;
  }
  .counter .count {
    font-weight: 600;
    animation: pop 0.5s ease-out;
    display: inline-block;
  }
  .counter .of {
    color: rgba(242, 239, 230, 0.6);
  }
  .counter .label {
    color: rgba(242, 239, 230, 0.6);
    font-size: 0.72rem;
    margin-left: 2px;
  }
  .counter.complete {
    border-color: rgba(233, 196, 106, 0.6);
  }
  .counter.complete .count {
    color: #e9c46a;
  }
  .counter .divider {
    width: 1px;
    height: 14px;
    margin: 0 2px;
    background: rgba(255, 255, 255, 0.16);
  }
  .counter .shards {
    display: flex;
    align-items: center;
    gap: 5px;
  }
  .counter .shard-icon {
    color: #c9b3ff;
    font-size: 0.7rem;
  }
  .counter .shards.complete .count {
    color: #c9b3ff;
  }

  /* Round buttons */
  .actions {
    position: absolute;
    top: 18px;
    right: 22px;
    display: flex;
    gap: 8px;
  }
  .round {
    width: 36px;
    height: 36px;
    border-radius: 50%;
    background: var(--glass);
    border: 1px solid var(--glass-border);
    backdrop-filter: blur(10px);
    display: grid;
    place-items: center;
    color: rgba(242, 239, 230, 0.85);
    transition: background 0.15s, border-color 0.15s;
  }
  .round:hover {
    background: rgba(255, 255, 255, 0.1);
    border-color: rgba(255, 255, 255, 0.28);
  }
  .round.active {
    border-color: rgba(233, 196, 106, 0.7);
    color: #e9c46a;
  }
  .round svg {
    width: 17px;
    height: 17px;
    fill: none;
    stroke: currentColor;
    stroke-width: 1.5;
    stroke-linecap: round;
    stroke-linejoin: round;
  }

  /* Biome caption */
  .caption {
    position: absolute;
    left: 24px;
    bottom: 26px;
    max-width: 260px;
    animation: rise 0.6s ease-out;
  }
  .caption h2 {
    margin: 4px 0 6px;
    font-family: var(--planet-serif);
    font-weight: 400;
    font-size: 1.9rem;
    letter-spacing: 0.01em;
    line-height: 1.1;
    color: #fbf8f0;
    text-shadow: 0 2px 14px rgba(0, 0, 0, 0.35);
  }
  .caption .tagline {
    margin: 0;
    font-size: 0.74rem;
    line-height: 1.45;
    color: rgba(242, 239, 230, 0.66);
  }

  /* Bottom stack */
  .bottom {
    position: absolute;
    left: 50%;
    bottom: 22px;
    transform: translateX(-50%);
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 12px;
  }

  .prompt {
    display: flex;
    align-items: center;
    gap: 14px;
    padding: 10px 16px 10px 12px;
    background: rgba(18, 40, 46, 0.72);
    border: 1px solid rgba(233, 196, 106, 0.35);
    border-radius: 12px;
    backdrop-filter: blur(10px);
    color: #f2efe6;
    text-align: left;
    animation: rise 0.35s ease-out;
    box-shadow: 0 10px 30px rgba(0, 0, 0, 0.3);
  }
  .prompt:hover {
    border-color: rgba(233, 196, 106, 0.7);
  }
  .prompt .key {
    width: 34px;
    height: 34px;
    border-radius: 8px;
    display: grid;
    place-items: center;
    border: 1px solid rgba(233, 196, 106, 0.7);
    color: #e9c46a;
    font-weight: 600;
    font-size: 0.85rem;
  }
  .prompt-text {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }
  .prompt .action {
    font-size: 0.95rem;
    font-weight: 500;
  }
  .prompt svg {
    width: 16px;
    height: 16px;
    fill: none;
    stroke: rgba(242, 239, 230, 0.6);
    stroke-width: 1.6;
  }

  .hint-line {
    margin: 0;
    font-size: 0.72rem;
    color: rgba(242, 239, 230, 0.7);
    letter-spacing: 0.02em;
    text-shadow: 0 1px 8px rgba(0, 0, 0, 0.5);
    animation: rise 0.6s ease-out;
    text-align: center;
  }
  .hint-line.small {
    font-size: 0.66rem;
    color: rgba(242, 239, 230, 0.5);
  }
  .hint-line .spark {
    color: #e9c46a;
  }
  .hint-line .sep {
    margin: 0 6px;
    color: rgba(242, 239, 230, 0.35);
  }

  .hints {
    display: flex;
    gap: 14px;
    padding: 7px 14px;
    background: var(--glass);
    border: 1px solid var(--glass-border);
    border-radius: 999px;
    backdrop-filter: blur(10px);
    font-size: 0.66rem;
    color: rgba(242, 239, 230, 0.6);
  }
  .hints .group {
    display: flex;
    align-items: center;
    gap: 4px;
  }
  kbd {
    font-family: var(--planet-sans);
    font-size: 0.6rem;
    line-height: 1;
    padding: 4px 5px;
    min-width: 18px;
    text-align: center;
    border: 1px solid rgba(255, 255, 255, 0.22);
    border-radius: 4px;
    color: rgba(242, 239, 230, 0.85);
    background: rgba(255, 255, 255, 0.04);
  }

  @keyframes rise {
    from {
      opacity: 0;
      transform: translateY(8px);
    }
    to {
      opacity: 1;
      transform: none;
    }
  }
  @keyframes pop {
    0% {
      transform: scale(1);
    }
    40% {
      transform: scale(1.35);
      color: #e9c46a;
    }
    100% {
      transform: scale(1);
    }
  }

  @media (max-width: 640px) {
    .brand-name,
    .brand-sub,
    .counter .label {
      display: none;
    }
    .brand {
      top: 12px;
      left: 12px;
    }
    .brand svg {
      width: 26px;
      height: 26px;
    }
    .counter {
      top: 56px;
      padding: 6px 12px;
    }
    .actions {
      top: 12px;
      right: 12px;
    }
    .round {
      width: 32px;
      height: 32px;
    }
    .caption {
      left: 14px;
      top: 100px;
      bottom: auto;
      max-width: 220px;
    }
    .caption h2 {
      font-size: 1.4rem;
    }
    .bottom {
      bottom: 172px;
      width: calc(100% - 28px);
    }
  }
</style>
