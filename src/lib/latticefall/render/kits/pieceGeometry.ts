// Geometry for kit pieces that have no model yet. Every piece is non-indexed with
// `position`, `normal`, and a linear `color` attribute, so one vertex-colored flat-shaded
// material draws them all and `BatchedMesh` can hold any mix of them.
//
//   - Greybox pieces (tag `block`, or any piece with colliders and no recognized tag) are
//     drawn from their colliders.
//   - The meadow scatter kit has no models; WP-041 gives each piece a small procedural
//     stand-in chosen by tag: `ash_grass`, `stone`, `dead_tree`.
//   - Anything else becomes a small marker box so a missing model is visible, not silent.

import { BoxGeometry, BufferAttribute, BufferGeometry, CapsuleGeometry, Color, CylinderGeometry, IcosahedronGeometry, Matrix4, Quaternion, Vector3 } from 'three';
import { mergeGeometries } from 'three/examples/jsm/utils/BufferGeometryUtils.js';
import type { ManifestPiece } from './manifest';

function paint(g: BufferGeometry, color: Color | ((x: number, y: number, z: number) => Color)): BufferGeometry {
  const out = g.index ? g.toNonIndexed() : g;
  out.deleteAttribute('uv');
  const pos = out.getAttribute('position');
  const col = new Float32Array(pos.count * 3);
  for (let i = 0; i < pos.count; i++) {
    const c = typeof color === 'function' ? color(pos.getX(i), pos.getY(i), pos.getZ(i)) : color;
    col[i * 3] = c.r;
    col[i * 3 + 1] = c.g;
    col[i * 3 + 2] = c.b;
  }
  out.setAttribute('color', new BufferAttribute(col, 3));
  return out;
}

function hash(seed: number): number {
  let h = Math.imul(seed | 0, 2654435761);
  h = Math.imul(h ^ (h >>> 15), 2246822519);
  return ((h ^ (h >>> 13)) >>> 0) / 4294967296;
}

const GREYBOX = new Color('#aaa69b');

function greybox(piece: ManifestPiece): BufferGeometry {
  const parts: BufferGeometry[] = [];
  for (const c of piece.colliders) {
    if ('Box' in c) {
      const { center: m, half_extents: h } = c.Box;
      const g = new BoxGeometry(h[0] * 2, h[1] * 2, h[2] * 2);
      g.translate(m[0], m[1], m[2]);
      parts.push(paint(g, GREYBOX));
    } else {
      const { center: m, half_height: hh, radius } = c.Capsule;
      const g = new CapsuleGeometry(radius, hh * 2, 2, 6);
      g.translate(m[0], m[1], m[2]);
      parts.push(paint(g, GREYBOX));
    }
  }
  return mergeGeometries(parts, false);
}

/** A tuft of thin blades, dark at the root and light at the tip. Drawn double-sided. */
function grassTuft(variant: number): BufferGeometry {
  const root = new Color(variant ? '#857d49' : '#7f7743');
  const tip = new Color(variant ? '#c4b878' : '#b8ac68');
  const blades: BufferGeometry[] = [];
  const count = variant ? 6 : 5;
  for (let i = 0; i < count; i++) {
    const a = (i / count) * Math.PI * 2 + hash(i + variant * 11) * 0.8;
    const h = 0.32 + hash(i * 3 + variant) * (variant ? 0.3 : 0.22);
    const lean = 0.1 + hash(i * 7 + 1) * 0.16;
    const w = 0.055;
    const g = new BufferGeometry();
    // Two triangles per blade give a slight bend at mid-height.
    const mid = h * 0.55;
    const v = new Float32Array([
      -w, 0, 0, w, 0, 0, -w * 0.6, mid, lean * 0.4,
      w, 0, 0, w * 0.6, mid, lean * 0.4, -w * 0.6, mid, lean * 0.4,
      -w * 0.6, mid, lean * 0.4, w * 0.6, mid, lean * 0.4, 0, h, lean
    ]);
    g.setAttribute('position', new BufferAttribute(v, 3));
    g.computeVertexNormals();
    g.rotateY(a);
    g.translate(Math.cos(a * 1.7) * 0.07, 0, Math.sin(a * 1.7) * 0.07);
    blades.push(paint(g, (_x, y) => root.clone().lerp(tip, Math.min(1, y / h))));
  }
  return mergeGeometries(blades, false);
}

/** A squat, lumpy rock. */
function stone(): BufferGeometry {
  const g = new IcosahedronGeometry(0.4, 0);
  const pos = g.getAttribute('position');
  const v = new Vector3();
  for (let i = 0; i < pos.count; i++) {
    v.fromBufferAttribute(pos, i);
    // Same input gives the same output, so shared icosahedron corners stay joined.
    const k = 0.82 + hash(Math.round(v.x * 100) * 31 + Math.round(v.y * 100) * 17 + Math.round(v.z * 100) * 13) * 0.36;
    pos.setXYZ(i, v.x * k * 1.15, v.y * k * 0.62, v.z * k);
  }
  g.translate(0, 0.17, 0);
  g.computeVertexNormals();
  const base = new Color('#7e7c76');
  const top = new Color('#9a978e');
  return paint(g, (_x, y) => base.clone().lerp(top, Math.min(1, Math.max(0, y / 0.4))));
}

/** A bare trunk with two broken branches, 4 m tall. */
function deadTree(): BufferGeometry {
  const bark = new Color('#4f4741');
  const pale = new Color('#6b625a');
  const parts: BufferGeometry[] = [];
  const trunk = new CylinderGeometry(0.1, 0.3, 4, 5, 1);
  trunk.translate(0, 2, 0);
  parts.push(paint(trunk, (_x, y) => bark.clone().lerp(pale, y / 4)));
  const limb = (len: number, r: number, yaw: number, tilt: number, at: number): void => {
    const g = new CylinderGeometry(r * 0.4, r, len, 4, 1);
    g.translate(0, len / 2, 0);
    const rot = new Quaternion()
      .setFromAxisAngle(new Vector3(0, 1, 0), yaw)
      .multiply(new Quaternion().setFromAxisAngle(new Vector3(0, 0, 1), tilt));
    const m = new Matrix4().compose(new Vector3(0, at, 0), rot, new Vector3(1, 1, 1));
    g.applyMatrix4(m);
    parts.push(paint(g, pale));
  };
  limb(1.5, 0.08, 0.5, -0.9, 2.7);
  limb(1.1, 0.07, 3.4, 0.8, 3.2);
  return mergeGeometries(parts, false);
}

function marker(): BufferGeometry {
  const g = new BoxGeometry(0.5, 0.5, 0.5);
  g.translate(0, 0.25, 0);
  return paint(g, new Color('#b04a8c'));
}

/** Builds the stand-in geometry for a piece by its tags, then its colliders. */
export function placeholderGeometry(id: string, piece: ManifestPiece): BufferGeometry {
  const tags = piece.tags;
  if (tags.includes('ash_grass')) return grassTuft(id.endsWith('_b') ? 1 : 0);
  if (tags.includes('stone')) return stone();
  if (tags.includes('dead_tree')) return deadTree();
  if (piece.colliders.length > 0) return greybox(piece);
  return marker();
}

/** Grass blades are single triangles, so they draw from both sides. */
export function needsDoubleSide(piece: ManifestPiece): boolean {
  return piece.tags.includes('ash_grass');
}

/** Converts a model's mesh geometry to the kit attribute set (position, normal, color). */
export function toKitGeometry(source: BufferGeometry): BufferGeometry {
  const g = source.index ? source.toNonIndexed() : source.clone();
  for (const name of Object.keys(g.attributes)) if (name !== 'position' && name !== 'normal' && name !== 'color') g.deleteAttribute(name);
  if (!g.getAttribute('normal')) g.computeVertexNormals();
  if (!g.getAttribute('color')) return paint(g, new Color('#9a968c'));
  return g;
}
