<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import { name, tagline } from '$lib/content';

  export let isTouch: boolean;
  export let resumed = false; // true when pointer lock was lost mid-game
  export let muted = false;
  export let dayNightCycle = false;

  const dispatch = createEventDispatcher();
</script>

<div class="start" role="button" tabindex="0"
  on:click={() => dispatch('start')}
  on:keydown={(e) => (e.key === 'Enter' || e.key === ' ') && dispatch('start')}
>
  <div class="inner">
    {#if !resumed}
      <h1 class="title">{name}</h1>
      <p class="tagline">{tagline}</p>
      <div class="how">
        {#if isTouch}
          <p>LEFT THUMB · MOVE — RIGHT THUMB · LOOK — FIRE · SHOOT — JUMP / DUCK</p>
        {:else}
          <p>WASD · MOVE — SHIFT · RUN — SPACE · JUMP — C · CROUCH — CLICK · SHOOT — E · LOOT</p>
        {/if}
        <p class="goal">SHOOT THE TARGETS. LOOT THE CRATES. WATCH OUT — THEY SHOOT BACK.</p>
      </div>
    {/if}
    <span class="enter">{resumed ? 'CLICK TO RESUME' : isTouch ? 'TAP TO ENTER' : 'CLICK TO ENTER'}</span>
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
    background: rgba(10, 10, 14, 0.55);
    backdrop-filter: blur(4px);
    cursor: pointer;
  }

  .inner {
    text-align: center;
    padding: 24px;
    max-width: 720px;
  }

  .title {
    font-family: var(--font-display, monospace);
    font-size: clamp(2.6rem, 8vw, 5.5rem);
    line-height: 0.9;
    letter-spacing: 0.03em;
    text-transform: uppercase;
    color: #ffd23f;
    text-shadow: 4px 4px 0 rgba(0, 0, 0, 0.45);
  }

  .tagline {
    font-family: var(--font-body, serif);
    font-size: clamp(0.95rem, 1.6vw, 1.1rem);
    line-height: 1.5;
    color: rgba(255, 255, 255, 0.85);
    margin: 18px auto 0;
    max-width: 560px;
  }

  .how {
    margin-top: 26px;
    font-family: var(--font-mono, monospace);
    font-size: 0.7rem;
    letter-spacing: 0.12em;
    color: rgba(255, 255, 255, 0.6);
  }

  .goal {
    margin-top: 8px;
    color: rgba(255, 255, 255, 0.45);
  }

  .enter {
    display: inline-block;
    margin-top: 30px;
    font-family: var(--font-display, monospace);
    font-size: 1.5rem;
    letter-spacing: 0.1em;
    color: #fff;
    border: 2px solid #fff;
    padding: 10px 28px;
    animation: blink 1.4s ease-in-out infinite;
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
    cursor: pointer;
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
    cursor: pointer;
  }

  .text-link:hover {
    color: #fff;
  }

  @keyframes blink {
    0%, 100% { opacity: 1; }
    50% { opacity: 0.55; }
  }
</style>
