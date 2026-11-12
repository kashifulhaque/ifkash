// Display names for the places on the map: the 24 Lanterns of the checkpoint table and the
// areas (docs/game/02-world.md, "Checkpoints"). The core knows a Lantern only by its ID, so the
// names live here with the rest of the DOM text.

const LANTERNS: Record<string, string> = {
  lantern_l00: 'The Hearth',
  lantern_l01: 'Waking Hollow',
  lantern_l02: 'Verge Crossroads',
  lantern_l03: 'Aqueduct Steps',
  lantern_l04: 'Ninefold Chapel',
  lantern_l05: 'Mossgut Mouth',
  lantern_l06: 'Reedwater Landing',
  lantern_l07: 'Stillwater Village',
  lantern_l08: 'Ossuary Stair',
  lantern_l09: 'Cathedral Causeway',
  lantern_l10: 'Cinderdeep Shaft',
  lantern_l11: 'Fungal Shore',
  lantern_l12: 'Glasswing Mouth',
  lantern_l13: 'Lamplight Hollow',
  lantern_l14: 'Graymantle Bridge',
  lantern_l15: 'Barbican Cistern',
  lantern_l16: 'Windward Rampart',
  lantern_l17: "Castellan's Antechamber",
  lantern_l18: 'Rimeshelf Pass',
  lantern_l19: 'Whitewarden Gate',
  lantern_l20: "Bellwright's Steps",
  lantern_l21: 'The Bound Ring',
  lantern_l22: 'Spire Foot',
  lantern_l23: 'The Loom',
  lantern_field: 'Test field',
  lantern_yard: 'Mechanics yard'
};

const AREAS: Record<string, string> = {
  verge: 'The Verge',
  mirrormere: 'Mirrormere',
  sunken_ossuary: 'Sunken Ossuary',
  drowned_belfry: 'Drowned Belfry',
  hollowdeep: 'Hollowdeep',
  cinderdeep_mine: 'Cinderdeep Mine',
  mossgut_cave: 'Mossgut Cave',
  glasswing_grotto: 'Glasswing Grotto',
  castle: 'Castle Graymantle',
  spire: 'Lattice Spire',
  hollowhearth: 'Hollowhearth',
  waking_hollow: 'Waking Hollow',
  test_field: 'Test field',
  mechanics_yard: 'Mechanics yard'
};

function titleCase(id: string): string {
  return id
    .split('_')
    .filter(Boolean)
    .map((w) => w[0].toUpperCase() + w.slice(1))
    .join(' ');
}

/** The Lantern's name: "lantern_l05" is "Mossgut Mouth". An unknown ID is made readable. */
export function placeName(lanternId: string): string {
  return LANTERNS[lanternId] ?? (titleCase(lanternId.replace(/^lantern_/, '')) || 'Lantern');
}

/** The area's name: "sunken_ossuary" is "Sunken Ossuary". */
export function areaName(area: string): string {
  return AREAS[area] ?? (titleCase(area) || 'Unknown area');
}
