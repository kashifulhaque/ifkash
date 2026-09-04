import { writable } from 'svelte/store';
import type { Wonder } from './wonders';

export type BiomeCaption = { name: string; kind: string; index: string; tagline: string };

/** Day-night clock as shown in the HUD. */
export type ClockState = { paused: boolean; night: boolean };

export type GameState = {
  ready: boolean;
  webglFailed: boolean;
  isTouch: boolean;
  /** True before the player's first move, while the camera shows the whole planet. */
  intro: boolean;
  globeView: boolean;
  biome: BiomeCaption | null;
  /** Nearby wonder or boat the player can interact with. */
  prompt: { id: string; action: string; found: boolean; kicker?: string } | null;
  found: string[];
  total: number;
  openWonder: Wonder | null;
  help: boolean;
  muted: boolean;
  /** Short fading notice, for example after finding a wonder. */
  toast: { id: number; text: string } | null;
  /** Label of the world seed, for example "2026-09-04" or "quiet-fox-73". */
  seed: string;
  clock: ClockState;
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
  muted: false,
  toast: null,
  seed: '',
  clock: { paused: false, night: false }
};

export const gameState = writable<GameState>({ ...initialState });
