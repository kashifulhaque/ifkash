import { writable } from 'svelte/store';
import type { Section, SectionId } from '$lib/content';
import type { Perk } from './perks';
import type { Mutator } from './mutators';

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
  combo: number; // consecutive kills inside the combo window
  comboMult: number; // current points multiplier (1..5)
  comboTimer: number; // 0..1 remaining fraction of the combo window (drives the bar)
  wave: number;
  waveKills: number;
  waveQuota: number;
  waveBanner: { id: number; wave: number } | null; // increments on wave start → banner flash
  scorePopups: { id: number; amount: number; mult: number; headshot: boolean }[];
  aiming: boolean;
  yaw: number;
  hitMarker: { id: number; headshot: boolean; killed: boolean } | null;
  dead: boolean;
  finalScore: number;
  // Feature 1 — section-completion objective
  sectionsOpened: SectionId[];
  allSectionsCleared: boolean;
  // Feature 2 — daily challenge
  daily: boolean;
  dailyDay: string | null; // YYYY-MM-DD of the active daily run
  mutators: Mutator[];
  // Feature 3 — between-wave perk picks
  perkOffer: Perk[] | null;
  perksTaken: string[];
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
  combo: 0,
  comboMult: 1,
  comboTimer: 0,
  wave: 1,
  waveKills: 0,
  waveQuota: 8,
  waveBanner: null,
  scorePopups: [],
  aiming: false,
  yaw: 0,
  hitMarker: null,
  dead: false,
  finalScore: 0,
  sectionsOpened: [],
  allSectionsCleared: false,
  daily: false,
  dailyDay: null,
  mutators: [],
  perkOffer: null,
  perksTaken: []
});
