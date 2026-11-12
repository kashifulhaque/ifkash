<script lang="ts">
  // The Lantern menu, open while the player rests: level up, attune spells, split flasks, and
  // travel (docs/game/06-progression-and-systems.md, "Lanterns"). Each action sends a core
  // `Command` (`LevelUp`, `SplitFlasks`, `Travel`); the cost, the previews, and whether a
  // command is allowed all come from the core.
  import { tick } from 'svelte';
  import type { MenuHost } from './menuHost';
  import type { MapHost, MapPlace } from '../map/mapState';
  import { travelBlock } from '../map/mapState';
  import { areaName, placeName } from '../map/places';
  import { ATTRIBUTES, lanternLabel, type LevelPreview, type MenuState } from './menuState';

  export let host: MenuHost;
  /** The realm-wide places (WP-069): travel can cross areas. Without it, only this area's Lanterns. */
  export let mapHost: MapHost | undefined = undefined;
  export let state: MenuState;
  export let announce: (text: string) => void;
  export let refresh: () => void;
  /** Closes the menu. */
  export let leave: () => void;

  const TABS = ['Level up', 'Attune spells', 'Split flasks', 'Travel'] as const;
  let tab: (typeof TABS)[number] = 'Level up';
  let preview: { attribute: number; data: LevelPreview } | null = null;
  let root: HTMLElement;

  $: here = state.lantern >= 0 ? lanternLabel(host.lanternName(state.lantern)) : 'a Lantern';
  $: canPay = state.glimmer >= state.levelCost;
  $: destinations = state.lanterns
    .filter((l) => l.lit && l.index !== state.lantern)
    .map((l) => ({ index: l.index, name: lanternLabel(host.lanternName(l.index)) }));
  // With the map (WP-069): every lit Lantern of the realm, grouped by the area it is in.
  $: realm = tab === 'Travel' && mapHost && state ? mapHost.read() : null;
  $: far = realm ? realm.places.filter((p) => p.lit && p.id !== realm.resting) : [];
  $: lucidMax = state.flasks.maxCharges - state.flasks.vigilMax;

  function raise(attribute: number): void {
    const name = ATTRIBUTES[attribute];
    const before = state.attributes[attribute];
    const cost = state.levelCost;
    host.levelUp(attribute);
    refresh();
    // The prop updates after this handler returns, so read the core again for the message.
    const now = host.read();
    const after = now.attributes[attribute];
    announce(
      after > before
        ? `${name} raised to ${after}. Level ${now.level}. ${cost} Glimmer spent, ${now.glimmer} left.`
        : `${name} could not be raised.`
    );
    preview = null;
    showPreview(attribute);
  }

  function showPreview(attribute: number): void {
    const data = host.previewLevel(attribute);
    preview = data ? { attribute, data } : null;
  }

  function split(vigil: number): void {
    host.splitFlasks(vigil);
    refresh();
    const f = host.read().flasks;
    announce(`Vigil Flask ${f.vigilMax} charges, Lucid Flask ${f.maxCharges - f.vigilMax}. Both refilled.`);
  }

  async function travel(index: number, name: string): Promise<void> {
    host.travel(index);
    refresh();
    announce(`Travelled to the ${name} Lantern`);
    await tick();
    root?.querySelector<HTMLElement>('[data-nav="body"]')?.focus();
  }

  function travelTo(p: MapPlace): void {
    if (!realm || !mapHost) return;
    const name = placeName(p.id);
    if (travelBlock(realm, p)) {
      announce(`${name}: ${travelBlock(realm, p)}.`);
      return;
    }
    announce(p.here ? `Travelled to ${name}.` : `Travelling to ${name}.`);
    mapHost.travel(p);
    refresh();
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
</script>

<div class="cols lantern" bind:this={root}>
  <div>
    <p class="note first">Resting at the {here} Lantern. Health, focus, and flasks are restored.</p>
    <ul class="list" role="tablist" aria-label="Lantern menu" aria-orientation="vertical">
      {#each TABS as t (t)}
        <li>
          <button
            type="button"
            role="tab"
            data-nav="tabs"
            aria-selected={tab === t}
            on:focus={() => (tab = t)}
            on:click={() => (tab = t)}
          >
            <span>{t}</span>
            {#if t === 'Level up'}<span class="sub">Level {state.level}</span>{/if}
          </button>
        </li>
      {/each}
    </ul>
    <p>
      <button type="button" data-nav="tabs" class="primary leave" on:click={leave}>Leave the Lantern</button>
    </p>
  </div>

  <div role="tabpanel" aria-label={tab}>
    {#if tab === 'Level up'}
      <div class="group">
        <h3>Level up</h3>
        <p class="note first">
          Level {state.level}. Glimmer {state.glimmer.toLocaleString('en-US')}. The next level costs
          {state.levelCost.toLocaleString('en-US')}.
        </p>
        <ul class="list" aria-label="Attributes">
          {#each ATTRIBUTES as a, i (a)}
            {@const maxed = state.attributes[i] >= state.attributeMax}
            <li>
              <button
                type="button"
                data-nav="body"
                aria-label={maxed
                  ? `${a} ${state.attributes[i]}, at the maximum`
                  : `Raise ${a} from ${state.attributes[i]} to ${state.attributes[i] + 1} for ${state.levelCost} Glimmer${canPay ? '' : ', not enough Glimmer'}`}
                aria-disabled={!canPay || maxed ? 'true' : undefined}
                on:focus={() => showPreview(i)}
                on:mouseenter={() => showPreview(i)}
                on:click={() => canPay && !maxed && raise(i)}
              >
                <span>{a}</span>
                <span class="sub">{state.attributes[i]}{maxed ? ' (max)' : ` → ${state.attributes[i] + 1}`}</span>
              </button>
            </li>
          {/each}
        </ul>
        {#if preview}
          {@const p = preview.data}
          <dl class="stats" aria-label="Effect of raising {ATTRIBUTES[preview.attribute]}">
            <dt>Health</dt><dd>{p.hp[1]}</dd><dd class="change" class:up={p.hp[1] > p.hp[0]}>{p.hp[1] > p.hp[0] ? `▲ +${p.hp[1] - p.hp[0]}` : ''}</dd>
            <dt>Focus</dt><dd>{p.fp[1]}</dd><dd class="change" class:up={p.fp[1] > p.fp[0]}>{p.fp[1] > p.fp[0] ? `▲ +${(p.fp[1] - p.fp[0]).toFixed(1)}` : ''}</dd>
            <dt>Stamina</dt><dd>{p.stamina[1]}</dd><dd class="change" class:up={p.stamina[1] > p.stamina[0]}>{p.stamina[1] > p.stamina[0] ? `▲ +${(p.stamina[1] - p.stamina[0]).toFixed(1)}` : ''}</dd>
            <dt>Max equip load</dt><dd>{p.load[1].toFixed(1)}</dd><dd class="change" class:up={p.load[1] > p.load[0]}>{p.load[1] > p.load[0] ? `▲ +${(p.load[1] - p.load[0]).toFixed(1)}` : ''}</dd>
          </dl>
        {:else}
          <p class="note">Focus an attribute to see what one more point does.</p>
        {/if}
      </div>
    {:else if tab === 'Attune spells'}
      <div class="group">
        <h3>Attune spells</h3>
        <p class="note first">No spells to attune yet. Catalysts and spells arrive with the spell system, and this tab will list them then.</p>
        <p>
          <button type="button" data-nav="body" aria-disabled="true">Attune (not built yet)</button>
        </p>
      </div>
    {:else if tab === 'Split flasks'}
      <div class="group">
        <h3>Split flasks</h3>
        <p class="note first">
          {state.flasks.maxCharges} charges in all, shared between the two flasks. Moving a charge refills both.
        </p>
        <dl class="stats">
          <dt>Vigil Flask (health)</dt><dd>{state.flasks.vigilMax}</dd><dd class="change"></dd>
          <dt>Lucid Flask (focus)</dt><dd>{lucidMax}</dd><dd class="change"></dd>
        </dl>
        <p class="row">
          <button
            type="button"
            data-nav="body"
            aria-disabled={lucidMax <= 0 ? 'true' : undefined}
            on:click={() => lucidMax > 0 && split(state.flasks.vigilMax + 1)}
          >Move a charge to Vigil</button>
          <button
            type="button"
            data-nav="body"
            aria-disabled={state.flasks.vigilMax <= 0 ? 'true' : undefined}
            on:click={() => state.flasks.vigilMax > 0 && split(state.flasks.vigilMax - 1)}
          >Move a charge to Lucid</button>
        </p>
      </div>
    {:else}
      <div class="group">
        <h3>Travel</h3>
        <p class="note first">Travel to a Lantern you have lit. You rest there too.</p>
        {#if realm}
          {#if far.length}
            <ul class="list" aria-label="Lit Lanterns">
              {#each far as p (p.id)}
                <li>
                  <button type="button" data-nav="body" on:click={() => travelTo(p)}>
                    <span>{placeName(p.id)}</span>
                    <span class="sub">{areaName(p.area)}</span>
                  </button>
                </li>
              {/each}
            </ul>
          {:else}
            <p class="note">No other Lantern is lit yet.</p>
          {/if}
        {:else if destinations.length}
          <ul class="list" aria-label="Lit Lanterns">
            {#each destinations as d (d.index)}
              <li>
                <button type="button" data-nav="body" on:click={() => travel(d.index, d.name)}>
                  <span>{d.name}</span>
                  <span class="sub">Lit</span>
                </button>
              </li>
            {/each}
          </ul>
        {:else}
          <p class="note">No other Lantern is lit yet.</p>
        {/if}
      </div>
    {/if}
  </div>
</div>

<style>
  .first {
    margin-top: 0;
  }

  .leave {
    width: 100%;
  }

  .row {
    display: flex;
    flex-wrap: wrap;
    gap: 0.5em;
  }

  [role='tab'][aria-selected='true'] {
    border-color: var(--fjord);
    box-shadow: inset 3px 0 0 var(--fjord);
  }
</style>
