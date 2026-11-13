// Music and ambience numbers for every region and boss. This file has no Web Audio import, so
// the tables can be read and checked as plain data. Region keys are the area ids the core uses
// (the folder names in `game/data/levels/`), and boss keys are the boss numbers; `bossKeyForName`
// maps the name on the HUD's boss bar to its key. All of it is generated in code: see
// `game/assets-src/audio/SOURCE.md`.

/** How loud each ambience layer runs in a region, 0 to 1. */
export interface BedMix {
  wind: number;
  water: number;
  drips: number;
  fire: number;
}

/** A region's calm exploration music: a pad under a plucked loop. */
export interface RegionVoice {
  mix: BedMix;
  /** MIDI note of the pad's root. */
  root: number;
  /** Pad chord, in semitones above the root. */
  chord: number[];
  /** Pad filter cutoff, in Hz. */
  tone: number;
  /** Tempo in beats per minute. The pluck plays eighth notes, so each step is half a beat. */
  bpm: number;
  /** One bar of eight eighth notes: semitones above the root plus an octave, or -1 for a rest. */
  pluck: number[];
}

/** Every region with a level folder. Adding a level means adding its key here (the compiler checks). */
export const REGION_IDS = [
  'waking_hollow',
  'verge',
  'hollowhearth',
  'bound_ring',
  'castle',
  'cinderdeep_mine',
  'drowned_belfry',
  'glasswing_grotto',
  'hollowdeep',
  'mechanics_yard',
  'mirrormere',
  'mossgut_cave',
  'rimeshelf',
  'spire',
  'sunken_ossuary',
  'test_field',
  'whitewarden'
] as const;

export type RegionId = (typeof REGION_IDS)[number];
export type RegionKey = RegionId | 'default';

export const REGION_VOICES: Record<RegionKey, RegionVoice> = {
  // Waking Hollow: a sheltered start, light wind and a trickle of water.
  waking_hollow: { mix: { wind: 0.35, water: 0.3, drips: 0.15, fire: 0 }, root: 50, chord: [0, 7, 12, 16], tone: 900, bpm: 72, pluck: [0, -1, 7, -1, 12, -1, 7, -1] },
  // The Verge: open and windy.
  verge: { mix: { wind: 0.7, water: 0.1, drips: 0, fire: 0 }, root: 53, chord: [0, 7, 12, 15], tone: 700, bpm: 88, pluck: [0, 3, 7, 10, 12, 10, 7, 3] },
  // Hollowhearth: the safe hub, a crackling hearth and almost no wind.
  hollowhearth: { mix: { wind: 0.05, water: 0, drips: 0.05, fire: 0.6 }, root: 48, chord: [0, 7, 12, 16], tone: 1100, bpm: 66, pluck: [0, -1, -1, 4, -1, 7, -1, 4] },
  // Bound Ring: a sealed circle of stones, hushed and minor.
  bound_ring: { mix: { wind: 0.2, water: 0, drips: 0, fire: 0 }, root: 45, chord: [0, 7, 12, 15], tone: 600, bpm: 60, pluck: [0, -1, 3, -1, 7, -1, 3, -1] },
  // Castle: stone halls and a distant wind.
  castle: { mix: { wind: 0.3, water: 0, drips: 0.05, fire: 0.1 }, root: 43, chord: [0, 7, 12, 14], tone: 900, bpm: 76, pluck: [0, 4, 7, 4, 0, -1, 2, -1] },
  // Cinderdeep Mine: dripping shafts and a low furnace glow.
  cinderdeep_mine: { mix: { wind: 0.1, water: 0, drips: 0.4, fire: 0.2 }, root: 40, chord: [0, 7, 12, 19], tone: 500, bpm: 64, pluck: [0, -1, -1, -1, 0, -1, 12, -1] },
  // Drowned Belfry: water below and wind through the bell louvres.
  drowned_belfry: { mix: { wind: 0.4, water: 0.5, drips: 0.3, fire: 0 }, root: 47, chord: [0, 7, 12, 16], tone: 1000, bpm: 70, pluck: [0, -1, 4, -1, 7, -1, 11, -1] },
  // Glasswing Grotto: bright and airy, with drips that ring.
  glasswing_grotto: { mix: { wind: 0.05, water: 0.3, drips: 0.5, fire: 0 }, root: 55, chord: [0, 4, 7, 11], tone: 1400, bpm: 92, pluck: [0, 4, 7, 11, 12, 11, 7, 4] },
  // Hollowdeep: the lowest, emptiest pit.
  hollowdeep: { mix: { wind: 0.1, water: 0, drips: 0.3, fire: 0 }, root: 38, chord: [0, 7, 12, 15], tone: 400, bpm: 56, pluck: [0, -1, -1, 3, -1, -1, 7, -1] },
  // Mechanics Yard: gears and a forge ticking over.
  mechanics_yard: { mix: { wind: 0.4, water: 0, drips: 0, fire: 0.3 }, root: 52, chord: [0, 7, 12, 14], tone: 800, bpm: 96, pluck: [0, -1, 7, -1, 0, -1, 5, -1] },
  // Mirrormere: still water and a faint, high shimmer.
  mirrormere: { mix: { wind: 0.1, water: 0.4, drips: 0.2, fire: 0 }, root: 57, chord: [0, 4, 7, 14], tone: 1300, bpm: 80, pluck: [0, 4, 7, 14, 12, 7, 4, 2] },
  // Mossgut Cave: wet and close.
  mossgut_cave: { mix: { wind: 0, water: 0.2, drips: 0.5, fire: 0 }, root: 41, chord: [0, 7, 12, 17], tone: 500, bpm: 62, pluck: [0, -1, 5, -1, 7, -1, 5, -1] },
  // Rimeshelf: an icy ledge with a hard wind.
  rimeshelf: { mix: { wind: 0.8, water: 0, drips: 0, fire: 0 }, root: 50, chord: [0, 5, 12, 17], tone: 1100, bpm: 68, pluck: [0, 5, 7, 12, 7, 5, 0, -1] },
  // Spire: high and open, with wind all the way up.
  spire: { mix: { wind: 0.6, water: 0, drips: 0, fire: 0 }, root: 58, chord: [0, 4, 7, 11], tone: 1600, bpm: 84, pluck: [0, 4, 7, 11, 7, 4, 2, -1] },
  // Sunken Ossuary: bones under black water.
  sunken_ossuary: { mix: { wind: 0, water: 0.1, drips: 0.6, fire: 0 }, root: 42, chord: [0, 6, 12, 15], tone: 450, bpm: 58, pluck: [0, -1, 6, -1, 3, -1, -1, -1] },
  // Test field: a plain bed for the development scenario.
  test_field: { mix: { wind: 0.3, water: 0, drips: 0, fire: 0 }, root: 50, chord: [0, 7, 12], tone: 800, bpm: 80, pluck: [0, -1, 7, -1, 12, -1, 7, -1] },
  // Whitewarden: a cold, watchful hall.
  whitewarden: { mix: { wind: 0.5, water: 0, drips: 0, fire: 0.1 }, root: 46, chord: [0, 7, 12, 16], tone: 700, bpm: 66, pluck: [0, -1, 3, -1, 7, -1, 10, -1] },
  // Used when an area has no voice of its own.
  default: { mix: { wind: 0.3, water: 0, drips: 0, fire: 0 }, root: 50, chord: [0, 7, 12], tone: 800, bpm: 72, pluck: [0, -1, 7, -1, 12, -1, 7, -1] }
};

/** A region's voice. Accepts hyphens or underscores, and falls back to the default voice. */
export function regionVoice(id: string): RegionVoice {
  const key = id.replace(/-/g, '_');
  return (REGION_IDS as readonly string[]).includes(key) ? REGION_VOICES[key as RegionId] : REGION_VOICES.default;
}

/**
 * How a boss's beat sounds on the loop stem:
 * - heavy: a kick on every beat and a snare on the bar's third beat.
 * - skitter: hats on the eighths, snares on two and four.
 * - chant: a low kick and a choral tone on one and three.
 * - drip: soft plucks between the beats.
 * - clatter: loose metallic clicks.
 * - swell: no percussion, only the drone.
 * - pulse: square eighths on the drone.
 * - toll: a bell on the bar's first beat and a lighter bell on the rest.
 */
export type BeatStyle = 'heavy' | 'skitter' | 'chant' | 'drip' | 'clatter' | 'swell' | 'pulse' | 'toll';

/** A boss's music: a loop stem, an intro, and a phase-2 stem layered over the loop. */
export interface BossVoice {
  /** Phase 1 tempo, in beats per minute. */
  bpm: number;
  /** Phase 2 tempo as a multiple of phase 1. */
  phase2Speed: number;
  /** MIDI note of the loop's bass drone. */
  drone: number;
  beat: BeatStyle;
  /** The phase-2 arpeggio, in semitones above the drone plus two octaves. */
  lead: number[];
  leadWave: OscillatorType;
  /** Beats come from the core's `BELL_BEAT` events, not the audio clock. */
  fromEvents?: boolean;
}

export const BOSS_IDS = ['b01', 'b02', 'b03', 'b04', 'b05', 'b06', 'b07', 'b08', 'b09', 'b10', 'b11', 'b12'] as const;
export type BossId = (typeof BOSS_IDS)[number];
export type BossKey = BossId | 'default';

/** Each boss's name on the HUD's boss bar, as in `game/data/bosses/`. */
export const BOSS_NAMES: Record<BossId, string> = {
  b01: 'The Hollow Sentry',
  b02: 'The Briar Stag',
  b03: 'Bridgewarden Corvane',
  b04: 'The Mossgut Brood',
  b05: 'The Ossuary Keeper',
  b06: 'The Drowned Choir',
  b07: 'Castellan Voss, the Overfit Warden',
  b08: 'The Forgeheart Colossus',
  b09: 'The Lamplight Matriarch',
  b10: 'The Bellwright',
  b11: 'The Bound Exile',
  b12: 'The Unspooled'
};

export const BOSS_VOICES: Record<BossKey, BossVoice> = {
  // B01: a slow, heavy march.
  b01: { bpm: 96, phase2Speed: 1.25, drone: 38, beat: 'heavy', lead: [0, 3, 7, 10], leadWave: 'square' },
  // B02: a skittering charge.
  b02: { bpm: 116, phase2Speed: 1.25, drone: 40, beat: 'skitter', lead: [0, 5, 7, 12], leadWave: 'sawtooth' },
  // B03: a lordly chant over a low kick.
  b03: { bpm: 88, phase2Speed: 1.25, drone: 43, beat: 'chant', lead: [0, 4, 7, 9], leadWave: 'triangle' },
  // B04: wet, uneven plucks.
  b04: { bpm: 100, phase2Speed: 1.25, drone: 41, beat: 'drip', lead: [0, 2, 5, 7], leadWave: 'triangle' },
  // B05: loose bone clatter.
  b05: { bpm: 84, phase2Speed: 1.25, drone: 36, beat: 'clatter', lead: [0, 3, 6, 9], leadWave: 'square' },
  // B06: a sung drone with no percussion.
  b06: { bpm: 72, phase2Speed: 1.25, drone: 45, beat: 'swell', lead: [0, 4, 7, 11, 12], leadWave: 'sine' },
  // B07: a stiff, dissonant beat.
  b07: { bpm: 108, phase2Speed: 1.25, drone: 39, beat: 'heavy', lead: [0, 1, 6, 7], leadWave: 'sawtooth' },
  // B08: a forge's hammer.
  b08: { bpm: 92, phase2Speed: 1.25, drone: 33, beat: 'heavy', lead: [0, 7, 12, 7], leadWave: 'sawtooth' },
  // B09: fast, pulsing flight.
  b09: { bpm: 128, phase2Speed: 1.25, drone: 52, beat: 'pulse', lead: [0, 4, 9, 12], leadWave: 'square' },
  // B10: the beat comes from the core's events. Phase 1 is 40 ticks a beat at 30 ticks a second (80 BPM).
  b10: { bpm: 80, phase2Speed: 1.25, drone: 38, beat: 'toll', lead: [0, 3, 7, 10], leadWave: 'triangle', fromEvents: true },
  // B11: a soft, floating swell.
  b11: { bpm: 76, phase2Speed: 1.25, drone: 47, beat: 'swell', lead: [0, 2, 7, 9, 14], leadWave: 'sine' },
  // B12: a fast, threaded skitter.
  b12: { bpm: 120, phase2Speed: 1.25, drone: 37, beat: 'skitter', lead: [0, 5, 10, 14], leadWave: 'square' },
  // Used for a boss with no entry of its own, and the test boss.
  default: { bpm: 90, phase2Speed: 1.25, drone: 40, beat: 'heavy', lead: [0, 3, 7, 10], leadWave: 'square' }
};

/** The boss key for a HUD boss-bar name. Unknown names get the default voice. */
export function bossKeyForName(name: string): BossKey {
  const hit = (BOSS_IDS as readonly BossId[]).find((id) => BOSS_NAMES[id] === name);
  return hit ?? 'default';
}
