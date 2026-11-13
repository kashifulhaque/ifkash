// Where each boss hazard lies, and the numbers the renderer needs to draw it. Render-only: the core
// owns the real values, and every number here is copied from the file named beside it. The events
// tell the renderer *that* a hazard started; these say *where* (the core doesn't send positions
// for lava, lamps, or ice, because they are fixed in the arena).
//
// If a number changes in the data, change it here too. The hazards are drawn from events and the
// drawn actors alone: nothing here reads or writes simulation state.

/** `MechanicEvent` sub-kinds (WASM kind 14) the hazards follow. See `drain_events` in the WASM crate. */
export const MECH = {
  /** B04: the last egg sac fell and the weak point is open. */
  WEAK_POINT_EXPOSED: 131,
  /** B08: phase 2's lava began to creep. */
  LAVA_STARTED: 171,
  /** B08: the lava burned the player. */
  LAVA_BURNED: 172,
  /** B09: the cavern's light was drained (phase 2). */
  MATRIARCH_DRAINED: 182,
  /** B09: lamp `index` lit. */
  LAMP_LIT: 183,
  /** B09: a lit lamp stunned her as she flew past; `index` is the lamp. */
  LAMP_STUN: 184,
  /** B09: the fight ended or reset and the lamps went out. */
  LAMPS_DOUSED: 185,
  /** B10: a beat of the fight; the toll's beat is 0. */
  BELL_BEAT: 190,
  /** B10: the bell winds up. */
  BELL_TOLL_WARNED: 191,
  /** B10: the bell tolled and the shockwave runs out. */
  BELL_TOLLED: 192,
  /** B10: the shockwave hit the player. */
  BELL_TOLL_HIT: 193,
  /** B10: ice patch `index` cracked. */
  ICE_CRACKED: 194,
  /** B10: ice patch `index` broke into freezing water. */
  ICE_BROKE: 195,
  /** B10: freezing water burned the player. */
  ICE_WATER_BURNED: 196,
  /** B10: the chain snapped (phase 2). */
  BELL_CHAIN_SNAPPED: 197
} as const;

/** Core tick rate. */
export const TICK_HZ = 30;

export type Vec3 = readonly [number, number, number];

/** B08, the Forgeheart Colossus (game/data/bosses/b08_forgeheart_colossus.ron, `params`). */
export const B08 = {
  /** The boss's spawn in `cinderdeep_mine` (levels/cinderdeep_mine/entities.ron); the lava edges are measured from it. */
  home: [62, -22, 72] as Vec3,
  /** The distance of each wall's lava edge from the spawn: west (-x), east (+x), south (-z), north (+z). */
  edges: { w: 18, e: 16, s: 18, n: 20 },
  /** Lava depth gained per tick, in meters (`lava_m_per_100_ticks` / 100). */
  perTick: 0.5 / 100,
  /** The deepest it gets (`lava_max_m`). */
  max: 7,
  /** The first lava phase starts below this fraction of HP; used only to tell a reset from a fight. */
  phase2Frac: 0.4
};

/** B09, the Lamplight Matriarch (bosses/b09_lamplight_matriarch.ron and levels/hollowdeep/layout.ron). */
export const B09 = {
  /** Her spawn in `hollowdeep`. */
  home: [112, 0, 120] as Vec3,
  /** Each lamp's offset from her spawn: x, z (`lamp_N_dx_m`, `lamp_N_dz_m`). */
  lamps: [
    [20, 0],
    [10, 17.32],
    [-10, 17.32],
    [-20, 0],
    [-10, -17.32],
    [10, -17.32]
  ] as ReadonlyArray<readonly [number, number]>,
  /** Height of a lamp's top above the floor (`lamp_height_m`). */
  lampHeight: 3,
  /** How far a lit lamp lights the ground (`lamp_light_m`). */
  lightRadius: 9
};

/** B10, the Bellwright (bosses/b10_the_bellwright.ron, `params`). */
export const B10 = {
  /** His spawn in `rimeshelf`. */
  home: [0, 6, 112] as Vec3,
  /** Shockwave speed in meters per tick (`toll_speed_m_per_tick`) and how far it runs (`toll_max_m`). */
  tollSpeed: 0.9,
  tollMax: 32,
  /** Ticks between the bell winding up and the toll (`toll_warn_ticks`). */
  tollWarn: 20,
  /** Radius of each ice patch (`ice_radius_m`). */
  iceRadius: 3.5,
  /** Each patch's offset from his spawn: x, z (`iceN_x_m`, `iceN_z_m`). */
  ice: [
    [-12, -6],
    [12, -6],
    [-12, 6],
    [12, 6]
  ] as ReadonlyArray<readonly [number, number]>
};

/** B02's intro: the Briar Stag waits at this spot in the `verge`, facing the fog gate (levels/verge/entities.ron). */
export const B02 = {
  home: [150, 0, -80] as Vec3,
  /** The scenario's boss index of B02 (the Verge has one boss). */
  boss: 0
};

/** B04 (registry numbers in `data/registry.ron`): the mother's body and her four egg sacs. */
export const B04 = {
  body: 33001,
  sacs: [33002, 33003, 33004, 33005] as readonly number[]
};
