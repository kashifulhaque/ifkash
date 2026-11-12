// Named spawn points for `&spawn=ID`, per scenario. Only x, z and yaw are listed: the
// height comes from a ray cast down onto the level, so the table can't go stale when the
// terrain moves. Yaw is in radians, 0 facing +Z and turning toward +X. A scenario's own
// start is always available as `spawn`.

export interface SpawnPoint {
  x: number;
  z: number;
  yaw: number;
  note: string;
  /**
   * The feet height, for a floor under another floor (the castle's cliff path under the bridge,
   * its cistern under the courtyard, and the Spire's stacked floors). Left out, the height comes
   * from a ray cast from above, which lands on the topmost floor.
   */
  y?: number;
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

// Sunken Ossuary (WP-061): the key rooms, in critical-path order.
const SUNKEN_OSSUARY: Record<string, SpawnPoint> = {
  stair: { x: 0, z: -72, yaw: 0, note: 'the foot of the stair, facing the entry hall' },
  basin: { x: 0, z: -50, yaw: 0, note: 'the teaching room, at the deep basin (lever 1 on the left)' },
  hall: { x: 0, z: -29, yaw: 0, note: 'the doorway of the Ossuary Hall, the sleeping Drowned ahead' },
  lever_room: { x: 24, z: -16, yaw: HALF_PI, note: 'the lever room (lever 2 drains the Flooded Crypt)' },
  branch: { x: -20, z: -16, yaw: -HALF_PI, note: 'the branch crypt, its hollow wall ahead' },
  crypt: { x: 0, z: -8, yaw: 0, note: 'the doorway of the Flooded Crypt, deep until lever 2' },
  corridor: { x: 22, z: 10, yaw: HALF_PI, note: "the shortcut corridor's mouth, east of the Flooded Crypt" },
  vault: { x: 0, z: 20, yaw: 0, note: "before the Keeper's Vault's fog gate" },
  arena: { x: 0, z: 30, yaw: 0, note: "inside B05's vault, facing the far dais" }
};

// Drowned Belfry (WP-061): the key rooms, in critical-path order.
const DROWNED_BELFRY: Record<string, SpawnPoint> = {
  stair: { x: 0, z: -72, yaw: 0, note: 'the foot of the stair, facing the landing' },
  bell_room: { x: 0, z: -50, yaw: 0, note: 'the teaching room, bell A ahead' },
  gallery: { x: 0, z: -28, yaw: 0, note: 'the Choir Gallery, bell B in the middle of the choir' },
  tide: { x: 0, z: -2, yaw: 0, note: 'the Tide Hall: the causeway between the two channels' },
  shaft: { x: 14, z: 9, yaw: HALF_PI, note: "the bell shaft's gate, beside the lift's doorway" },
  shaft_bottom: { x: 26, z: 9, yaw: -HALF_PI, note: "the shaft's floor, 12 m down, facing the lift" },
  loft: { x: 0, z: 26, yaw: 0, note: 'the Belfry Loft, before the Choir Knight and bell D' },
  sluice: { x: 12, z: 46, yaw: Math.PI, note: 'beside the sluice lever, behind the Knight' }
};

// Castle Graymantle, part 1 (WP-064): the Lanterns, the bridge and B03's arena, the cliff path,
// the Barbican, and the Lower Ward. `y` is set where one floor stands under another.
const CASTLE: Record<string, SpawnPoint> = {
  road: { x: -20, z: -62, yaw: 0.2187, note: 'the arrival road, facing the bridge' },
  l14: { x: -12, z: -23.5, yaw: 0, note: 'beside L14, Graymantle Bridge, facing the bridge' },
  landmark: { x: -12, z: -23.5, yaw: 0, note: 'the landmark view: L14 looking at the bridge and the Barbican (use a fixed camera)' },
  bridge_head: { x: 0, z: -8, yaw: 0, note: 'the head of the bridge, before the two pylons' },
  gate: { x: 0, z: 0.5, yaw: 0, note: "at B03's fog gate (the plane z = 3.5)" },
  arena: { x: 0, z: 10, yaw: 0, note: "inside B03's arena, on the bridge, facing north" },
  cliff_top: { x: 0, z: -22, yaw: 0, note: 'the top of the cut stair down to the cliff path' },
  shelf: { x: 0, y: -8, z: 20, yaw: 0, note: 'on the cliff path under the bridge, facing north' },
  l15: { x: -4, y: -8, z: 71.5, yaw: 0, note: 'beside L15, the Barbican cistern, facing the Lantern' },
  cistern_stair: { x: 9, y: -8, z: 73, yaw: 0, note: "at the foot of the cistern's stair" },
  lever: { x: -3, z: 77.5, yaw: Math.PI, note: 'in the gate tunnel, facing the gatehouse lever and the main gate' },
  guard_room: { x: 16, z: 73, yaw: Math.PI, note: 'inside the guard room (the Castle Kite Shield, the Bulwark Charm)' },
  forecourt: { x: 9, z: 98, yaw: 0, note: 'the Barbican forecourt, facing the inner arch' },
  ward: { x: 0, z: 118, yaw: 0, note: 'the Lower Ward, just inside the inner arch' },
  armory: { x: 21, z: 134, yaw: HALF_PI, note: "at the armory's door, facing the Mimic's chest" },
  armory_inside: { x: 27, z: 134, yaw: HALF_PI, note: 'inside the armory, 4 m from the chest that is a Mimic' },
  kennel: { x: -22, z: 131, yaw: -HALF_PI, note: "at the kennel's gate, facing the Thorn Hounds" },
  chapel: { x: 19, z: 173, yaw: HALF_PI, note: "at the chapel's door, facing the altar" },
  ladder: { x: -44, z: 130, yaw: -HALF_PI, note: "at the wall-walk ladder's foot" },
  wall_walk: { x: -49.5, z: 125, yaw: 0, note: 'on the wall-walk, facing the ballista' },
  stair: { x: 0, z: 186, yaw: 0, note: 'at the foot of the keep stair, before the two Gray Knights' },
  landing: { x: 0, z: 217, yaw: 0, note: "the keep landing, where part 2's way on begins" }
};

// The Cinderdeep Mine (WP-062): the rooms of D4, north (+Z) and down. Lantern spawns stand a meter
// or two off the Lantern.
const CINDERDEEP_MINE: Record<string, SpawnPoint> = {
  mouth: { x: 0, z: -92, yaw: 0, note: 'the Verge mouth, the area start' },
  cart: { x: 1, z: -50, yaw: 0, note: 'beside the ride cart at the top of the gallery' },
  gallery: { x: 0, z: -4, yaw: 0, note: 'mid-gallery, facing down the slope (the Miner is by the rail)' },
  l10: { x: 2, z: 64, yaw: Math.PI, note: 'beside L10, Cinderdeep Shaft, facing back up the gallery' },
  hall: { x: 0, z: 76, yaw: 0, note: 'the middle of the lower hall, facing the exit tunnel' },
  push: { x: -8.6, z: 77.4, yaw: -HALF_PI, note: 'beside the push cart, facing the sealed wall' },
  wall: { x: -29, z: 78, yaw: -HALF_PI, note: 'in the west corridor, facing the sealed wall' },
  chamber: { x: -44, z: 78, yaw: HALF_PI, note: 'inside the smoltorch chamber, facing the broken wall' },
  forge_gate: { x: 36, z: 82, yaw: HALF_PI, note: "the corridor to B08's fog gate" },
  forge: { x: 50, z: 82, yaw: HALF_PI, note: "inside B08's forge, facing the furnace side" },
  exit: { x: 0, z: 94, yaw: 0, note: 'the exit tunnel to Hollowdeep' }
};

// Hollowdeep (WP-062): each Lantern, the landmark view, the dark places, and the Root Stair.
const HOLLOWDEEP: Record<string, SpawnPoint> = {
  arrival: { x: -132, z: -128, yaw: HALF_PI, note: 'the arrival cave, the area start' },
  l11: { x: -62, z: -52, yaw: HALF_PI, note: 'beside L11, Fungal Shore, facing the lake and the Lamp Tree' },
  landmark: { x: -62, z: -52, yaw: HALF_PI, note: 'the landmark view: L11 looking across the Glowing Lake' },
  dark_shore: { x: -42, z: -88, yaw: HALF_PI, note: 'the shore east of the glowing band: dark, six paces of sight' },
  nook: { x: -108, z: -64, yaw: -HALF_PI, note: 'the dark nook west of the shore (no moss, the Lantern Sword)' },
  belfry: { x: -88, z: -52, yaw: -HALF_PI, note: "the Belfry lift's frame on the shore" },
  passage: { x: -80, z: 4, yaw: 0, note: 'the west passage, facing north' },
  l12: { x: -18, z: 58, yaw: 0, note: 'beside L12, Glasswing Mouth, facing the arch to the north' },
  mouth: { x: -16, z: 90, yaw: 0, note: "the Glasswing Mouth's arch to D5, in the dark" },
  ford: { x: 24, z: 44, yaw: Math.PI, note: "the head of the ford to the Lamp Tree's island (unlit)" },
  island: { x: 30, z: -42, yaw: Math.PI, note: "the Lamp Tree's island, under the cap" },
  l13: { x: 100, z: 68, yaw: 0, note: 'beside L13, Lamplight Hollow, facing the fog gate' },
  gate: { x: 104, z: 88, yaw: 0, note: "B09's fog gate, from the antechamber" },
  arena: { x: 104, z: 100, yaw: 0, note: "inside B09's cavern, facing the six lamps" },
  east_shore: { x: 73, z: 20, yaw: Math.PI, note: 'the east shore, facing the Shard Golem' },
  stair: { x: 86, z: -34, yaw: HALF_PI, note: "the foot of the Root Stair, looking up it" },
  alcove: { x: 104, z: -22, yaw: 0, note: "the Root Stair's alcove (the endark relic)" },
  hatch: { x: 144, z: -34, yaw: HALF_PI, note: "the top of the Root Stair: the ladder's foot, under the hatch" },
  ledge: { x: 148, z: -34, yaw: HALF_PI, note: "the ledge above the hatch (after it opens), where the Spire's connection is" }
};

// The Lattice Spire (WP-067): the gate, the spiral's gaps, each floor, the Lanterns, and the arena.
// `y` is the floor height, because a ray cast down from above would land on a higher floor.
const SPIRE: Record<string, SpawnPoint> = {
  l22: { x: 2, z: -26, yaw: 0, note: 'beside L22, Spire Foot, facing the gate and the tower' },
  gate: { x: 2, z: -17, yaw: 0, note: 'before the Spire Gate' },
  forecourt: { x: 1, z: -12, yaw: Math.PI, note: 'the forecourt, at the foot of the outer spiral' },
  gap1: { x: -12, z: -1, yaw: 0, y: 2.2, note: 'the first gap on the west side of the spiral, facing north' },
  gap2: { x: -6, z: 12, yaw: HALF_PI, y: 3.5, note: 'the second gap on the north side, facing east' },
  gap3: { x: 12, z: -3, yaw: Math.PI, y: 6.3, note: 'the third gap on the east side, facing south' },
  ringend: { x: 8, z: -12, yaw: 0, y: 6.9, note: "the top of the spiral, before floor 1's south door" },
  f1: { x: 7.5, z: -8, yaw: 0, y: 7, note: 'floor 1, at the foot of the first stair' },
  f2: { x: 7.5, z: 3, yaw: 0, y: 12, note: 'floor 2, at the top of the first stair' },
  f3: { x: -6, z: 7, yaw: Math.PI, y: 17, note: 'floor 3, beside the top of the second stair' },
  l23: { x: -4, z: -2, yaw: HALF_PI, y: 17, note: 'beside L23, The Loom, on floor 3' },
  corridor: { x: -4, z: -7, yaw: HALF_PI, y: 17, note: 'the stair corridor to the fog gate' },
  fog: { x: 6.5, z: -7, yaw: 0, y: 22, note: "the corridor's landing, facing B12's fog gate" },
  arena: { x: 0, z: 4, yaw: Math.PI, y: 22, note: "B12's roof, past the gate" }
};

// Mossgut Cave (WP-063): the rooms of D1, north (+Z). Every room but the mouth hall and the Knife's
// alcove is under a 2.0 m roof slab, so the heights are listed: a ray cast down from above would land
// on a slab. `y` is the floor.
const MOSSGUT_CAVE: Record<string, SpawnPoint> = {
  mouth: { x: 0, z: -92, y: 0, yaw: 0, note: 'the cave mouth, the area start (L05 is outside, in the Verge)' },
  gallery: { x: 0, z: -66, y: 0, yaw: 0, note: 'the Low Gallery, the teaching room: the ceiling at 2.0 m, one Crawler ahead' },
  spore: { x: 0, z: -40, y: 0, yaw: 0, note: "the Spore Hall's south end, facing the pillars" },
  branch: { x: -20, z: -27, y: 0, yaw: -HALF_PI, note: 'the side tunnel to the Nursery' },
  nursery: { x: -32, z: -20, y: 0, yaw: Math.PI, note: 'the Nursery, facing the slab with the Wick Bead' },
  wall: { x: -35, z: -30, y: 0, yaw: Math.PI, note: 'facing the hollow wall in the Nursery south wall' },
  closet: { x: -35, z: -41, y: 0, yaw: 0, note: 'in the closet behind the hollow wall' },
  warren: { x: 0, z: -8, y: 0, yaw: 0, note: "the Brood Warren's south door, the nests along the walls" },
  lever: { x: -3.5, z: 15.8, y: 0, yaw: 0, note: "beside the Warren's lever, the gate to the north shut" },
  gate_hall: { x: 0, z: 30, y: 0, yaw: 0, note: 'the Gate Hall, before the fog gate' },
  gate: { x: 0, z: 43, y: 0, yaw: 0, note: "B04's fog gate (placeholder: its frame is two posts)" },
  chamber: { x: 0, z: 52, y: 0, yaw: 0, note: "inside B04's chamber, facing the Brood's wall" },
  alcove: { x: 20.9, z: 75, y: 0, yaw: HALF_PI, note: "the Fungal Knife's alcove, open to the chimney" },
  lift: { x: 32, z: 59, y: 0, yaw: HALF_PI, note: "the lift's doorway: the platform at the bottom of an 18 m shaft" },
  pit: { x: 47, z: 59, y: -18, yaw: -HALF_PI, note: "the shaft's floor, east of the platform, facing the lift" },
  corridor: { x: 30, z: 34, y: 0, yaw: HALF_PI, note: 'the return corridor, running east from the Gate Hall' }
};

// Glasswing Grotto (WP-063): the rooms of D5, north (+Z).
const GLASSWING_GROTTO: Record<string, SpawnPoint> = {
  mouth: { x: 0, z: -92, yaw: 0, note: 'the crystal mouth, the area start (L12 is outside, in Hollowdeep)' },
  mirror: { x: -6, z: -62, yaw: HALF_PI, note: 'the Mirror Hall: the beam runs from the wall to the mirror and stops' },
  socket: { x: 6, z: -54, yaw: -HALF_PI, note: "beside the Mirror Hall's socket and its sealed door" },
  prism: { x: -8, z: -26, yaw: HALF_PI, note: 'the Prism Hall: three mirrors, two sockets' },
  nook_door: { x: -14, z: -23, yaw: -HALF_PI, note: "the Shard Nook's sealed tunnel in the west wall" },
  nook: { x: -34, z: -20, yaw: Math.PI, note: 'the Shard Nook, with the Estoc on its slab' },
  facets: { x: -8, z: 4, yaw: HALF_PI, note: 'the Gallery of Facets: the first mirror wakes the nests' },
  guardian: { x: 0, z: 32, yaw: 0, note: 'the Guardian Hall, from the door: two Stalkers and the Spear beyond' },
  plinth: { x: 0, z: 60, yaw: 0, note: "the Shardglass Spear's plinth" },
  corridor: { x: 30, z: 38, yaw: HALF_PI, note: 'the return corridor, running east from the Guardian Hall' }
};

const TABLES: Record<string, Record<string, SpawnPoint>> = {
  sunken_ossuary: SUNKEN_OSSUARY,
  drowned_belfry: DROWNED_BELFRY,
  test_field: TEST_FIELD,
  waking_hollow: WAKING_HOLLOW,
  verge: VERGE,
  hollowhearth: HOLLOWHEARTH,
  mirrormere: MIRRORMERE,
  castle: CASTLE,
  cinderdeep_mine: CINDERDEEP_MINE,
  hollowdeep: HOLLOWDEEP,
  spire: SPIRE,
  mossgut_cave: MOSSGUT_CAVE,
  glasswing_grotto: GLASSWING_GROTTO,
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
