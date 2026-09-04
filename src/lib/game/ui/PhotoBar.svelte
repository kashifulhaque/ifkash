<script lang="ts">
  import { createEventDispatcher } from 'svelte';

  export let isTouch = false;
  export let saving = false;

  const dispatch = createEventDispatcher<{ exit: void; save: void }>();
</script>

<div class="photo">
  <p class="hint" aria-live="polite">
    {#if isTouch}
      Photo mode · use the corner buttons to save or leave
    {:else}
      Photo mode · <kbd>P</kbd> or <kbd>Esc</kbd> to leave
    {/if}
  </p>

  <div class="tools">
    <button class="round" on:click={() => dispatch('save')} disabled={saving} aria-label="Save a picture" title="Save a picture">
      <svg viewBox="0 0 24 24" aria-hidden="true">
        <path d="M12 4v11M7.5 10.5L12 15l4.5-4.5M5 18.5h14" />
      </svg>
    </button>
    <button class="round" on:click={() => dispatch('exit')} aria-label="Leave photo mode" title="Leave photo mode (P)">
      <svg viewBox="0 0 24 24" aria-hidden="true"><path d="M6 6l12 12M18 6L6 18" /></svg>
    </button>
  </div>
</div>

<style>
  .photo {
    position: absolute;
    inset: 0;
    pointer-events: none;
    z-index: 20;
    color: #f2efe6;
    font-family: var(--planet-sans);
  }
  .hint {
    position: absolute;
    left: 50%;
    bottom: 26px;
    transform: translateX(-50%);
    margin: 0;
    padding: 8px 14px;
    border-radius: 999px;
    background: rgba(10, 24, 34, 0.55);
    border: 1px solid rgba(255, 255, 255, 0.12);
    backdrop-filter: blur(10px);
    font-size: 0.72rem;
    letter-spacing: 0.02em;
    color: rgba(242, 239, 230, 0.8);
    white-space: nowrap;
    animation: fadeout 2.6s ease-in forwards;
  }
  kbd {
    font-family: var(--planet-sans);
    font-size: 0.6rem;
    line-height: 1;
    padding: 3px 5px;
    border: 1px solid rgba(255, 255, 255, 0.22);
    border-radius: 4px;
    background: rgba(255, 255, 255, 0.04);
  }
  .tools {
    position: absolute;
    right: 22px;
    bottom: 22px;
    display: flex;
    gap: 8px;
    pointer-events: auto;
    opacity: 0.55;
    transition: opacity 0.2s;
  }
  .tools:hover,
  .tools:focus-within {
    opacity: 1;
  }
  .round {
    width: 36px;
    height: 36px;
    border-radius: 50%;
    background: rgba(10, 24, 34, 0.55);
    border: 1px solid rgba(255, 255, 255, 0.12);
    backdrop-filter: blur(10px);
    display: grid;
    place-items: center;
    color: rgba(242, 239, 230, 0.85);
  }
  .round:hover {
    background: rgba(255, 255, 255, 0.1);
    border-color: rgba(255, 255, 255, 0.28);
  }
  .round:disabled {
    opacity: 0.5;
    cursor: wait;
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
  @keyframes fadeout {
    0%,
    70% {
      opacity: 1;
    }
    100% {
      opacity: 0;
    }
  }
  @media (max-width: 640px) {
    .tools {
      opacity: 0.85;
    }
  }
</style>
