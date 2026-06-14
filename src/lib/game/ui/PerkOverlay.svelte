<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import type { Perk } from '$lib/game/perks';

  export let perks: Perk[] = [];
  export let wave = 1;

  const dispatch = createEventDispatcher<{ choose: Perk }>();
</script>

<div class="overlay" role="dialog" aria-label="Choose a perk">
  <div class="inner">
    <p class="tag">WAVE {wave} · CHOOSE A PERK</p>
    <div class="cards">
      {#each perks as perk}
        <button class="card" on:click={() => dispatch('choose', perk)}>
          <span class="card-title">{perk.label}</span>
          <span class="card-desc">{perk.desc}</span>
        </button>
      {/each}
    </div>
  </div>
</div>

<style>
  .overlay {
    position: absolute;
    inset: 0;
    z-index: 28;
    display: flex;
    align-items: center;
    justify-content: center;
    background: rgba(10, 10, 14, 0.72);
    backdrop-filter: blur(4px);
  }

  .inner {
    text-align: center;
    padding: 24px;
    max-width: 820px;
    width: 100%;
  }

  .tag {
    font-family: var(--font-display, monospace);
    font-size: 1.2rem;
    letter-spacing: 0.16em;
    color: #ffd23f;
    text-shadow: 0 2px 6px rgba(0, 0, 0, 0.6);
    margin-bottom: 22px;
  }

  .cards {
    display: flex;
    gap: 16px;
    justify-content: center;
    flex-wrap: wrap;
  }

  .card {
    flex: 1 1 200px;
    max-width: 240px;
    min-height: 150px;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 12px;
    padding: 22px 18px;
    background: rgba(0, 0, 0, 0.45);
    border: 2px solid rgba(255, 255, 255, 0.35);
    cursor: pointer;
    transition: border-color 0.15s, transform 0.15s, background 0.15s;
  }

  .card:hover,
  .card:focus-visible {
    border-color: #ffd23f;
    background: rgba(255, 210, 63, 0.08);
    transform: translateY(-3px);
    outline: none;
  }

  .card-title {
    font-family: var(--font-display, monospace);
    font-size: 1.25rem;
    letter-spacing: 0.08em;
    color: #fff;
  }

  .card-desc {
    font-family: var(--font-mono, monospace);
    font-size: 0.82rem;
    line-height: 1.4;
    color: rgba(255, 255, 255, 0.7);
  }

  @media (max-width: 600px) {
    .cards {
      flex-direction: column;
      align-items: stretch;
    }
    .card {
      max-width: none;
      min-height: 0;
      flex-direction: row;
      justify-content: flex-start;
      gap: 14px;
      text-align: left;
      padding: 16px;
    }
    .card-title {
      font-size: 1.05rem;
      flex: 0 0 38%;
    }
  }
</style>
