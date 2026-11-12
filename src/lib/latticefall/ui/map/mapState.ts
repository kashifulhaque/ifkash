// The map's side of the engine (WP-069): what it reads from the core, the commands it sends,
// and the pure helpers the map and its list view share. No Three.js or WASM imports; `LfWorld`
// satisfies `MapWorld` structurally. The core owns everything the map shows (the visited
// cells, the Lanterns, the markers); the DOM only reads it and sends `Command`s through
// `command`, which queues them (see `queue_command` in game/crates/wasm/src/lib.rs).

import { areaName, placeName } from './places';

/** The `kind` operand of `queue_command` for the map's commands. */
export const MAP_COMMAND = {
  /** The core's `Travel`: a Lantern of the same area, by its index there. */
  Travel: 7,
  /** The core's `TravelFar`: a place in another area, by its index in `places`. */
  TravelFar: 60,
  /** The core's `MapMarker`: toggles a marker at a point (decimeters, each plus 32768). */
  MapMarker: 61
} as const;

/** The most markers the core keeps, over all areas. */
export const MAX_MARKERS = 20;

/** The part of `LfWorld` the map reads. */
export interface MapWorld {
  map_json(): string;
}

export interface MapPlace {
  /** The place's index, which `TravelFar` takes. */
  index: number;
  area: string;
  /** The Lantern's ID, such as `lantern_l02`. */
  id: string;
  x: number;
  y: number;
  z: number;
  lit: boolean;
  /** The Lantern's index in the current area for `Travel`, or -1 in another area. */
  idx: number;
  here: boolean;
}

export interface MapBoss {
  name: string;
  x: number;
  z: number;
}

export interface MapView {
  /** The area the player is in. */
  area: string;
  /** Side of a map cell, in meters. */
  cell: number;
  /** `[minX, minZ, maxX, maxZ]` of the area, if known. */
  bounds: [number, number, number, number] | null;
  player: { x: number; z: number; yaw: number };
  /** The cells the player has stood in, as `[ix, iz]`. */
  cells: Array<[number, number]>;
  /** The ID of the Lantern being rested at, or null. */
  resting: string | null;
  remnant: { x: number; z: number; glimmer: number } | null;
  /** The markers in this area, in meters. */
  markers: Array<[number, number]>;
  /** Markers over all areas, against `MAX_MARKERS`. */
  markerTotal: number;
  /** Defeated bosses of this area. */
  bosses: MapBoss[];
  places: MapPlace[];
}

export function emptyMapView(): MapView {
  return {
    area: '',
    cell: 16,
    bounds: null,
    player: { x: 0, z: 0, yaw: 0 },
    cells: [],
    resting: null,
    remnant: null,
    markers: [],
    markerTotal: 0,
    bosses: [],
    places: []
  };
}

/** Parses the core's `map_json`; an empty view if it isn't valid. */
export function parseMapView(json: string): MapView {
  try {
    const raw = JSON.parse(json) as Record<string, unknown>;
    const v = emptyMapView();
    return {
      ...v,
      ...(raw as Partial<MapView>),
      places: ((raw.places as Array<Record<string, unknown>>) ?? []).map((p) => ({
        index: Number(p.i),
        area: String(p.area),
        id: String(p.id),
        x: Number(p.x),
        y: Number(p.y),
        z: Number(p.z),
        lit: p.lit === true,
        idx: Number(p.idx),
        here: p.here === true
      }))
    };
  } catch {
    return emptyMapView();
  }
}

export interface MapHost {
  read(): MapView;
  /** Travels to a lit place: a `Travel` in this area, a `TravelFar` in another. */
  travel(place: MapPlace): void;
  /** Toggles a marker at a point in meters: places one, or removes the one within 3 m. */
  toggleMarker(x: number, z: number): void;
}

export function createMapHost(
  world: MapWorld,
  sink: { command(kind: number, a: number, b: number): void }
): MapHost {
  return {
    read: () => parseMapView(world.map_json()),
    travel(place) {
      if (place.here) sink.command(MAP_COMMAND.Travel, place.idx, 0);
      else sink.command(MAP_COMMAND.TravelFar, place.index, 0);
    },
    toggleMarker(x, z) {
      const dm = (m: number) => Math.round(Math.min(3276, Math.max(-3276, m)) * 10) + 32768;
      sink.command(MAP_COMMAND.MapMarker, dm(x), dm(z));
    }
  };
}

// ---------------------------------------------------------------------------------------
// Pure helpers

const COMPASS = ['north', 'north-east', 'east', 'south-east', 'south', 'south-west', 'west', 'north-west'];

/** North is +Z and east is +X (the world map has north at the top). */
export function compass(dx: number, dz: number): string {
  const turns = Math.atan2(dx, dz) / (Math.PI / 4); // 0 north, 2 east
  return COMPASS[((Math.round(turns) % 8) + 8) % 8];
}

/** "about 40 m north-east", "right here" within 3 m. */
export function describeOffset(fromX: number, fromZ: number, toX: number, toZ: number): string {
  const dx = toX - fromX;
  const dz = toZ - fromZ;
  const d = Math.hypot(dx, dz);
  if (d < 3) return 'right here';
  const rounded = d < 100 ? Math.round(d / 5) * 5 : Math.round(d / 10) * 10;
  return `about ${rounded} m ${compass(dx, dz)}`;
}

/** Why the player can't travel to `place` now, or null if they can. */
export function travelBlock(view: MapView, place: MapPlace): string | null {
  if (view.resting === null) return 'Rest at a Lantern to travel';
  if (view.resting === place.id) return 'You are resting here';
  if (!place.lit) return 'Not lit yet';
  if (!place.here && view.remnant) return 'Recover your Remnant first';
  return null;
}

export interface PlaceGroup {
  area: string;
  name: string;
  places: MapPlace[];
}

/** The lit places grouped by area: the current area first, then the rest in order. */
export function groupPlaces(view: MapView): PlaceGroup[] {
  const groups = new Map<string, PlaceGroup>();
  for (const p of view.places) {
    if (!p.lit) continue;
    let g = groups.get(p.area);
    if (!g) {
      g = { area: p.area, name: areaName(p.area), places: [] };
      groups.set(p.area, g);
    }
    g.places.push(p);
  }
  return [...groups.values()].sort((a, b) => Number(b.area === view.area) - Number(a.area === view.area));
}

/** The text a list row says for a place: its name and where it is. */
export function placeSummary(view: MapView, p: MapPlace): string {
  const name = placeName(p.id);
  if (p.id === view.resting) return `${name}, Lantern lit, you are resting here`;
  if (p.here) return `${name}, Lantern lit, ${describeOffset(view.player.x, view.player.z, p.x, p.z)}`;
  return `${name}, Lantern lit, in ${areaName(p.area)}`;
}

/** The cells the map draws as uncovered: the visited cells and their eight neighbors. */
export function revealedCells(cells: Array<[number, number]>): { visited: Set<string>; halo: Array<[number, number]> } {
  const visited = new Set(cells.map(([x, z]) => `${x},${z}`));
  const seen = new Set<string>();
  const halo: Array<[number, number]> = [];
  for (const [x, z] of cells) {
    for (let dz = -1; dz <= 1; dz++) {
      for (let dx = -1; dx <= 1; dx++) {
        const k = `${x + dx},${z + dz}`;
        if (visited.has(k) || seen.has(k)) continue;
        seen.add(k);
        halo.push([x + dx, z + dz]);
      }
    }
  }
  return { visited, halo };
}

/** The rectangle the map shows, `[minX, minZ, maxX, maxZ]` in meters. */
export function mapExtent(view: MapView): [number, number, number, number] {
  if (view.bounds) return view.bounds;
  let [minX, minZ, maxX, maxZ] = [view.player.x - 40, view.player.z - 40, view.player.x + 40, view.player.z + 40];
  for (const [ix, iz] of view.cells) {
    minX = Math.min(minX, ix * view.cell);
    minZ = Math.min(minZ, iz * view.cell);
    maxX = Math.max(maxX, (ix + 1) * view.cell);
    maxZ = Math.max(maxZ, (iz + 1) * view.cell);
  }
  return [minX, minZ, maxX, maxZ];
}
