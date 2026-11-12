// Local saves: three IndexedDB slots holding the blob that `LfWorld.save()` makes
// (game/crates/core/src/save.rs), plus the autosave hooks. See
// docs/game/01-architecture.md, "Persistence".
//
// A blob is the persistent state only (character, inventory, flags, last Lantern). Loading
// one builds a fresh world with `LfWorld.fromSave(blob)`, so the slot record also keeps the
// scenario and seed for the host's camera and replay setup. Nothing here imports Three.js
// or the WASM module: a world is anything with `save()` and `persistent_hash()`.
//
// Autosave: `Autosave.onEventWords` watches the core's raw event words for the player
// resting at a Lantern, and `Autosave.onAreaChange` is for the level and menu work packages
// to call when the player enters a new area (no area-change event exists yet).

import { EVENT_WORDS } from './events';

export const SLOT_COUNT = 3;
/** The slot autosaves go to until the player picks another. */
export const AUTOSAVE_SLOT = 0;

const DB_NAME = 'latticefall-saves';
const STORE = 'slots';
const DB_VERSION = 1;

/** Event kind and Life code of "the player rested at a Lantern" (see `drain_events`). */
const EVENT_LIFE = 2;
const LIFE_RESTED = 3;

export type SaveReason = 'manual' | 'rest' | 'area';

/** What a slot menu shows without reading the blob. */
export interface SlotMeta {
  slot: number;
  /** Milliseconds since the epoch. */
  savedAt: number;
  scenario: string;
  /** The world's seed as a decimal string (it is a u64). */
  seed: string;
  /** `persistent_hash()` of the world when it was saved. */
  hash: string;
  reason: SaveReason;
}

export interface SaveRecord extends SlotMeta {
  data: Uint8Array;
}

/** The part of `LfWorld` saving needs. */
export interface SaveSource {
  save(): Uint8Array;
  persistent_hash(): string;
}

/** Where the running world came from; stored beside the blob. */
export interface WorldOrigin {
  scenario: string;
  seed: bigint;
}

function checkSlot(slot: number): void {
  if (!Number.isInteger(slot) || slot < 0 || slot >= SLOT_COUNT) {
    throw new RangeError(`save slot must be 0 to ${SLOT_COUNT - 1}, got ${slot}`);
  }
}

function request<T>(req: IDBRequest<T>): Promise<T> {
  return new Promise((resolve, reject) => {
    req.onsuccess = () => resolve(req.result);
    req.onerror = () => reject(req.error ?? new Error('IndexedDB request failed'));
  });
}

/** The three slots. Opens the database on first use. */
export class SaveStore {
  private db: Promise<IDBDatabase> | null = null;

  constructor(
    private readonly name = DB_NAME,
    private readonly factory: IDBFactory = indexedDB
  ) {}

  private open(): Promise<IDBDatabase> {
    this.db ??= new Promise<IDBDatabase>((resolve, reject) => {
      const req = this.factory.open(this.name, DB_VERSION);
      // Future schema changes bump DB_VERSION and migrate here, keyed on `event.oldVersion`.
      req.onupgradeneeded = () => {
        if (!req.result.objectStoreNames.contains(STORE)) req.result.createObjectStore(STORE, { keyPath: 'slot' });
      };
      req.onsuccess = () => resolve(req.result);
      req.onerror = () => reject(req.error ?? new Error('cannot open the save database'));
    }).catch((err) => {
      this.db = null;
      throw err;
    });
    return this.db;
  }

  private async store(mode: IDBTransactionMode): Promise<IDBObjectStore> {
    return (await this.open()).transaction(STORE, mode).objectStore(STORE);
  }

  /** The record in `slot`, or null when it is empty. */
  async read(slot: number): Promise<SaveRecord | null> {
    checkSlot(slot);
    const rec = await request((await this.store('readonly')).get(slot) as IDBRequest<SaveRecord | undefined>);
    return rec ?? null;
  }

  async write(record: SaveRecord): Promise<void> {
    checkSlot(record.slot);
    await request((await this.store('readwrite')).put(record));
  }

  async clear(slot: number): Promise<void> {
    checkSlot(slot);
    await request((await this.store('readwrite')).delete(slot));
  }

  /** One entry per slot, null for an empty one, without the blobs. */
  async list(): Promise<(SlotMeta | null)[]> {
    const out: (SlotMeta | null)[] = [];
    for (let slot = 0; slot < SLOT_COUNT; slot++) {
      const rec = await this.read(slot);
      if (!rec) {
        out.push(null);
        continue;
      }
      const { data: _data, ...meta } = rec;
      out.push(meta);
    }
    return out;
  }

  close(): void {
    void this.db?.then((db) => db.close());
    this.db = null;
  }
}

/** Saves the running world and reads slots back. */
export class SaveManager {
  /** The slot autosaves go to. */
  activeSlot = AUTOSAVE_SLOT;
  private chain: Promise<unknown> = Promise.resolve();

  constructor(
    readonly store: SaveStore,
    private readonly source: SaveSource,
    private readonly origin: WorldOrigin,
    /** Called when an autosave fails, since nothing else is waiting on its promise. */
    private readonly onError: (err: unknown) => void = () => {}
  ) {}

  /**
   * Captures the world now and writes it to `slot`. The blob is taken before the first
   * await, so it is the state at the call even if the write queues behind another. Writes
   * run one at a time, so overlapping autosaves never interleave.
   */
  save(slot: number, reason: SaveReason = 'manual'): Promise<SlotMeta> {
    checkSlot(slot);
    const record: SaveRecord = {
      slot,
      savedAt: Date.now(),
      scenario: this.origin.scenario,
      seed: this.origin.seed.toString(),
      hash: this.source.persistent_hash(),
      reason,
      data: this.source.save()
    };
    const done = this.chain.then(() => this.store.write(record));
    this.chain = done.catch(() => {});
    return done.then(() => {
      const { data: _data, ...meta } = record;
      return meta;
    });
  }

  /** The record in `slot` for the host to hand to `startEngine({ save })`, or null. */
  load(slot: number): Promise<SaveRecord | null> {
    return this.store.read(slot);
  }

  private autosave(reason: SaveReason): void {
    this.save(this.activeSlot, reason).catch(this.onError);
  }

  /**
   * Scans the words from `LfWorld.drain_events()` for the player resting at a Lantern and
   * autosaves when there is one. Call it after the tick that drained them, so the save
   * holds the state the rest produced.
   */
  onEventWords(words: ArrayLike<number>): void {
    if (restedIn(words)) this.autosave('rest');
  }

  /** For the level and menu work packages: the player has entered a new area. */
  onAreaChange(): void {
    this.autosave('area');
  }
}

/** True when `words` holds a Life event with the "rested" code. */
export function restedIn(words: ArrayLike<number>): boolean {
  let i = 0;
  while (i < words.length) {
    const len = EVENT_WORDS[words[i]];
    if (len === undefined || i + len > words.length) return false;
    if (words[i] === EVENT_LIFE && words[i + 1] === LIFE_RESTED) return true;
    i += len;
  }
  return false;
}
