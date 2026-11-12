<script lang="ts">
  // Equipment: pick a slot, then an item for it. Focusing an item previews what the change
  // does to attack rating, defense, equip load, and roll tier (numbers from the core's
  // `preview_equip`); choosing it sends `Equip` or `Unequip`.
  import { tick } from 'svelte';
  import type { MenuHost } from './menuHost';
  import { SLOT_DEFS, TIER_NAMES, delta, type Loadout, type MenuState } from './menuState';

  export let host: MenuHost;
  export let state: MenuState;
  export let announce: (text: string) => void;
  export let refresh: () => void;

  /** The slot being changed, or null while choosing a slot. */
  let picking: number | null = null;
  let preview: { item: number; after: Loadout } | null = null;
  let listEl: HTMLElement;

  let current: Loadout | null = null;
  $: {
    state;
    current = host.loadout();
  }

  interface Candidate {
    item: number;
    name: string;
    after: Loadout;
  }

  function candidatesFor(slot: number, st: MenuState): Candidate[] {
    const pool = new Set<number>([...st.gear, ...st.stacks.map((s) => s.item), ...st.flaskItems.filter((n) => n > 0)]);
    const out: Candidate[] = [];
    for (const item of pool) {
      if (st.slots[slot] === item) continue;
      const after = host.previewEquip(slot, item);
      if (after?.allowed) out.push({ item, name: host.itemName(item), after });
    }
    return out.sort((a, b) => a.name.localeCompare(b.name));
  }

  $: slotDefs = SLOT_DEFS.map((d) => ({
    ...d,
    item: state.slots[d.slot],
    locked: d.group === 'Charms' && d.slot - 6 >= state.charmSlots
  }));
  $: groups = [...new Set(slotDefs.map((d) => d.group))];
  $: cands = picking === null ? [] : candidatesFor(picking, state);
  $: canEmpty = picking !== null && state.slots[picking] !== 0 && !!host.previewEquip(picking, 0)?.allowed;
  $: emptyAfter = picking !== null && canEmpty ? host.previewEquip(picking, 0) : null;
  $: pickingDef = picking === null ? null : SLOT_DEFS[picking];

  /** The label of a slot with what it holds, for the button and the screen reader. */
  function slotText(label: string, item: number): string {
    return `${label}: ${item ? host.itemName(item) : 'empty'}`;
  }

  async function choose(slot: number): Promise<void> {
    picking = slot;
    preview = null;
    announce(`Choosing for ${SLOT_DEFS[slot].label}`);
    await tick();
    const first = listEl?.querySelector<HTMLElement>('[data-nav="cands"]');
    first?.focus();
  }

  async function apply(item: number): Promise<void> {
    if (picking === null) return;
    const slot = picking;
    const label = SLOT_DEFS[slot].label;
    const name = item ? host.itemName(item) : state.slots[slot] ? host.itemName(state.slots[slot]) : '';
    if (item) host.equip(slot, item);
    else host.unequip(slot);
    refresh();
    announce(item ? `Equipped ${name} in ${label}` : `Emptied ${label}; ${name} unequipped`);
    picking = null;
    preview = null;
    await tick();
    listEl?.querySelector<HTMLElement>(`[data-slot="${slot}"]`)?.focus();
  }

  /** Leaves the item list first, then lets the menu go back. */
  export function back(): boolean {
    if (picking === null) return false;
    const slot = picking;
    picking = null;
    preview = null;
    void tick().then(() => listEl?.querySelector<HTMLElement>(`[data-slot="${slot}"]`)?.focus());
    return true;
  }

  function show(item: number, after: Loadout): void {
    preview = { item, after };
  }

  function describe(cur: Loadout | null, after: Loadout | null): string {
    if (!cur || !after) return '';
    const parts = [
      `Attack rating ${after.attackRating}${delta(cur.attackRating, after.attackRating) ? `, ${delta(cur.attackRating, after.attackRating)}` : ''}`,
      `defense ${after.defense}`,
      `equip load ${after.load.toFixed(1)} of ${after.maxLoad.toFixed(1)}`,
      `roll ${after.tier}`
    ];
    if (after.unwieldy) parts.push('requirements not met');
    return parts.join(', ');
  }

  $: shown = preview?.after ?? null;
  $: rows = current
    ? [
        { label: 'Attack rating', now: String(current.attackRating), then: shown ? String(shown.attackRating) : '', d: shown ? delta(current.attackRating, shown.attackRating) : '', better: shown ? shown.attackRating - current.attackRating : 0 },
        { label: 'Defense', now: String(current.defense), then: shown ? String(shown.defense) : '', d: shown ? delta(current.defense, shown.defense) : '', better: shown ? shown.defense - current.defense : 0 },
        { label: 'Equip load', now: `${current.load.toFixed(1)} / ${current.maxLoad.toFixed(1)}`, then: shown ? `${shown.load.toFixed(1)} / ${shown.maxLoad.toFixed(1)}` : '', d: shown ? delta(current.load, shown.load, 1) : '', better: shown ? current.load - shown.load : 0 },
        { label: 'Roll', now: current.tier, then: shown ? shown.tier : '', d: shown && shown.tier !== current.tier ? (TIER_NAMES.indexOf(shown.tier) > TIER_NAMES.indexOf(current.tier) ? 'slower' : 'faster') : '', better: shown ? TIER_NAMES.indexOf(current.tier) - TIER_NAMES.indexOf(shown.tier) : 0 }
      ]
    : [];
</script>

<div class="cols" bind:this={listEl}>
  <div>
    {#each groups as g (g)}
      <div class="group">
        <h3>{g}</h3>
        <ul class="list" aria-label={g}>
          {#each slotDefs.filter((d) => d.group === g) as d (d.slot)}
            <li>
              <button
                type="button"
                data-nav="slots"
                data-slot={d.slot}
                aria-label={d.locked ? `${d.label}: locked` : slotText(d.label, d.item)}
                aria-disabled={d.locked ? 'true' : undefined}
                aria-pressed={picking === d.slot}
                on:click={() => !d.locked && choose(d.slot)}
                on:focus={() => {
                  if (picking === null) preview = null;
                }}
              >
                <span>{d.label}</span>
                <span class="sub">{d.locked ? 'Locked' : d.item ? host.itemName(d.item) : 'Empty'}</span>
              </button>
            </li>
          {/each}
        </ul>
      </div>
    {/each}
  </div>

  <div>
    {#if picking === null}
      <p class="note">
        Choose a slot. Then focus an item to see what it changes before you equip it.
      </p>
    {:else}
      <div class="group">
        <h3>For {pickingDef?.label}</h3>
        <ul class="list" aria-label={`Items for ${pickingDef?.label}`}>
          {#each cands as c (c.item)}
            <li>
              <button
                type="button"
                data-nav="cands"
                aria-label={`${c.name}. ${describe(current, c.after)}`}
                on:focus={() => show(c.item, c.after)}
                on:mouseenter={() => show(c.item, c.after)}
                on:click={() => apply(c.item)}
              >
                <span>{c.name}</span>
              </button>
            </li>
          {/each}
          {#if canEmpty && emptyAfter}
            <li>
              <button
                type="button"
                data-nav="cands"
                aria-label={`Empty this slot. ${describe(current, emptyAfter)}`}
                on:focus={() => show(0, emptyAfter)}
                on:mouseenter={() => show(0, emptyAfter)}
                on:click={() => apply(0)}
              >
                <span>Empty this slot</span>
              </button>
            </li>
          {/if}
        </ul>
        {#if cands.length === 0 && !canEmpty}
          <p class="note">Nothing you own fits this slot.</p>
        {/if}
      </div>
    {/if}

    <div class="group">
      <h3>{shown ? 'After this change' : 'Current loadout'}</h3>
      {#if current}
        <dl class="stats">
          {#each rows as r (r.label)}
            <dt>{r.label}</dt>
            <dd>{shown ? r.then : r.now}</dd>
            <dd class="change" class:up={r.better > 0} class:down={r.better < 0}>
              {#if shown && r.d}{r.better > 0 ? '▲' : r.better < 0 ? '▼' : ''} {r.d}{/if}
            </dd>
          {/each}
        </dl>
        {#if shown?.unwieldy}
          <p class="note">This weapon's requirements aren't met: scaling is lost and recovery is slower.</p>
        {/if}
      {/if}
    </div>
  </div>
</div>
