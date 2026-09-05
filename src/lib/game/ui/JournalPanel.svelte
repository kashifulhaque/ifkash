<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import { BIOMES, OCEAN } from '../biomes';
  import { JOURNAL_BIOMES, MILESTONES, SHARD_COUNT, SPECIES, type Journal } from '../journal';
  import { WONDERS } from '../wonders';

  export let journal: Journal;
  /** Ids of the wonders found on this planet. */
  export let found: string[] = [];
  export let shards: { found: number; total: number } = { found: 0, total: SHARD_COUNT };
  export let seed = '';

  const dispatch = createEventDispatcher();

  const biomes = JOURNAL_BIOMES.map((id) => (id === 'ocean' ? OCEAN : BIOMES.find((b) => b.id === id)!));

  $: biomesSeen = journal.biomes.filter((id) => JOURNAL_BIOMES.includes(id as (typeof JOURNAL_BIOMES)[number])).length;
  $: speciesSeen = SPECIES.filter((sp) => journal.species.includes(sp)).length;
  $: earned = MILESTONES.filter((m) => journal.milestones.includes(m.id)).length;
  $: paces = Math.floor(journal.paces);

  /** "polar bear" reads better as "Polar bear" on a stamp. */
  const cap = (s: string) => s.charAt(0).toUpperCase() + s.slice(1);
</script>

<div class="overlay" role="dialog" aria-modal="true" aria-label="Traveller's journal">
  <button class="backdrop" on:click={() => dispatch('close')} aria-label="Close"></button>
  <div class="panel">
    <header>
      <p class="kicker">Traveller's journal</p>
      <h2>Everything you have gathered</h2>
      <p class="lead">
        Kept across every planet you visit. This one is <code>{seed}</code>.
      </p>
      <button class="close" on:click={() => dispatch('close')} aria-label="Close">
        <svg viewBox="0 0 24 24" aria-hidden="true"><path d="M6 6l12 12M18 6L6 18" /></svg>
      </button>
    </header>

    <div class="stats">
      <div class="stat">
        <strong>{found.length}<span>/{WONDERS.length}</span></strong>
        <p>wonders here</p>
      </div>
      <div class="stat">
        <strong class="lilac">{shards.found}<span>/{shards.total}</span></strong>
        <p>shards here</p>
      </div>
      <div class="stat">
        <strong>{paces.toLocaleString()}</strong>
        <p>paces walked</p>
      </div>
      <div class="stat">
        <strong>{journal.hops.toLocaleString()}</strong>
        <p>hops</p>
      </div>
      <div class="stat">
        <strong>{journal.planets.length}</strong>
        <p>{journal.planets.length === 1 ? 'planet' : 'planets'}</p>
      </div>
    </div>

    <section>
      <h3>Wonders <span class="tally">{found.length} / {WONDERS.length}</span></h3>
      <ul class="stamps">
        {#each WONDERS as w}
          <li class:known={found.includes(w.id)}>
            <span class="mark">{found.includes(w.id) ? '◆' : '◇'}</span>
            <span class="name">{found.includes(w.id) ? w.title : w.action}</span>
          </li>
        {/each}
      </ul>
    </section>

    <section>
      <h3>Biomes <span class="tally">{biomesSeen} / {biomes.length}</span></h3>
      <ul class="stamps">
        {#each biomes as b}
          {@const known = journal.biomes.includes(b.id)}
          <li class:known>
            <span class="swatch" style={`--c:#${b.ground[1].toString(16).padStart(6, '0')}`}></span>
            <span class="name">{known ? b.name : b.kind.charAt(0) + b.kind.slice(1).toLowerCase()}</span>
          </li>
        {/each}
      </ul>
    </section>

    <section>
      <h3>Animals befriended <span class="tally">{speciesSeen} / {SPECIES.length}</span></h3>
      <ul class="stamps">
        {#each SPECIES as sp}
          {@const known = journal.species.includes(sp)}
          <li class:known>
            <span class="mark">{known ? '♥' : '·'}</span>
            <span class="name">{known ? cap(sp) : '? ? ?'}</span>
          </li>
        {/each}
      </ul>
    </section>

    <section>
      <h3>Milestones <span class="tally">{earned} / {MILESTONES.length}</span></h3>
      <ul class="milestones">
        {#each MILESTONES as m}
          {@const known = journal.milestones.includes(m.id)}
          <li class:known>
            <span class="badge">{known ? '✦' : '·'}</span>
            <div>
              <p class="title">{m.title}</p>
              <p class="blurb">{m.blurb}</p>
            </div>
          </li>
        {/each}
      </ul>
    </section>

    <footer>
      <button class="continue" on:click={() => dispatch('close')}>Back to the planet</button>
      <span class="esc">J, Esc, or tap outside to close</span>
    </footer>
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
    width: min(560px, 100%);
    max-height: min(84vh, 84dvh);
    overflow-y: auto;
    padding: 26px 28px 22px;
    background: rgba(14, 30, 40, 0.94);
    border: 1px solid rgba(255, 255, 255, 0.12);
    border-radius: 18px;
    box-shadow: 0 30px 80px rgba(0, 0, 0, 0.5);
    animation: pop 0.3s cubic-bezier(0.2, 0.8, 0.2, 1);
  }
  header {
    position: relative;
    padding-right: 36px;
  }
  .kicker {
    margin: 0;
    font-size: 0.62rem;
    letter-spacing: 0.2em;
    text-transform: uppercase;
    color: #e9c46a;
  }
  h2 {
    margin: 8px 0 8px;
    font-family: var(--planet-serif);
    font-weight: 400;
    font-size: 1.8rem;
    line-height: 1.1;
    color: #fbf8f0;
  }
  .lead {
    margin: 0;
    font-size: 0.82rem;
    line-height: 1.5;
    color: rgba(242, 239, 230, 0.66);
  }
  .lead code {
    font-family: var(--font-mono, ui-monospace, monospace);
    font-size: 0.78rem;
    color: #fbf8f0;
  }
  .close {
    position: absolute;
    top: 0;
    right: 0;
    width: 32px;
    height: 32px;
    border-radius: 50%;
    border: 1px solid rgba(255, 255, 255, 0.14);
    background: rgba(255, 255, 255, 0.04);
    color: rgba(242, 239, 230, 0.8);
    display: grid;
    place-items: center;
  }
  .close:hover {
    background: rgba(255, 255, 255, 0.1);
  }
  .close svg {
    width: 14px;
    height: 14px;
    fill: none;
    stroke: currentColor;
    stroke-width: 1.6;
  }

  .stats {
    display: grid;
    grid-template-columns: repeat(5, 1fr);
    gap: 8px;
    margin: 18px 0 6px;
  }
  .stat {
    padding: 10px 8px;
    border-radius: 12px;
    border: 1px solid rgba(255, 255, 255, 0.1);
    background: rgba(255, 255, 255, 0.03);
    text-align: center;
  }
  .stat strong {
    display: block;
    font-family: var(--planet-serif);
    font-weight: 400;
    font-size: 1.35rem;
    line-height: 1;
    color: #fbf8f0;
  }
  .stat strong span {
    font-size: 0.8rem;
    color: rgba(242, 239, 230, 0.5);
  }
  .stat strong.lilac {
    color: #d7c4ff;
  }
  .stat p {
    margin: 5px 0 0;
    font-size: 0.6rem;
    letter-spacing: 0.12em;
    text-transform: uppercase;
    color: rgba(242, 239, 230, 0.55);
  }

  section {
    margin-top: 18px;
  }
  h3 {
    display: flex;
    align-items: baseline;
    justify-content: space-between;
    margin: 0 0 8px;
    font-size: 0.7rem;
    font-weight: 600;
    letter-spacing: 0.16em;
    text-transform: uppercase;
    color: rgba(242, 239, 230, 0.8);
  }
  .tally {
    font-weight: 400;
    letter-spacing: 0.05em;
    color: rgba(242, 239, 230, 0.5);
  }
  ul {
    list-style: none;
    margin: 0;
    padding: 0;
  }
  .stamps {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(150px, 1fr));
    gap: 6px;
  }
  .stamps li {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 7px 10px;
    border-radius: 9px;
    border: 1px dashed rgba(255, 255, 255, 0.12);
    font-size: 0.76rem;
    color: rgba(242, 239, 230, 0.4);
  }
  .stamps li.known {
    border-style: solid;
    border-color: rgba(233, 196, 106, 0.35);
    background: rgba(233, 196, 106, 0.07);
    color: #fbf8f0;
  }
  .stamps .mark {
    width: 14px;
    text-align: center;
    color: rgba(242, 239, 230, 0.35);
  }
  .stamps li.known .mark {
    color: #e9c46a;
  }
  .stamps .swatch {
    width: 12px;
    height: 12px;
    border-radius: 3px;
    background: var(--c);
    opacity: 0.35;
    filter: saturate(0.3);
  }
  .stamps li.known .swatch {
    opacity: 1;
    filter: none;
  }
  .name {
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .milestones {
    display: grid;
    gap: 6px;
  }
  .milestones li {
    display: grid;
    grid-template-columns: 26px 1fr;
    gap: 10px;
    align-items: start;
    padding: 9px 12px;
    border-radius: 10px;
    border: 1px dashed rgba(255, 255, 255, 0.12);
    color: rgba(242, 239, 230, 0.45);
  }
  .milestones li.known {
    border-style: solid;
    border-color: rgba(233, 196, 106, 0.35);
    background: rgba(233, 196, 106, 0.07);
    color: #fbf8f0;
  }
  .milestones .badge {
    width: 26px;
    height: 26px;
    border-radius: 50%;
    display: grid;
    place-items: center;
    border: 1px solid rgba(255, 255, 255, 0.14);
    font-size: 0.8rem;
    color: rgba(242, 239, 230, 0.4);
  }
  .milestones li.known .badge {
    border-color: rgba(233, 196, 106, 0.6);
    color: #e9c46a;
    background: rgba(233, 196, 106, 0.12);
  }
  .milestones .title {
    margin: 0;
    font-size: 0.82rem;
    font-weight: 500;
  }
  .milestones .blurb {
    margin: 2px 0 0;
    font-size: 0.72rem;
    line-height: 1.45;
    color: rgba(242, 239, 230, 0.55);
  }

  footer {
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
  .esc {
    font-size: 0.68rem;
    color: rgba(242, 239, 230, 0.45);
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
    .panel {
      padding: 20px 18px 18px;
    }
    .stats {
      grid-template-columns: repeat(3, 1fr);
    }
    .stamps {
      grid-template-columns: repeat(2, 1fr);
    }
  }
</style>
