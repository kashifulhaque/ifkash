// Named spawn points for `&spawn=ID`, per scenario. Only x, z and yaw are listed: the
// height comes from a ray cast down onto the level, so the table can't go stale when the
// terrain moves. Yaw is in radians, 0 facing +Z and turning toward +X. A scenario's own
// start is always available as `spawn`.

export interface SpawnPoint {
  x: number;
  z: number;
  yaw: number;
  note: string;
}

const HALF_PI = Math.PI / 2;

const TEST_FIELD: Record<string, SpawnPoint> = {
  ramp_foot: { x: 0, z: 8, yaw: 0, note: 'foot of the ramp' },
  ramp_top: { x: 0, z: 27, yaw: 0, note: 'top of the ramp, facing the ledge wall' },
  ledge_top: { x: 0, z: 32, yaw: 0, note: 'on top of the ledge' },
  drop_edge: { x: 0, z: 38.6, yaw: 0, note: 'at the 10 m drop' },
  ridge: { x: 24, z: -9, yaw: 0, note: 'before the steep ridge' },
  steps: { x: -23, z: 1.2, yaw: 0, note: 'before the four steps' },
  dummies: { x: 0, z: 3, yaw: 0, note: 'in front of the three practice dummies' },
  east_edge: { x: 30, z: 10, yaw: -HALF_PI, note: 'near the field edge' }
};

// Waking Hollow (WP-043): the rooms of the tutorial cave, south to north.
const WAKING_HOLLOW: Record<string, SpawnPoint> = {
  lantern: { x: 0, z: -2, yaw: 0, note: 'beside Lantern L01' },
  jump: { x: 0, z: 11, yaw: 0, note: 'corridor 1, before the jump ledge' },
  fight: { x: 0, z: 24, yaw: 0, note: 'the first fight, facing the husk' },
  alcove: { x: 9, z: 30, yaw: 1.5707964, note: 'the salts alcove' },
  hall: { x: 0, z: 46, yaw: 0, note: 'the dodge hall, before the two husks' },
  crack: { x: -6, z: 52, yaw: -1.5707964, note: 'facing the cracked wall' },
  twist: { x: 0, z: 69, yaw: 0, note: 'the ambush room' },
  antechamber: { x: 0, z: 89, yaw: 0, note: 'the safe room before the fog gate' },
  gate: { x: 0, z: 95, yaw: 0, note: 'at the B01 fog gate' },
  arena: { x: 0, z: 100, yaw: 0, note: 'inside the B01 arena' }
};

// The Verge (WP-044): each Lantern, the landmarks, and the side branches. Lantern spawns
// stand one meter or two off the Lantern.
const VERGE: Record<string, SpawnPoint> = {
  mouth: { x: -110, z: -120, yaw: 0.8569, note: 'the hollow mouth, the area start' },
  l02: { x: -40, z: -62.5, yaw: 0, note: 'beside L02, Verge Crossroads, facing the Spire' },
  l03: { x: 7.5, z: -30, yaw: HALF_PI, note: 'beside L03, Aqueduct Steps, facing the ramp' },
  ramp_top: { x: 42, z: -30, yaw: HALF_PI, note: 'top of the aqueduct ramp, on the deck' },
  l04: { x: 90, z: 33.5, yaw: 0, note: 'beside L04, in Ninefold Chapel' },
  altar: { x: 90, z: 41.5, yaw: 0, note: 'before the chapel altar and the raised ladder' },
  crypt: { x: 116, z: 46, yaw: HALF_PI, note: 'in the crypt, facing the hollow wall' },
  roof: { x: 90, z: 47, yaw: 0, note: 'on the chapel roof platform, above the raised ladder' },
  l05: { x: -20, z: 53.5, yaw: 0, note: 'beside L05, Mossgut Mouth' },
  windmill: { x: -86, z: 10, yaw: 0, note: 'on the Windmill Rise, before the tower' },
  thicket: { x: 104, z: -80, yaw: HALF_PI, note: 'the Briar Thicket lane, facing the fog gate' },
  arena: { x: 118, z: -80, yaw: HALF_PI, note: 'inside the B02 arena, past the gate gap' },
  spire_road: { x: -40, z: 160, yaw: 0, note: 'the Spire Approach, at the Spire base' },
  spire_view: { x: -40, z: -64, yaw: 0, note: 'the landmark view: L02 looking at the Spire' }
};

// Hollowhearth (WP-068): the spot in front of each NPC's counter and each station, facing it.
const HOLLOWHEARTH: Record<string, SpawnPoint> = {
  lantern: { x: 0, z: -21, yaw: 0, note: 'beside Lantern L00, facing the Hearth' },
  hearth: { x: 0, z: -2.6, yaw: 0, note: 'at the Hearth and its inscription' },
  archivist: { x: -21, z: 15, yaw: 0, note: 'at the Archivist\'s desk' },
  archive: { x: -25, z: 8, yaw: -HALF_PI, note: 'at the Archive shelves' },
  keepsakes: { x: -21.6, z: -8, yaw: -HALF_PI, note: 'at the keepsake table' },
  cartographer: { x: -14, z: -13.5, yaw: Math.PI, note: 'at the Cartographer\'s map table' },
  smith: { x: 20, z: -6, yaw: HALF_PI, note: 'at the Smith\'s counter' },
  courier: { x: 20, z: 12, yaw: HALF_PI, note: 'at the Courier\'s counter' },
  peddler: { x: 10, z: -17, yaw: Math.PI, note: 'at the Peddler\'s counter' },
  ledger: { x: 8, z: 22.4, yaw: 0, note: 'at the Ledger Stand' },
  mirror: { x: -8, z: 23.8, yaw: 0, note: 'before the Mirror' },
  overview: { x: 0, z: -14, yaw: 0, note: 'mid-hall, looking north across the hub' }
};

// Mirrormere (WP-060): each Lantern, the landmark view, and the side branches. Lantern spawns
// stand one meter or two off the Lantern. Yaw is from +Z toward +X.
const MIRRORMERE: Record<string, SpawnPoint> = {
  road: { x: -120, z: -140, yaw: 1.2217, note: 'the Verge road, the area start' },
  l06: { x: -74, z: -126, yaw: 0.8, note: 'beside L06, Reedwater Landing, facing the Cathedral tower across the lake' },
  landmark: { x: -74, z: -126, yaw: 0.8, note: 'the landmark view: L06 looking at the Drowned Cathedral' },
  l07: { x: -78, z: -31, yaw: HALF_PI, note: 'beside L07, Stillwater Village, facing the lake' },
  village: { x: -88, z: -34, yaw: -HALF_PI, note: 'the flooded village street, facing the Drowned' },
  chief: { x: -92, z: -10, yaw: -HALF_PI, note: "at the chief's house door, Chronicle Page 2 inside" },
  islet: { x: -131, z: -27, yaw: -HALF_PI, note: 'behind the village at the rowboat, facing the islet' },
  marsh: { x: -74, z: 34, yaw: 0, note: 'the road beside the Reed Marsh' },
  l08: { x: -44, z: 113.5, yaw: 0, note: 'beside L08, Ossuary Stair' },
  causeway: { x: 85, z: 98, yaw: Math.PI, note: 'the head of the causeway, facing the Cathedral' },
  l09: { x: 85, z: 59, yaw: Math.PI, note: 'beside L09, Cathedral Causeway, facing the nave doors' },
  forecourt: { x: 85, z: 26, yaw: Math.PI, note: "the Cathedral forecourt, before the nave's fog gate" },
  nave: { x: 86, z: 8, yaw: Math.PI, note: "inside B06's nave, facing the apse" },
  ford_west: { x: -64, z: 23, yaw: HALF_PI, note: 'the village end of the ford (a wall until the sluice opens)' },
  side_door: { x: 66, z: 23, yaw: -HALF_PI, note: "the Cathedral's side door, where the ford lands" }
};

const TABLES: Record<string, Record<string, SpawnPoint>> = {
  test_field: TEST_FIELD,
  waking_hollow: WAKING_HOLLOW,
  verge: VERGE,
  hollowhearth: HOLLOWHEARTH,
  mirrormere: MIRRORMERE,
  m0_flat: {
    far: { x: 40, z: 40, yaw: Math.PI, note: 'far corner of the flat floor' }
  }
};

/** Lantern IDs will join this table when Lanterns exist (WP-029); until then spawns stand in. */
export function spawnIds(scenario: string): string[] {
  return ['spawn', ...Object.keys(TABLES[scenario] ?? {})];
}

/** The point named `id` in `scenario`, or null for the scenario's own start and unknown IDs. */
export function findSpawn(scenario: string, id: string): SpawnPoint | null {
  return TABLES[scenario]?.[id] ?? null;
}
