// The menus' side of the engine: what they read from the core and the commands they send.
// No Three.js or WASM imports; `LfWorld` satisfies `MenuWorld` structurally. The menus change
// the world only through `command`, which queues a core `Command` (see `queue_command` in
// game/crates/wasm/src/lib.rs).

import {
  parseLevelPreview,
  parseLoadout,
  parseMenuState,
  type LevelPreview,
  type Loadout,
  type MenuState
} from './menuState';

/** The `kind` operand of `queue_command`, in the order of the core's `Command` enum. */
export const COMMAND = {
  Rest: 0,
  LevelUp: 1,
  Equip: 2,
  Unequip: 3,
  UseItem: 4,
  Gesture: 5,
  SplitFlasks: 6,
  Travel: 7
} as const;

/** The part of `LfWorld` the menus read. */
export interface MenuWorld {
  menu_state(): Float32Array;
  item_name(num: number): string;
  lantern_name(index: number): string;
  preview_equip(slot: number, item: number): Float32Array;
  loadout_stats(): Float32Array;
  attribute_preview(attribute: number): Float32Array;
}

/** How the engine reports to the menus. */
export interface MenuSink {
  /** A press of the menu or map action (Escape, Start, M). */
  ui(action: 'menu' | 'map'): void;
  /** The player rested at a Lantern: open the Lantern menu. */
  rested(): void;
}

export interface MenuHost {
  read(): MenuState;
  itemName(num: number): string;
  lanternName(index: number): string;
  /** The current loadout's numbers. */
  loadout(): Loadout | null;
  /** The numbers after putting `item` in `slot` (0 empties the slot). */
  previewEquip(slot: number, item: number): Loadout | null;
  previewLevel(attribute: number): LevelPreview | null;
  equip(slot: number, item: number): void;
  unequip(slot: number): void;
  /** Uses the item in quick slot `index`, 0 to 4. */
  useItem(index: number): void;
  levelUp(attribute: number): void;
  splitFlasks(vigil: number): void;
  travel(lantern: number): void;
}

export function createMenuHost(
  world: MenuWorld,
  sink: { command(kind: number, a: number, b: number): void }
): MenuHost {
  const names = new Map<number, string>();
  const send = (kind: number, a = 0, b = 0) => sink.command(kind, a, b);
  return {
    read: () => parseMenuState(world.menu_state()),
    itemName(num) {
      let n = names.get(num);
      if (n === undefined) {
        n = world.item_name(num) || `Item ${num}`;
        names.set(num, n);
      }
      return n;
    },
    lanternName: (i) => world.lantern_name(i),
    loadout: () => parseLoadout(world.loadout_stats()),
    previewEquip: (slot, item) => parseLoadout(world.preview_equip(slot, item)),
    previewLevel: (a) => parseLevelPreview(world.attribute_preview(a)),
    equip: (slot, item) => send(COMMAND.Equip, slot, item),
    unequip: (slot) => send(COMMAND.Unequip, slot),
    useItem: (i) => send(COMMAND.UseItem, i),
    levelUp: (a) => send(COMMAND.LevelUp, a),
    splitFlasks: (v) => send(COMMAND.SplitFlasks, v),
    travel: (l) => send(COMMAND.Travel, l)
  };
}
