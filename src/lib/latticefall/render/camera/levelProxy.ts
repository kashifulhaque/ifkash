// Dev-only: a mesh of the static level built by raycasting down onto it, so camera
// screenshots show the walls the camera collides with. The real level renderer is a later
// work package; this exists only because nothing else draws the heightfield and boxes yet.
// Walls come out as steep one-cell slopes, which is fine for a collision proxy.

import { BufferAttribute, BufferGeometry, Color, Mesh, MeshStandardMaterial, Vector3 } from 'three';
import type { RaycastFn } from './CameraRig';

export interface LevelBounds {
  minX: number;
  maxX: number;
  minZ: number;
  maxZ: number;
}

/** The test field spans x -32..32 and z -32..64. */
export const TEST_FIELD_BOUNDS: LevelBounds = { minX: -32, maxX: 32, minZ: -32, maxZ: 64 };

const CELL = 0.5;
/** Parry 0.31 can miss a vertical ray that passes exactly through a heightfield vertex, so sample slightly off the grid. */
const JITTER_X = 0.0123;
const JITTER_Z = 0.0071;
const TOP = 80;
const VOID_Y = -4;

/** Samples the level's height on a grid and returns a mesh of it. Misses draw at `VOID_Y`. */
export function buildLevelProxy(raycast: RaycastFn, bounds: LevelBounds, floor: Color, wall: Color): Mesh {
  const cols = Math.round((bounds.maxX - bounds.minX) / CELL) + 1;
  const rows = Math.round((bounds.maxZ - bounds.minZ) / CELL) + 1;
  const origin = new Vector3();
  const down = new Vector3(0, -1, 0);
  const pos = new Float32Array(cols * rows * 3);
  for (let r = 0; r < rows; r++) {
    for (let c = 0; c < cols; c++) {
      const x = bounds.minX + c * CELL + JITTER_X;
      const z = bounds.minZ + r * CELL + JITTER_Z;
      const hit = raycast(origin.set(x, TOP, z), down, TOP * 2);
      const i = (r * cols + c) * 3;
      pos[i] = x;
      pos[i + 1] = hit === undefined ? VOID_Y : TOP - hit;
      pos[i + 2] = z;
    }
  }
  const idx: number[] = [];
  for (let r = 0; r < rows - 1; r++) {
    for (let c = 0; c < cols - 1; c++) {
      const a = r * cols + c;
      const b = a + 1;
      const d = a + cols;
      const e = d + 1;
      idx.push(a, d, b, b, d, e);
    }
  }
  const geo = new BufferGeometry();
  geo.setAttribute('position', new BufferAttribute(pos, 3));
  geo.setIndex(idx);
  geo.computeVertexNormals();
  // Steep faces take the wall color so ledges and ridges read against the floor.
  const normals = geo.getAttribute('normal');
  const colors = new Float32Array(cols * rows * 3);
  const tmp = new Color();
  for (let i = 0; i < cols * rows; i++) {
    tmp.copy(floor).lerp(wall, Math.min(1, Math.max(0, (0.9 - normals.getY(i)) * 4)));
    colors[i * 3] = tmp.r;
    colors[i * 3 + 1] = tmp.g;
    colors[i * 3 + 2] = tmp.b;
  }
  geo.setAttribute('color', new BufferAttribute(colors, 3));
  const mesh = new Mesh(geo, new MeshStandardMaterial({ vertexColors: true, roughness: 0.9 }));
  mesh.name = 'level-proxy';
  return mesh;
}
