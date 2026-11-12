<script lang="ts">
  // The map screen (WP-069; docs/game/06-progression-and-systems.md, "Map"). Two views of the
  // same core state: a drawn map (an SVG of the cells the player has visited, the lit Lanterns,
  // the defeated bosses, the Remnant, and the markers) and a list view that says every
  // discovered place as text, with how far away it is, for screen-reader users and for players
  // who don't read maps well. Lit Lanterns are travel buttons while the player rests at a
  // Lantern. The screen reads `host.read()` and changes the world only through `host`.
  import { tick } from 'svelte';
  import { MAX_MARKERS, describeOffset, groupPlaces, mapExtent, placeSummary, revealedCells, travelBlock } from './mapState';
  import type { MapHost, MapPlace, MapView } from './mapState';
  import { areaName, placeName } from './places';

  export let host: MapHost;
  export let announce: (text: string) => void;

  const TABS = ['Map', 'List'] as const;
  let tab: (typeof TABS)[number] = 'Map';
  let view: MapView = host.read();
  let root: HTMLElement;
  let svg: SVGSVGElement;

  function refresh(): void {
    view = host.read();
  }

  $: extent = mapExtent(view);
  $: width = extent[2] - extent[0];
  $: height = extent[3] - extent[1];
  $: unit = Math.max(width, height) / 70; // One drawn "dot" in meters, so marks scale with the area.
  $: revealed = revealedCells(view.cells);
  $: groups = groupPlaces(view);
  $: here = view.places.filter((p) => p.here && p.lit);
  $: nearMarkerIndex = view.markers.findIndex(([x, z]) => Math.hypot(x - view.player.x, z - view.player.z) <= 3);
  $: canAddMarker = view.markerTotal < MAX_MARKERS;

  const sx = (x: number) => x - extent[0];
  const sy = (z: number) => extent[3] - z;

  function travel(p: MapPlace): void {
    const block = travelBlock(view, p);
    if (block) {
      announce(`${placeName(p.id)}: ${block}.`);
      return;
    }
    announce(`Travelling to ${placeName(p.id)}.`);
    host.travel(p);
    refresh();
  }

  function mark(x: number, z: number, what: string): void {
    const removing = nearestMarker(x, z) >= 0;
    if (!removing && !canAddMarker) {
      announce(`The map holds ${MAX_MARKERS} markers. Remove one first.`);
      return;
    }
    host.toggleMarker(x, z);
    refresh();
    announce(removing ? `Marker removed ${what}.` : `Marker placed ${what}.`);
  }

  function nearestMarker(x: number, z: number): number {
    return view.markers.findIndex(([mx, mz]) => Math.hypot(mx - x, mz - z) <= 3);
  }

  function markHere(): void {
    mark(view.player.x, view.player.z, 'where you stand');
  }

  function onMapClick(e: MouseEvent): void {
    const r = svg.getBoundingClientRect();
    if (r.width === 0 || r.height === 0) return;
    // preserveAspectRatio "xMidYMid meet": the drawing is centered inside the box.
    const scale = Math.min(r.width / width, r.height / height);
    const x = extent[0] + (e.clientX - r.left - (r.width - width * scale) / 2) / scale;
    const z = extent[3] - (e.clientY - r.top - (r.height - height * scale) / 2) / scale;
    if (x < extent[0] || x > extent[2] || z < extent[1] || z > extent[3]) return;
    mark(x, z, 'on the map');
  }

  function removeMarker(i: number): void {
    const [x, z] = view.markers[i];
    mark(x, z, `${describeOffset(view.player.x, view.player.z, x, z)}`);
  }

  async function pick(next: (typeof TABS)[number]): Promise<void> {
    tab = next;
    refresh();
    await tick();
  }

  /** From the body, back to the tab list. */
  export function back(): boolean {
    const active = document.activeElement;
    if (active instanceof HTMLElement && active.dataset.nav === 'body') {
      root?.querySelector<HTMLElement>('[data-nav="tabs"][aria-selected="true"]')?.focus();
      return true;
    }
    return false;
  }

  /** The parent calls this after a command ran, so the screen shows the new state. */
  export function update(): void {
    refresh();
  }

  $: summary = `Map of ${areaName(view.area)}. ${view.cells.length} cells explored, ${here.length} lit ${here.length === 1 ? 'Lantern' : 'Lanterns'}, ${view.bosses.length} defeated ${view.bosses.length === 1 ? 'boss' : 'bosses'}, ${view.markers.length} ${view.markers.length === 1 ? 'marker' : 'markers'}. The List view has the same places as text.`;
</script>

<div class="cols map" bind:this={root}>
  <div>
    <p class="note first">
      {areaName(view.area)}.
      {#if view.resting}Resting at {placeName(view.resting)}: choose a lit Lantern to travel.{:else}Rest at a Lantern to travel.{/if}
    </p>
    <ul class="list" role="tablist" aria-label="Map views" aria-orientation="vertical">
      {#each TABS as t (t)}
        <li>
          <button
            type="button"
            role="tab"
            data-nav="tabs"
            aria-selected={tab === t}
            on:focus={() => pick(t)}
            on:click={() => pick(t)}
          >
            <span>{t}</span>
            <span class="sub">{t === 'Map' ? 'Drawn' : 'Text'}</span>
          </button>
        </li>
      {/each}
    </ul>
    <p class="note">Up to {MAX_MARKERS} markers. {view.markerTotal} placed.</p>
  </div>

  <div role="tabpanel" aria-label={tab}>
    {#if tab === 'Map'}
      <div class="group">
        <h3>{areaName(view.area)}</h3>
        <!-- svelte-ignore a11y-click-events-have-key-events -->
        <!-- svelte-ignore a11y-no-noninteractive-element-interactions -->
        <svg
          bind:this={svg}
          class="chart"
          viewBox="0 0 {width} {height}"
          preserveAspectRatio="xMidYMid meet"
          role="img"
          aria-label={summary}
          on:click={onMapClick}
        >
          <rect class="ground" x="0" y="0" width={width} height={height} />
          {#each revealed.halo as [ix, iz] (`${ix},${iz}`)}
            <rect class="halo" x={sx(ix * view.cell)} y={sy((iz + 1) * view.cell)} width={view.cell} height={view.cell} />
          {/each}
          {#each view.cells as [ix, iz] (`${ix},${iz}`)}
            <rect class="seen" x={sx(ix * view.cell)} y={sy((iz + 1) * view.cell)} width={view.cell} height={view.cell} />
          {/each}
          {#each view.bosses as b (b.name)}
            <g class="boss" transform="translate({sx(b.x)} {sy(b.z)})">
              <title>{b.name}, defeated</title>
              <path d="M0 {-unit * 1.4} L{unit * 1.4} 0 L0 {unit * 1.4} L{-unit * 1.4} 0 Z" />
            </g>
          {/each}
          {#each here as p (p.id)}
            <g class="lantern" class:resting={p.id === view.resting} transform="translate({sx(p.x)} {sy(p.z)})">
              <title>{placeName(p.id)}, Lantern lit</title>
              <circle r={unit * 1.2} />
              <circle class="core" r={unit * 0.5} />
            </g>
          {/each}
          {#if view.remnant}
            <g class="remnant" transform="translate({sx(view.remnant.x)} {sy(view.remnant.z)})">
              <title>Your Remnant, {view.remnant.glimmer} Glimmer</title>
              <circle r={unit * 1.1} fill="none" stroke-width={unit * 0.4} />
              <circle r={unit * 0.35} />
            </g>
          {/if}
          {#each view.markers as [mx, mz], i (i)}
            <g class="marker" transform="translate({sx(mx)} {sy(mz)})">
              <title>Marker {i + 1}</title>
              <path d="M0 {-unit * 1.5} L{unit * 1.1} {unit * 0.4} L{-unit * 1.1} {unit * 0.4} Z" />
            </g>
          {/each}
          <g class="player" transform="translate({sx(view.player.x)} {sy(view.player.z)}) rotate({(view.player.yaw * 180) / Math.PI})">
            <title>You</title>
            <path d="M0 {-unit * 1.6} L{unit * 1.1} {unit * 1.1} L0 {unit * 0.5} L{-unit * 1.1} {unit * 1.1} Z" />
          </g>
        </svg>
        <p class="row">
          <button
            type="button"
            data-nav="body"
            aria-disabled={nearMarkerIndex < 0 && !canAddMarker ? 'true' : undefined}
            on:click={markHere}
          >{nearMarkerIndex >= 0 ? 'Remove the marker here' : 'Place a marker where I stand'}</button>
        </p>
        <p class="note">Click the map to place a marker, or click a marker to remove it.</p>
        <ul class="legend" aria-label="Legend">
          <li><svg viewBox="-2 -2 4 4" aria-hidden="true"><path class="player" d="M0 -1.6 L1.1 1.1 L0 0.5 L-1.1 1.1 Z" /></svg> You</li>
          <li><svg viewBox="-2 -2 4 4" aria-hidden="true"><g class="lantern"><circle r="1.2" /><circle class="core" r="0.5" /></g></svg> Lit Lantern</li>
          <li><svg viewBox="-2 -2 4 4" aria-hidden="true"><g class="boss"><path d="M0 -1.4 L1.4 0 L0 1.4 L-1.4 0 Z" /></g></svg> Defeated boss</li>
          <li><svg viewBox="-2 -2 4 4" aria-hidden="true"><g class="remnant"><circle r="1.1" fill="none" stroke-width="0.4" /><circle r="0.35" /></g></svg> Remnant</li>
          <li><svg viewBox="-2 -2 4 4" aria-hidden="true"><g class="marker"><path d="M0 -1.5 L1.1 0.4 L-1.1 0.4 Z" /></g></svg> Marker</li>
        </ul>
      </div>
    {:else}
      <div class="group">
        <h3>Discovered places</h3>
        <p class="note first">
          {view.cells.length} map {view.cells.length === 1 ? 'cell' : 'cells'} explored in {areaName(view.area)}.
          Distances are from where you stand; north is up on the map.
        </p>
        {#each groups as g (g.area)}
          <h4 id="map-area-{g.area}">{g.name}{g.area === view.area ? ' (you are here)' : ''}</h4>
          <ul class="list places" aria-labelledby="map-area-{g.area}">
            {#each g.places as p (p.id)}
              {@const block = travelBlock(view, p)}
              <li>
                <button
                  type="button"
                  data-nav="body"
                  aria-label="{placeSummary(view, p)}{block ? `. ${block}` : '. Travel here'}"
                  aria-disabled={block ? 'true' : undefined}
                  on:click={() => travel(p)}
                >
                  <span>{placeName(p.id)}</span>
                  <span class="sub">
                    {#if p.id === view.resting}Resting here
                    {:else if p.here}{describeOffset(view.player.x, view.player.z, p.x, p.z)}
                    {:else}{areaName(p.area)}{/if}
                  </span>
                </button>
              </li>
            {/each}
          </ul>
        {:else}
          <p class="note">No Lantern is lit yet. Rest at one to chart the way back to it.</p>
        {/each}

        {#if view.bosses.length}
          <h4>Defeated bosses in {areaName(view.area)}</h4>
          <ul class="plain">
            {#each view.bosses as b (b.name)}
              <li>{b.name}, {describeOffset(view.player.x, view.player.z, b.x, b.z)}</li>
            {/each}
          </ul>
        {/if}

        {#if view.remnant}
          <h4>Your Remnant</h4>
          <p class="note first">
            {view.remnant.glimmer.toLocaleString('en-US')} Glimmer, {describeOffset(view.player.x, view.player.z, view.remnant.x, view.remnant.z)}.
          </p>
        {/if}

        <h4 id="map-markers">Markers in {areaName(view.area)}</h4>
        {#if view.markers.length}
          <ul class="list" aria-labelledby="map-markers">
            {#each view.markers as m, i (i)}
              <li>
                <button
                  type="button"
                  data-nav="body"
                  aria-label="Marker {i + 1}, {describeOffset(view.player.x, view.player.z, m[0], m[1])}. Remove it"
                  on:click={() => removeMarker(i)}
                >
                  <span>Marker {i + 1}</span>
                  <span class="sub">{describeOffset(view.player.x, view.player.z, m[0], m[1])}</span>
                </button>
              </li>
            {/each}
          </ul>
        {:else}
          <p class="note first">None placed here.</p>
        {/if}
        <p class="row">
          <button
            type="button"
            data-nav="body"
            aria-disabled={nearMarkerIndex < 0 && !canAddMarker ? 'true' : undefined}
            on:click={markHere}
          >{nearMarkerIndex >= 0 ? 'Remove the marker here' : 'Place a marker where I stand'}</button>
        </p>
      </div>
    {/if}
  </div>
</div>

<style>
  .first {
    margin-top: 0;
  }

  .row {
    display: flex;
    flex-wrap: wrap;
    gap: 0.5em;
    margin: 0.75em 0 0;
  }

  [role='tab'][aria-selected='true'] {
    border-color: var(--fjord);
    box-shadow: inset 3px 0 0 var(--fjord);
  }

  h4 {
    margin: 1em 0 0.4em;
    font-size: 0.875em;
    font-weight: 600;
    color: var(--ink);
  }

  .plain {
    margin: 0;
    padding-left: 1.2em;
    color: var(--ink-2);
    font-size: 0.875em;
  }

  .chart {
    display: block;
    width: 100%;
    max-height: 52vh;
    aspect-ratio: 1 / 1;
    border: 1px solid var(--line-strong);
    border-radius: var(--radius-sm);
    background: var(--surface-2);
    cursor: crosshair;
  }

  .chart .ground {
    fill: var(--surface-2);
  }

  .chart .halo {
    fill: var(--line-strong);
    opacity: 0.35;
  }

  .chart .seen {
    fill: var(--line-strong);
    opacity: 0.8;
    stroke: var(--ink-3);
    stroke-width: 0.2;
  }

  .lantern circle:not(.core) {
    fill: var(--surface);
    stroke: var(--fjord);
    stroke-width: 0.5;
  }

  .lantern .core {
    fill: var(--fjord);
  }

  .lantern.resting circle:not(.core) {
    fill: var(--fjord-wash);
    stroke-width: 1;
  }

  .boss path {
    fill: var(--falu);
  }

  .remnant circle {
    stroke: var(--clay);
    fill: var(--clay);
  }

  .remnant circle[fill='none'] {
    fill: none;
  }

  .marker path {
    fill: var(--moss);
  }

  .player path {
    fill: var(--ink);
    stroke: var(--surface);
    stroke-width: 0.3;
  }

  .legend {
    list-style: none;
    margin: 0.5em 0 0;
    padding: 0;
    display: flex;
    flex-wrap: wrap;
    gap: 0.25em 1em;
    font-size: 0.8125em;
    color: var(--ink-2);
  }

  .legend li {
    display: inline-flex;
    align-items: center;
    gap: 0.35em;
  }

  .legend svg {
    width: 1.25em;
    height: 1.25em;
  }

  @media (max-width: 40em) {
    .map {
      grid-template-columns: minmax(0, 1fr);
    }
  }
</style>
