import { writable } from 'svelte/store';
import type { Wonder } from './wonders';
import { emptyJournal, type Journal } from './journal';
import { MAX_FUEL, SHIP_PARTS, type SpaceDestination, type SpaceStatus } from './space';

export type BiomeCaption = { name: string; kind: string; index: string; tagline: string };

export type GameState = {
  ready: boolean;
  webglFailed: boolean;
  isTouch: boolean;
  /** True before the player's first move, while the camera shows the whole planet. */
  intro: boolean;
  globeView: boolean;
  biome: BiomeCaption | null;
  /** Nearby wonder, animal, boat, or starship the player can interact with. */
  prompt: { id: string; action: string; found: boolean; kicker?: string } | null;
  found: string[];
  total: number;
  openWonder: Wonder | null;
  help: boolean;
  /** Photo mode: the HUD and touch controls are hidden so the planet can be captured. */
  photo: boolean;
  muted: boolean;
  /** Short fading notice, for example after finding a wonder. */
  toast: { id: number; text: string; kicker?: string; link?: { href: string; label: string } } | null;
  /** Label of the world seed, for example "2026-09-04" or "quiet-fox-73". */
  seed: string;
  /** The traveller's journal overlay. */
  journalOpen: boolean;
  journal: Journal;
  /** Starlight shards collected on this planet. */
  shards: { found: number; total: number };
  /** Current ship inventory and planet summary. */
  space: SpaceStatus;
  /** The star-map overlay. */
  navigationOpen: boolean;
  /** Procedurally generated routes available from the current planet. */
  destinations: SpaceDestination[];
};

export const initialState: GameState = {
  ready: false,
  webglFailed: false,
  isTouch: false,
  intro: true,
  globeView: true,
  biome: null,
  prompt: null,
  found: [],
  total: 0,
  openWonder: null,
  help: false,
  photo: false,
  muted: false,
  toast: null,
  seed: '',
  journalOpen: false,
  journal: emptyJournal(),
  shards: { found: 0, total: 0 },
  space: {
    parts: 0,
    totalParts: SHIP_PARTS.length,
    crafted: false,
    fuel: 0,
    maxFuel: MAX_FUEL,
    planetName: 'Earth',
    planetKind: 'Homeworld',
    depth: 0,
    isEarth: true
  },
  navigationOpen: false,
  destinations: []
};

export const gameState = writable<GameState>({ ...initialState });
