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

const TABLES: Record<string, Record<string, SpawnPoint>> = {
  test_field: TEST_FIELD,
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
