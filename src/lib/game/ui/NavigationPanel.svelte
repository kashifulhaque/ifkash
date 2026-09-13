<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import { MAX_FUEL, SHIP_PARTS, type SpaceDestination, type SpaceStatus } from '../space';

  export let space: SpaceStatus = {
    parts: 0,
    totalParts: SHIP_PARTS.length,
    crafted: false,
    fuel: 0,
    maxFuel: MAX_FUEL,
    planetName: 'Earth',
    planetKind: 'Homeworld',
    depth: 0,
    isEarth: true
  };
  export let destinations: SpaceDestination[] = [];

  const dispatch = createEventDispatcher<{
    travel: SpaceDestination;
    earth: void;
    close: void;
  }>();

  $: boundedFuel = Math.min(Math.max(space.fuel, 0), Math.max(space.maxFuel, 0));

</script>

<div class="overlay" role="dialog" aria-modal="true" aria-label="Star map">
  <button class="backdrop" on:click={() => dispatch('close')} aria-label="Close star map"></button>
  <section class="panel">
    <header>
      <div>
        <p class="kicker">Flight computer · depth {space.depth}</p>
        <h2>Choose a destination</h2>
      </div>
      <button class="close" on:click={() => dispatch('close')} aria-label="Close star map" title="Close">
        <svg viewBox="0 0 24 24" aria-hidden="true"><path d="m6 6 12 12M18 6 6 18" /></svg>
      </button>
    </header>

    <div class="fuel" aria-label="Available ship fuel">
      <div>
        <span>Fuel reserves</span>
        <strong>{boundedFuel} / {space.maxFuel}</strong>
      </div>
      <progress value={boundedFuel} max={Math.max(space.maxFuel, 1)} aria-label={`Ship fuel: ${boundedFuel} of ${space.maxFuel}`}></progress>
    </div>

    <p class="intro">Every flight reaches one layer deeper into procedurally generated space. Pick a route your reserves can cover.</p>

    <div class="routes" aria-label="Generated destinations">
      {#each destinations as destination (destination.seed)}
        {@const affordable = destination.fuelCost <= boundedFuel}
        {@const routeColor = `#${(destination.color & 0xffffff).toString(16).padStart(6, '0')}`}
        {@const routeKind = destination.kind.replace(/[-_]/g, ' ')}
        <article class:unavailable={!affordable}>
          <span class="planet" style:background={routeColor} aria-hidden="true"></span>
          <div class="route-copy">
            <div class="route-title">
              <h3>{destination.name}</h3>
              <span>Depth {destination.depth}</span>
            </div>
            <p>{routeKind}</p>
          </div>
          <div class="route-action">
            <span class="cost">{destination.fuelCost} fuel</span>
            <button
              disabled={!affordable}
              on:click={() => dispatch('travel', destination)}
              aria-label={affordable
                ? `Travel to ${destination.name} for ${destination.fuelCost} fuel`
                : `Cannot travel to ${destination.name}; requires ${destination.fuelCost} fuel`}
              title={affordable ? `Travel to ${destination.name}` : `Need ${destination.fuelCost - boundedFuel} more fuel`}
            >{affordable ? 'Travel' : 'Insufficient fuel'}</button>
          </div>
        </article>
      {:else}
        <p class="empty">No flight paths are available yet.</p>
      {/each}
    </div>

    {#if !space.isEarth}
      <div class="earth-route">
        <div>
          <p class="kicker">Emergency recall</p>
          <strong>Earth</strong>
          <span>Teleport home without spending fuel.</span>
        </div>
        <button on:click={() => dispatch('earth')} aria-label="Teleport home to Earth for free">Return to Earth · free</button>
      </div>
    {/if}

    <p class="footnote">New destinations are generated after every flight. There is no final depth.</p>
  </section>
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
    color: #f2efe6;
    font-family: var(--planet-sans);
  }
  .backdrop {
    position: absolute;
    inset: 0;
    border: 0;
    background: rgba(6, 16, 24, 0.62);
    backdrop-filter: blur(5px);
  }
  .panel {
    position: relative;
    width: min(620px, 100%);
    max-height: min(88vh, 88dvh);
    overflow-y: auto;
    padding: 25px 27px 22px;
    border: 1px solid rgba(255, 255, 255, 0.12);
    border-radius: 18px;
    background: rgba(14, 30, 40, 0.95);
    box-shadow: 0 30px 80px rgba(0, 0, 0, 0.5);
    animation: pop 0.3s cubic-bezier(0.2, 0.8, 0.2, 1);
  }
  header,
  .fuel > div,
  .route-title,
  .route-action,
  .earth-route {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 14px;
  }
  .kicker {
    margin: 0;
    color: #e9c46a;
    font-size: 0.62rem;
    letter-spacing: 0.2em;
    text-transform: uppercase;
  }
  h2 {
    margin: 7px 0 0;
    color: #fbf8f0;
    font-family: var(--planet-serif);
    font-size: 1.85rem;
    font-weight: 400;
    line-height: 1.1;
  }
  .close {
    flex: none;
    width: 36px;
    height: 36px;
    display: grid;
    place-items: center;
    border: 1px solid rgba(255, 255, 255, 0.14);
    border-radius: 50%;
    background: rgba(255, 255, 255, 0.04);
    color: rgba(242, 239, 230, 0.75);
  }
  .close:hover {
    border-color: rgba(255, 255, 255, 0.3);
    background: rgba(255, 255, 255, 0.09);
  }
  .close svg {
    width: 17px;
    fill: none;
    stroke: currentColor;
    stroke-linecap: round;
    stroke-width: 1.6;
  }
  .fuel {
    margin-top: 20px;
    padding: 11px 13px;
    border: 1px solid rgba(255, 255, 255, 0.1);
    border-radius: 11px;
    background: rgba(255, 255, 255, 0.03);
  }
  .fuel span,
  .fuel strong {
    font-size: 0.7rem;
  }
  .fuel span {
    color: rgba(242, 239, 230, 0.62);
    letter-spacing: 0.08em;
    text-transform: uppercase;
  }
  .fuel strong {
    color: #e9c46a;
    font-weight: 600;
  }
  progress {
    display: block;
    width: 100%;
    height: 5px;
    margin-top: 8px;
    overflow: hidden;
    border: 0;
    border-radius: 999px;
    background: rgba(255, 255, 255, 0.09);
    color: #e9c46a;
  }
  progress::-webkit-progress-bar {
    border-radius: 999px;
    background: rgba(255, 255, 255, 0.09);
  }
  progress::-webkit-progress-value {
    border-radius: 999px;
    background: #e9c46a;
  }
  progress::-moz-progress-bar {
    border-radius: 999px;
    background: #e9c46a;
  }
  .intro {
    margin: 15px 0;
    color: rgba(242, 239, 230, 0.6);
    font-size: 0.77rem;
    line-height: 1.55;
  }
  .routes {
    display: grid;
    gap: 9px;
  }
  article {
    display: grid;
    grid-template-columns: auto minmax(0, 1fr) auto;
    align-items: center;
    gap: 13px;
    padding: 11px 12px;
    border: 1px solid rgba(255, 255, 255, 0.1);
    border-radius: 12px;
    background: rgba(255, 255, 255, 0.025);
  }
  article.unavailable {
    border-color: rgba(255, 255, 255, 0.06);
    background: rgba(0, 0, 0, 0.08);
  }
  article.unavailable .planet,
  article.unavailable .route-copy {
    opacity: 0.48;
  }
  .planet {
    width: 34px;
    height: 34px;
    border: 1px solid rgba(255, 255, 255, 0.2);
    border-radius: 50%;
    box-shadow: inset -8px -5px 12px rgba(0, 0, 0, 0.26), 0 0 16px rgba(255, 255, 255, 0.06);
  }
  .route-copy {
    min-width: 0;
  }
  .route-title {
    justify-content: flex-start;
    gap: 8px;
  }
  h3 {
    margin: 0;
    overflow: hidden;
    color: #fbf8f0;
    font-family: var(--planet-serif);
    font-size: 1.05rem;
    font-weight: 400;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .route-title span {
    flex: none;
    padding: 2px 6px;
    border: 1px solid rgba(255, 255, 255, 0.1);
    border-radius: 999px;
    color: rgba(242, 239, 230, 0.5);
    font-size: 0.56rem;
    letter-spacing: 0.08em;
    text-transform: uppercase;
  }
  .route-copy p {
    margin: 3px 0 0;
    color: rgba(242, 239, 230, 0.52);
    font-size: 0.66rem;
    text-transform: capitalize;
  }
  .route-action {
    flex-direction: column;
    align-items: flex-end;
    gap: 5px;
  }
  .cost {
    color: rgba(242, 239, 230, 0.58);
    font-size: 0.64rem;
  }
  .route-action button,
  .earth-route button {
    padding: 7px 12px;
    border: 1px solid rgba(233, 196, 106, 0.52);
    border-radius: 999px;
    background: rgba(233, 196, 106, 0.1);
    color: #fbf1d3;
    font: inherit;
    font-size: 0.69rem;
    font-weight: 500;
    white-space: nowrap;
  }
  .route-action button:hover:not(:disabled),
  .earth-route button:hover {
    border-color: rgba(233, 196, 106, 0.8);
    background: rgba(233, 196, 106, 0.2);
  }
  .route-action button:disabled {
    cursor: not-allowed;
    border-color: rgba(255, 255, 255, 0.1);
    background: rgba(255, 255, 255, 0.025);
    color: rgba(242, 239, 230, 0.36);
  }
  .empty {
    margin: 8px 0;
    padding: 18px;
    border: 1px dashed rgba(255, 255, 255, 0.12);
    border-radius: 12px;
    color: rgba(242, 239, 230, 0.52);
    font-size: 0.76rem;
    text-align: center;
  }
  .earth-route {
    margin-top: 15px;
    padding: 12px 13px;
    border: 1px solid rgba(255, 255, 255, 0.11);
    border-radius: 12px;
    background: rgba(255, 255, 255, 0.035);
  }
  .earth-route > div {
    display: grid;
    gap: 3px;
  }
  .earth-route strong {
    color: #fbf8f0;
    font-family: var(--planet-serif);
    font-size: 1rem;
    font-weight: 400;
  }
  .earth-route span {
    color: rgba(242, 239, 230, 0.55);
    font-size: 0.68rem;
  }
  .earth-route button {
    border-color: rgba(255, 255, 255, 0.18);
    background: rgba(255, 255, 255, 0.04);
    color: rgba(242, 239, 230, 0.82);
  }
  .footnote {
    margin: 14px 0 0;
    color: rgba(242, 239, 230, 0.42);
    font-size: 0.64rem;
    line-height: 1.5;
    text-align: center;
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
      padding: 20px 16px 18px;
      border-radius: 17px;
    }
    h2 {
      font-size: 1.55rem;
    }
    article {
      grid-template-columns: auto minmax(0, 1fr);
    }
    .route-action {
      grid-column: 1 / -1;
      flex-direction: row;
      align-items: center;
    }
    .route-action button {
      flex: 1;
    }
    .earth-route {
      align-items: stretch;
      flex-direction: column;
    }
    .earth-route button {
      width: 100%;
    }
  }
</style>
