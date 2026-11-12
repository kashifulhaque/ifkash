<script lang="ts">
  // Inventory: owned gear, consumables, and the flasks. A consumable can go in a quick slot
  // (`Equip`) or be used from one (`UseItem`); gear is equipped from the Equipment screen.
  import { tick } from 'svelte';
  import type { MenuHost } from './menuHost';
  import { SLOT, SLOT_DEFS, type MenuState } from './menuState';

  export let host: MenuHost;
  export let state: MenuState;
  export let announce: (text: string) => void;
  export let refresh: () => void;
  export let go: (screen: 'equipment') => void;
  /** Closes the menus so the game runs and a used item takes effect. */
  export let useAndClose: () => void;

  interface Row {
    key: string;
    item: number;
    name: string;
    kind: 'gear' | 'stack' | 'flask';
    detail: string;
  }

  let selected: string | null = null;
  let root: HTMLElement;

  function equippedIn(item: number): number[] {
    return state.slots.flatMap((n, slot) => (n === item ? [slot] : []));
  }

  $: rows = [
    ...state.gear.map<Row>((item) => ({
      key: `g${item}`,
      item,
      name: host.itemName(item),
      kind: 'gear',
      detail: ''
    })),
    ...state.flaskItems
      .filter((n) => n > 0)
      .map<Row>((item, i) => ({
        key: `f${item}`,
        item,
        name: host.itemName(item),
        kind: 'flask',
        detail: i === 0 ? `${state.flasks.vigil} / ${state.flasks.vigilMax}` : `${state.flasks.lucid} / ${state.flasks.maxCharges - state.flasks.vigilMax}`
      })),
    ...state.stacks.map<Row>(({ item, count }) => ({
      key: `s${item}`,
      item,
      name: host.itemName(item),
      kind: 'stack',
      detail: `×${count}`
    }))
  ];
  $: sections = [
    { title: 'Weapons and gear', rows: rows.filter((r) => r.kind === 'gear') },
    { title: 'Flasks', rows: rows.filter((r) => r.kind === 'flask') },
    { title: 'Consumables', rows: rows.filter((r) => r.kind === 'stack') }
  ].filter((s) => s.rows.length > 0);
  $: row = rows.find((r) => r.key === selected) ?? null;
  $: where = row ? equippedIn(row.item) : [];
  $: quickAssign = row && row.kind !== 'gear'
    ? [0, 1, 2, 3, 4].map((i) => ({ i, slot: SLOT.QUICK_1 + i, held: state.slots[SLOT.QUICK_1 + i], ok: !!host.previewEquip(SLOT.QUICK_1 + i, row.item)?.allowed }))
    : [];

  function slotName(slot: number): string {
    return SLOT_DEFS[slot].label;
  }

  async function select(r: Row): Promise<void> {
    selected = r.key;
    announce(`${r.name}${r.detail ? `, ${r.detail}` : ''}`);
    await tick();
    root?.querySelector<HTMLElement>('[data-nav="detail"]')?.focus();
  }

  async function assign(i: number): Promise<void> {
    if (!row) return;
    host.equip(SLOT.QUICK_1 + i, row.item);
    refresh();
    announce(`${row.name} set in quick slot ${i + 1}`);
    await tick();
    root?.querySelector<HTMLElement>('[data-nav="detail"]')?.focus();
  }

  function use(i: number): void {
    if (!row) return;
    host.useItem(i);
    announce(`Using ${row.name}`);
    useAndClose();
  }

  /** From the detail pane, back to the list. */
  export function back(): boolean {
    if (selected === null) return false;
    const key = selected;
    selected = null;
    void tick().then(() => root?.querySelector<HTMLElement>(`[data-key="${key}"]`)?.focus());
    return true;
  }
</script>

<div class="cols" bind:this={root}>
  <div>
    {#each sections as s (s.title)}
      <div class="group">
        <h3>{s.title}</h3>
        <ul class="list" aria-label={s.title}>
          {#each s.rows as r (r.key)}
            <li>
              <button
                type="button"
                data-nav="items"
                data-key={r.key}
                aria-pressed={selected === r.key}
                aria-label={r.detail ? `${r.name}, ${r.detail}` : r.name}
                on:click={() => select(r)}
              >
                <span>{r.name}</span>
                {#if r.detail}<span class="sub">{r.detail}</span>{/if}
              </button>
            </li>
          {/each}
        </ul>
      </div>
    {/each}
    {#if sections.length === 0}
      <p class="note">You are carrying nothing.</p>
    {/if}
  </div>

  <div>
    {#if row}
      <div class="group" aria-label={`${row.name} details`}>
        <h3>{row.name}</h3>
        <p class="note">
          {#if where.length}
            In use: {where.map(slotName).join(', ')}.
          {:else}
            Not equipped.
          {/if}
        </p>
        {#if row.kind === 'gear'}
          <p class="note">Equip it from the Equipment screen.</p>
          <p>
            <button type="button" data-nav="detail" on:click={() => go('equipment')}>Open Equipment</button>
          </p>
        {:else}
          <h3>Quick slots</h3>
          <ul class="list" aria-label="Quick slots">
            {#each quickAssign as q (q.i)}
              <li>
                {#if q.held === row.item}
                  <button type="button" data-nav="detail" on:click={() => use(q.i)}>
                    <span>Use from quick slot {q.i + 1}</span>
                  </button>
                {:else}
                  <button
                    type="button"
                    data-nav="detail"
                    aria-disabled={q.ok ? undefined : 'true'}
                    on:click={() => q.ok && assign(q.i)}
                  >
                    <span>Put in quick slot {q.i + 1}</span>
                    <span class="sub">{q.held ? `Replaces ${host.itemName(q.held)}` : 'Empty'}</span>
                  </button>
                {/if}
              </li>
            {/each}
          </ul>
          {#if row.kind === 'stack'}
            <p class="note">Only the flasks have an effect so far; the other consumables arrive later.</p>
          {/if}
        {/if}
      </div>
    {:else}
      <p class="note">Choose an item to see what you can do with it.</p>
    {/if}
    <div class="group">
      <h3>Flask potency</h3>
      <p class="note">
        +{state.flasks.level}. {state.flasks.maxCharges} charges in all, {state.flasks.vigilMax} for the Vigil Flask
        and {state.flasks.maxCharges - state.flasks.vigilMax} for the Lucid Flask. Set the split at a Lantern.
      </p>
    </div>
  </div>
</div>
