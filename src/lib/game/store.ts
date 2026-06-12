import { writable } from 'svelte/store';
import type { Section } from '$lib/content';

export type GameState = {
  started: boolean;
  pointerLocked: boolean;
  interactPrompt: string | null;
  openSection: Section | null;
  webglFailed: boolean;
  isTouch: boolean;
  textMode: boolean;
  health: number;
  hitCount: number; // increments on damage; drives the red flash
  deathCount: number; // increments on death; drives the death flash
  muted: boolean;
  dayNightCycle: boolean;
  ammo: number;
  reserve: number;
  reloading: boolean;
  score: number;
  streak: number;
  aiming: boolean;
  yaw: number;
  hitMarker: { id: number; headshot: boolean; killed: boolean } | null;
  dead: boolean;
  finalScore: number;
};

export const gameState = writable<GameState>({
  started: false,
  pointerLocked: false,
  interactPrompt: null,
  openSection: null,
  webglFailed: false,
  isTouch: false,
  textMode: false,
  health: 100,
  hitCount: 0,
  deathCount: 0,
  muted: false,
  dayNightCycle: false,
  ammo: 12,
  reserve: 24,
  reloading: false,
  score: 0,
  streak: 0,
  aiming: false,
  yaw: 0,
  hitMarker: null,
  dead: false,
  finalScore: 0
});
