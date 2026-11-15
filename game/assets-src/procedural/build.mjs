// Builds Cinderheart's procedural props and landmarks as GLB files: the models no CC0 pack
// has (the Lantern checkpoint, Hollowdeep's crystals and glowing fungi, the mine rails and
// cart, the Glasswing mirror, the Spire's woven pieces, and the region landmarks).
//
// Staging only: nothing in the game loads these yet. See docs/game/assets/MODELS.md.
//
// Conventions, the same as the kit packs after `game/scripts/bake-vertex-colors.mjs`:
// - Meters, +Y up, the front faces +Z, and the origin at the base center.
// - Flat shading: every triangle has its own vertices and face normal.
// - Color comes from COLOR_0 (linear) on a white material. Parts that glow use a second
//   material with an emissive color, so the renderer can find and dim them.
// - Parts that move are their own named nodes with the pivot at the node origin (for
//   example `handle` on the lever and `wheel_fl` on the mine cart).
//
// Usage: bun game/assets-src/procedural/build.mjs [OUT_DIR]
// OUT_DIR defaults to this script's folder.

import { Document, NodeIO } from '@gltf-transform/core';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const here = path.dirname(fileURLToPath(import.meta.url));
const outDir = process.argv[2] ?? here;
const io = new NodeIO();

// ---------------------------------------------------------------------------------------
// Colors (sRGB hex, converted to linear for COLOR_0). Region accents follow the table in
// docs/game/07-presentation.md, "Art direction".

const PALETTE = {
  ash: '#8a8580',
  ashDark: '#5f5b57',
  stone: '#9a968c',
  stoneDark: '#6e6a62',
  iron: '#3d3b3a',
  ironLight: '#5a5856',
  wood: '#6b4e33',
  woodDark: '#4a3523',
  amber: '#ffb347', // Lantern amber (the Verge accent, and every Lantern)
  amberDeep: '#e07a1f',
  glass: '#bfe3e6',
  cyan: '#4fe0ff', // Hollowdeep bioluminescent cyan
  cyanDeep: '#1f8fb0',
  violet: '#3a2f4f',
  violetDeep: '#241c33',
  fungusStem: '#cfc6b8',
  reed: '#c9c08f',
  reedDark: '#8f8a5c',
  indigo: '#2b2950', // Spire dusk indigo
  gold: '#e8c25a', // Spire thread gold
  bronze: '#8c6a3f',
  frost: '#dfeff5',
  rail: '#4a4847'
};

const lin = (c) => {
  const s = c / 255;
  return s <= 0.04045 ? s / 12.92 : Math.pow((s + 0.055) / 1.055, 2.4);
};
const rgb = (hex) => {
  const n = parseInt(hex.slice(1), 16);
  return [lin((n >> 16) & 255), lin((n >> 8) & 255), lin(n & 255)];
};

// ---------------------------------------------------------------------------------------
// Geometry. A shape is a flat list of triangles, each three [x, y, z] points.

const sub = (a, b) => [a[0] - b[0], a[1] - b[1], a[2] - b[2]];
const cross = (a, b) => [a[1] * b[2] - a[2] * b[1], a[2] * b[0] - a[0] * b[2], a[0] * b[1] - a[1] * b[0]];
const norm = (v) => {
  const l = Math.hypot(v[0], v[1], v[2]) || 1;
  return [v[0] / l, v[1] / l, v[2] / l];
};

/** A quad as two triangles, corners counter-clockwise seen from outside. */
const quad = (a, b, c, d) => [[a, b, c], [a, c, d]];

/** An axis-aligned box from its size, centered on `c`. */
function box([sx, sy, sz], c = [0, 0, 0]) {
  const [x0, x1] = [c[0] - sx / 2, c[0] + sx / 2];
  const [y0, y1] = [c[1] - sy / 2, c[1] + sy / 2];
  const [z0, z1] = [c[2] - sz / 2, c[2] + sz / 2];
  return [
    ...quad([x0, y0, z1], [x1, y0, z1], [x1, y1, z1], [x0, y1, z1]),
    ...quad([x1, y0, z0], [x0, y0, z0], [x0, y1, z0], [x1, y1, z0]),
    ...quad([x1, y0, z1], [x1, y0, z0], [x1, y1, z0], [x1, y1, z1]),
    ...quad([x0, y0, z0], [x0, y0, z1], [x0, y1, z1], [x0, y1, z0]),
    ...quad([x0, y1, z1], [x1, y1, z1], [x1, y1, z0], [x0, y1, z0]),
    ...quad([x0, y0, z0], [x1, y0, z0], [x1, y0, z1], [x0, y0, z1])
  ];
}

/**
 * A surface of revolution around +Y: `profile` is a list of [radius, y] points from the
 * bottom up. Radius 0 closes a pole. `sides` sets the facet count (low-poly: 5 to 10).
 */
function lathe(profile, sides, { capBottom = true, capTop = true, twist = 0 } = {}) {
  const tris = [];
  const ring = (r, y, k) => {
    const a = (k / sides) * Math.PI * 2 + twist;
    return [Math.cos(a) * r, y, Math.sin(a) * r];
  };
  for (let i = 0; i < profile.length - 1; i++) {
    const [r0, y0] = profile[i];
    const [r1, y1] = profile[i + 1];
    for (let k = 0; k < sides; k++) {
      const a = ring(r0, y0, k), b = ring(r0, y0, k + 1);
      const c = ring(r1, y1, k + 1), d = ring(r1, y1, k);
      if (r0 > 0) tris.push([a, d, b]);
      if (r1 > 0) tris.push([b, d, c]);
    }
  }
  const capAt = (r, y, up) => {
    if (r <= 0) return;
    for (let k = 0; k < sides; k++) {
      const a = ring(r, y, k), b = ring(r, y, k + 1), m = [0, y, 0];
      tris.push(up ? [m, b, a] : [m, a, b]);
    }
  };
  if (capBottom) capAt(profile[0][0], profile[0][1], false);
  if (capTop) capAt(profile.at(-1)[0], profile.at(-1)[1], true);
  return tris;
}

const cylinder = (r, h, sides, y0 = 0, rTop = r) => lathe([[r, y0], [rTop, y0 + h]], sides);

/** A crystal: a prism with pointed ends, `sides` facets, length `h` along +Y. */
const crystal = (r, h, sides = 6) => lathe([[0, 0], [r, h * 0.18], [r * 0.85, h * 0.78], [0, h]], sides);

/** A beam from point a to point b with a square section of side `t`. */
function beam(a, b, t) {
  const d = sub(b, a);
  const len = Math.hypot(...d);
  const shape = box([t, len, t], [0, len / 2, 0]);
  return transform(shape, { from: [0, 1, 0], to: norm(d), at: a });
}

// Transforms applied to the points of a shape.

function rotateAxis(p, axis, angle) {
  const [x, y, z] = axis, c = Math.cos(angle), s = Math.sin(angle), t = 1 - c;
  return [
    (t * x * x + c) * p[0] + (t * x * y - s * z) * p[1] + (t * x * z + s * y) * p[2],
    (t * x * y + s * z) * p[0] + (t * y * y + c) * p[1] + (t * y * z - s * x) * p[2],
    (t * x * z - s * y) * p[0] + (t * y * z + s * x) * p[1] + (t * z * z + c) * p[2]
  ];
}

/**
 * Moves a shape: optional `scale` [x, y, z], then a rotation (`axis` + `angle`, or the
 * rotation taking direction `from` to `to`), then a translation `at`.
 */
function transform(shape, { scale, axis, angle, from, to, at = [0, 0, 0] }) {
  let rot = null;
  if (axis) rot = [norm(axis), angle];
  if (from && to) {
    const c = cross(from, to);
    const dot = from[0] * to[0] + from[1] * to[1] + from[2] * to[2];
    if (Math.hypot(...c) > 1e-6) rot = [norm(c), Math.acos(Math.max(-1, Math.min(1, dot)))];
    else if (dot < 0) rot = [[1, 0, 0], Math.PI];
  }
  return shape.map((tri) =>
    tri.map((p) => {
      let q = scale ? [p[0] * scale[0], p[1] * scale[1], p[2] * scale[2]] : p;
      if (rot) q = rotateAxis(q, rot[0], rot[1]);
      return [q[0] + at[0], q[1] + at[1], q[2] + at[2]];
    })
  );
}

// A small seeded random so rebuilds give identical files.
function rng(seed) {
  let s = seed >>> 0;
  return () => ((s = (s * 1664525 + 1013904223) >>> 0) / 4294967296);
}

// ---------------------------------------------------------------------------------------
// Model assembly. A model is a tree of named parts; each part holds colored shapes.

class Model {
  constructor(name) {
    this.name = name;
    this.root = { name, at: [0, 0, 0], shapes: [], children: [] };
  }
  /** Adds a child node (a pivot for a moving part) and returns it. */
  node(name, at = [0, 0, 0], parent = this.root) {
    const n = { name, at, shapes: [], children: [] };
    parent.children.push(n);
    return n;
  }
  /** Adds triangles in `color` to a node. `glow` puts them in the emissive material. */
  add(shape, color, { node = this.root, glow = false } = {}) {
    node.shapes.push({ shape, color: rgb(PALETTE[color] ?? color), glow });
    return this;
  }
}

async function write(model, glowColor = PALETTE.amber, glowStrength = 1) {
  const doc = new Document();
  const buffer = doc.createBuffer();
  const plain = doc.createMaterial('vertex').setBaseColorFactor([1, 1, 1, 1]).setMetallicFactor(0).setRoughnessFactor(0.95);
  const glow = doc
    .createMaterial('glow')
    .setBaseColorFactor([1, 1, 1, 1])
    .setEmissiveFactor(rgb(glowColor).map((c) => c * glowStrength))
    .setMetallicFactor(0)
    .setRoughnessFactor(0.6);
  const scene = doc.createScene(model.name);
  let triangles = 0;

  const build = (n, parentNode) => {
    const node = doc.createNode(n.name).setTranslation(n.at);
    const prims = [];
    for (const lit of [false, true]) {
      const parts = n.shapes.filter((s) => s.glow === lit);
      if (!parts.length) continue;
      const pos = [], nor = [], col = [];
      for (const { shape, color } of parts) {
        for (const [a, b, c] of shape) {
          const fn = norm(cross(sub(b, a), sub(c, a)));
          for (const p of [a, b, c]) {
            pos.push(p[0] - n.absAt[0], p[1] - n.absAt[1], p[2] - n.absAt[2]);
            nor.push(...fn);
            col.push(...color);
          }
          triangles++;
        }
      }
      const acc = (type, arr) => doc.createAccessor().setType(type).setArray(new Float32Array(arr)).setBuffer(buffer);
      prims.push(
        doc
          .createPrimitive()
          .setAttribute('POSITION', acc('VEC3', pos))
          .setAttribute('NORMAL', acc('VEC3', nor))
          .setAttribute('COLOR_0', acc('VEC3', col))
          .setMaterial(lit ? glow : plain)
      );
    }
    if (prims.length) {
      const mesh = doc.createMesh(n.name);
      prims.forEach((p) => mesh.addPrimitive(p));
      node.setMesh(mesh);
    }
    for (const child of n.children) build(child, node);
    parentNode ? parentNode.addChild(node) : scene.addChild(node);
  };

  // Shapes are authored in model space; store each node's absolute origin so its vertices
  // can be written relative to its pivot.
  const absolute = (n, base) => {
    n.absAt = [base[0] + n.at[0], base[1] + n.at[1], base[2] + n.at[2]];
    n.children.forEach((c) => absolute(c, n.absAt));
  };
  absolute(model.root, [0, 0, 0]);
  build(model.root, null);

  const file = path.join(outDir, `${model.name}.glb`);
  await io.write(file, doc);
  return { name: model.name, triangles };
}

// ---------------------------------------------------------------------------------------
// The models.

const models = [];

// The Lantern: the checkpoint at every rest point (L00-L23). A stone plinth, an iron post
// and arm, and a hanging cage. The `flame` node is the light source: the renderer dims it
// for an unlit Lantern and brightens it when the player lights it.
models.push(() => {
  const m = new Model('lantern');
  m.add(cylinder(0.5, 0.18, 6, 0, 0.46), 'stoneDark');
  m.add(cylinder(0.4, 0.16, 6, 0.18, 0.36), 'stone');
  m.add(box([0.12, 1.55, 0.12], [0, 0.34 + 0.775, 0]), 'iron');
  m.add(box([0.1, 0.08, 0.55], [0, 1.85, 0.2]), 'iron');
  m.add(box([0.04, 0.12, 0.04], [0, 1.77, 0.44]), 'ironLight');
  const cage = m.node('cage', [0, 1.71, 0.44]);
  // The cage: a hexagonal roof cap, a floor plate, and six bars, hanging from the arm.
  m.add(cylinder(0.16, 0.06, 6, 1.64, 0.05), 'iron', { node: cage });
  m.add(cylinder(0.15, 0.04, 6, 1.3, 0.15), 'iron', { node: cage });
  for (let k = 0; k < 6; k++) {
    const a = (k / 6) * Math.PI * 2;
    const x = Math.cos(a) * 0.14, z = 0.44 + Math.sin(a) * 0.14;
    m.add(box([0.025, 0.3, 0.025], [x, 1.49, z]), 'ironLight', { node: cage });
  }
  const flame = m.node('flame', [0, 1.34, 0.44], cage);
  m.add(lathe([[0, 1.34], [0.07, 1.4], [0.05, 1.5], [0, 1.6]], 5), 'amber', { node: flame, glow: true });
  return write(m, PALETTE.amber, 2.5);
});

// The Warden Brazier (Whitewarden Catacombs): an iron bowl on three legs. The `fire` node
// glows once the brazier is lit (flag `brazier.<id>.lit`).
models.push(() => {
  const m = new Model('warden_brazier');
  m.add(cylinder(0.42, 0.12, 6, 0, 0.38), 'stoneDark');
  for (let k = 0; k < 3; k++) {
    const a = (k / 3) * Math.PI * 2;
    m.add(beam([Math.cos(a) * 0.32, 0.12, Math.sin(a) * 0.32], [Math.cos(a) * 0.18, 0.85, Math.sin(a) * 0.18], 0.06), 'iron');
  }
  m.add(lathe([[0.12, 0.8], [0.42, 0.95], [0.48, 1.08]], 8, { capTop: false }), 'iron');
  m.add(lathe([[0.4, 1.0], [0.4, 1.02]], 8, { capBottom: false }), 'ashDark');
  const fire = m.node('fire', [0, 1.02, 0]);
  const r = rng(7);
  for (let i = 0; i < 4; i++) {
    const a = r() * Math.PI * 2, d = r() * 0.15;
    const h = 0.35 + r() * 0.3;
    m.add(transform(crystal(0.12, h, 5), { at: [Math.cos(a) * d, 1.0, Math.sin(a) * d] }), i % 2 ? 'amber' : 'amberDeep', { node: fire, glow: true });
  }
  return write(m, PALETTE.amber, 2);
});

// The Glasswing mirror (Glasswing Grotto, D5): a crystal panel on a yoke. The yoke turns
// about Y (node `yoke`) and the panel tilts about X (node `mirror`) to aim a light beam.
models.push(() => {
  const m = new Model('glass_mirror');
  m.add(cylinder(0.5, 0.25, 8, 0, 0.42), 'violet');
  const yoke = m.node('yoke', [0, 0.25, 0]);
  m.add(cylinder(0.2, 0.2, 6, 0.25), 'violetDeep', { node: yoke });
  m.add(box([0.1, 1.1, 0.14], [-0.62, 0.45 + 0.55, 0]), 'violetDeep', { node: yoke });
  m.add(box([0.1, 1.1, 0.14], [0.62, 0.45 + 0.55, 0]), 'violetDeep', { node: yoke });
  m.add(box([1.34, 0.1, 0.14], [0, 0.45, 0]), 'violetDeep', { node: yoke });
  const mirror = m.node('mirror', [0, 1.3, 0], yoke);
  m.add(transform(lathe([[0.55, -0.03], [0.55, 0.03]], 6), { axis: [1, 0, 0], angle: Math.PI / 2, at: [0, 1.3, 0] }), 'glass', { node: mirror, glow: true });
  m.add(box([1.16, 0.08, 0.08], [0, 1.3, 0]), 'violetDeep', { node: mirror });
  return write(m, PALETTE.cyan, 0.6);
});

// Hollowdeep crystals, three sizes. Shard Golems and the grotto walls use the same look.
for (const [name, count, scale, seed] of [
  ['crystal_cluster_small', 4, 0.5, 11],
  ['crystal_cluster_medium', 6, 1, 23],
  ['crystal_cluster_large', 9, 2.2, 37]
]) {
  models.push(() => {
    const m = new Model(name);
    const r = rng(seed);
    m.add(transform(lathe([[0.55, 0], [0.45, 0.2], [0, 0.32]], 6), { scale: [scale, scale, scale] }), 'violetDeep');
    for (let i = 0; i < count; i++) {
      const a = r() * Math.PI * 2, d = r() * 0.35 * scale;
      const h = (0.5 + r() * 0.9) * scale, rad = (0.08 + r() * 0.08) * scale;
      const lean = [Math.cos(a) * (0.3 + r() * 0.5), 1, Math.sin(a) * (0.3 + r() * 0.5)];
      m.add(transform(crystal(rad, h, 6), { from: [0, 1, 0], to: norm(lean), at: [Math.cos(a) * d, 0.05 * scale, Math.sin(a) * d] }), i % 3 ? 'cyan' : 'cyanDeep', { glow: i % 3 !== 0 });
    }
    return write(m, PALETTE.cyan, 0.9);
  });
}

// A glowing fungus, small (Fungal Shore dressing). The cap's underside glows.
function fungus(m, at, h, capR, node = m.root) {
  m.add(transform(lathe([[capR * 0.22, 0], [capR * 0.16, h * 0.6], [capR * 0.2, h]], 6), { at }), 'fungusStem', { node });
  m.add(transform(lathe([[capR * 0.2, h - capR * 0.05], [capR, h - capR * 0.1]], 8, { capBottom: false, capTop: false }), { at }), 'cyan', { node, glow: true });
  m.add(transform(lathe([[capR, h - capR * 0.1], [capR * 0.8, h + capR * 0.2], [capR * 0.35, h + capR * 0.4], [0, h + capR * 0.45]], 8, { capBottom: false }), { at }), 'violet', { node });
}
models.push(() => {
  const m = new Model('glow_fungus_cluster');
  const r = rng(5);
  for (let i = 0; i < 5; i++) {
    const a = r() * Math.PI * 2, d = i ? 0.25 + r() * 0.35 : 0;
    fungus(m, [Math.cos(a) * d, 0, Math.sin(a) * d], 0.25 + r() * 0.5, 0.12 + r() * 0.2);
  }
  return write(m, PALETTE.cyan, 1.2);
});

// The Lamp Tree: Hollowdeep's landmark, a giant glowing fungus about 18 m tall, with
// smaller caps up the trunk and glowing pods hanging from the main cap (node `pods`).
models.push(() => {
  const m = new Model('lamp_tree');
  m.add(lathe([[2.4, 0], [1.6, 1.2], [1.1, 6], [0.9, 12], [1.1, 16]], 8), 'fungusStem');
  // Roots.
  for (let k = 0; k < 6; k++) {
    const a = (k / 6) * Math.PI * 2 + 0.3;
    m.add(beam([Math.cos(a) * 1.2, 1.4, Math.sin(a) * 1.2], [Math.cos(a) * 3.6, 0, Math.sin(a) * 3.6], 0.6), 'stoneDark');
  }
  fungus(m, [0, 0, 0], 16, 7);
  fungus(m, [1.1, 5, 0.4], 2.5, 1.6);
  fungus(m, [-0.9, 9, -0.6], 2.2, 1.4);
  const pods = m.node('pods', [0, 15.3, 0]);
  const r = rng(91);
  for (let i = 0; i < 14; i++) {
    const a = r() * Math.PI * 2, d = 2 + r() * 4;
    const x = Math.cos(a) * d, z = Math.sin(a) * d, len = 0.8 + r() * 2.2;
    m.add(box([0.05, len, 0.05], [x, 15.3 - len / 2, z]), 'fungusStem', { node: pods });
    m.add(transform(crystal(0.22, 0.6, 5), { at: [x, 15.3 - len - 0.45, z] }), 'cyan', { node: pods, glow: true });
  }
  return write(m, PALETTE.cyan, 1.4);
});

// Mine rails (Cinderdeep Mine, D4): one 2 m straight segment on the kit grid, along +Z.
models.push(() => {
  const m = new Model('mine_rail_straight');
  for (const z of [-0.75, 0, 0.75]) m.add(box([1.3, 0.08, 0.2], [0, 0.04, z]), 'woodDark');
  for (const x of [-0.45, 0.45]) {
    m.add(box([0.06, 0.06, 2], [x, 0.11, 0]), 'rail');
    m.add(box([0.12, 0.02, 2], [x, 0.09, 0]), 'rail');
  }
  return write(m);
});

// The mine cart: a wooden tub on an iron frame. Each wheel is its own node so it can spin
// (`wheel_fl`, `wheel_fr`, `wheel_bl`, `wheel_br`), and the body is node `body`, so a ride
// can tilt it on slopes. The gauge matches `mine_rail_straight`.
models.push(() => {
  const m = new Model('mine_cart');
  const body = m.node('body', [0, 0.3, 0]);
  m.add(box([1.1, 0.08, 1.5], [0, 0.34, 0]), 'iron', { node: body });
  m.add(box([1.2, 0.6, 0.08], [0, 0.68, 0.76]), 'wood', { node: body });
  m.add(box([1.2, 0.6, 0.08], [0, 0.68, -0.76]), 'wood', { node: body });
  m.add(box([0.08, 0.6, 1.6], [0.6, 0.68, 0]), 'wood', { node: body });
  m.add(box([0.08, 0.6, 1.6], [-0.6, 0.68, 0]), 'wood', { node: body });
  m.add(box([1.1, 0.06, 1.44], [0, 0.41, 0]), 'woodDark', { node: body });
  for (const y of [0.45, 0.92]) {
    m.add(box([1.24, 0.05, 1.64], [0, y, 0]).filter((_, i) => i < 8), 'iron', { node: body });
  }
  for (const [nm, x, z] of [['wheel_fl', 0.45, 0.5], ['wheel_fr', -0.45, 0.5], ['wheel_bl', 0.45, -0.5], ['wheel_br', -0.45, -0.5]]) {
    const w = m.node(nm, [x, 0.2, z]);
    m.add(transform(cylinder(0.2, 0.08, 8, -0.04), { axis: [0, 0, 1], angle: Math.PI / 2, at: [x, 0.2, z] }), 'iron', { node: w });
  }
  return write(m);
});

// A lever (sluice, gatehouse, and lift levers): a stone block with an iron handle. The
// `handle` node's pivot is the hinge; rotate it about X from +0.6 rad (up) to -0.6 (down).
models.push(() => {
  const m = new Model('lever');
  m.add(box([0.6, 0.5, 0.45], [0, 0.25, 0]), 'stoneDark');
  m.add(box([0.5, 0.08, 0.35], [0, 0.54, 0]), 'stone');
  const h = m.node('handle', [0, 0.6, 0]);
  m.add(box([0.36, 0.12, 0.12], [0, 0.6, 0]), 'iron', { node: h });
  m.add(transform(box([0.07, 0.9, 0.07], [0, 0.45, 0]), { axis: [1, 0, 0], angle: 0.6, at: [0, 0.6, 0] }), 'ironLight', { node: h });
  m.add(transform(box([0.14, 0.14, 0.14], [0, 0.92, 0]), { axis: [1, 0, 0], angle: 0.6, at: [0, 0.6, 0] }), 'wood', { node: h });
  return write(m);
});

// A wooden ladder, 3 m tall and 0.6 m wide, against a wall behind it (-Z). The core's
// ladder entity sets the climb height; the renderer stacks or scales this piece.
models.push(() => {
  const m = new Model('ladder_3m');
  for (const x of [-0.27, 0.27]) m.add(box([0.07, 3, 0.07], [x, 1.5, 0]), 'wood');
  for (let y = 0.3; y < 3; y += 0.35) m.add(box([0.54, 0.05, 0.05], [0, y, 0]), 'woodDark');
  return write(m);
});

// A reed clump (Mirrormere's Reed Marsh, where Reed Stalkers hide): thin leaning blades
// with seed heads, about 1.8 m tall. Scatter tag `reeds`.
models.push(() => {
  const m = new Model('reed_clump');
  const r = rng(3);
  for (let i = 0; i < 16; i++) {
    const a = r() * Math.PI * 2, d = r() * 0.45;
    const h = 1.1 + r() * 0.8;
    const base = [Math.cos(a) * d, 0, Math.sin(a) * d];
    const tip = [base[0] + (r() - 0.5) * 0.5, h, base[2] + (r() - 0.5) * 0.5];
    m.add(beam(base, tip, 0.035), i % 3 ? 'reed' : 'reedDark');
    if (i % 4 === 0) m.add(transform(crystal(0.04, 0.22, 4), { at: tip }), 'woodDark');
  }
  return write(m);
});

// The Lattice Spire's woven pieces: a thread pillar 6 m tall, and a 4 m ramp segment for
// the outer spiral. Gold threads glow against the dusk-indigo frame.
models.push(() => {
  const m = new Model('spire_thread_pillar');
  m.add(cylinder(0.7, 0.4, 6, 0, 0.6), 'indigo');
  m.add(cylinder(0.6, 0.4, 6, 5.6, 0.7), 'indigo');
  const turns = 6;
  for (let k = 0; k < turns; k++) {
    for (const dir of [1, -1]) {
      let prev = null;
      for (let s = 0; s <= 10; s++) {
        const y = 0.4 + (s / 10) * 5.2;
        const a = ((k / turns) + dir * (s / 10) * 0.5) * Math.PI * 2;
        const p = [Math.cos(a) * 0.45, y, Math.sin(a) * 0.45];
        if (prev) m.add(beam(prev, p, 0.05), 'gold', { glow: true });
        prev = p;
      }
    }
  }
  m.add(cylinder(0.2, 5.2, 6, 0.4), 'indigo');
  return write(m, PALETTE.gold, 1.6);
});
models.push(() => {
  const m = new Model('spire_ramp_segment');
  // A deck 4 m long (+Z) and 2.5 m wide that rises 1 m, made of woven slats.
  const rise = 1, len = 4, wide = 2.5;
  m.add(transform(box([wide, 0.2, len], [0, 0, 0]), { axis: [1, 0, 0], angle: -Math.atan2(rise, len), at: [0, rise / 2 - 0.1, 0] }), 'indigo');
  for (let i = -3; i <= 3; i++) {
    const z = (i / 3) * (len / 2 - 0.1);
    const y = rise / 2 + (z / len) * rise + 0.02;
    m.add(box([wide * 0.96, 0.03, 0.05], [0, y, z]), 'gold', { glow: true });
  }
  // The frayed outer edge: thread rails on posts.
  for (let i = -2; i <= 2; i++) {
    const z = i * 0.9, y = rise / 2 + (z / len) * rise;
    m.add(box([0.06, 0.9, 0.06], [wide / 2 - 0.05, y + 0.45, z]), 'indigo');
  }
  m.add(beam([wide / 2 - 0.05, 0.9 - 0.2 + 0.1, -1.8], [wide / 2 - 0.05, 0.9 + 0.8 + 0.1, 1.8], 0.04), 'gold', { glow: true });
  return write(m, PALETTE.gold, 1.6);
});

// The Great Bell (Rimeshelf's landmark, frozen in the cliff): a bronze bell 6 m tall with a
// frost crust and icicles. Placed tilted into the cliff by the level, not by the model.
models.push(() => {
  const m = new Model('great_bell');
  m.add(lathe([[3.0, 0], [3.1, 0.35], [2.6, 0.8], [2.1, 2.2], [1.9, 4.2], [1.5, 5.3], [0.9, 5.8], [0, 5.9]], 10, { capBottom: false }), 'bronze');
  m.add(lathe([[2.75, 0.3], [2.0, 2.0], [1.7, 4.8], [0, 5.5]], 10, { capBottom: false }).map(([a, b, c]) => [a, c, b]), 'iron');
  m.add(box([0.6, 1.0, 2.0], [0, 6.3, 0]), 'bronze');
  m.add(lathe([[3.12, 0.3], [2.7, 0.85], [2.15, 2.0]], 10, { capBottom: false, capTop: false, twist: 0.1 }), 'frost');
  const r = rng(17);
  for (let i = 0; i < 12; i++) {
    const a = r() * Math.PI * 2;
    m.add(transform(lathe([[0.12, 0], [0, -0.5 - r() * 0.9]], 4), { at: [Math.cos(a) * 3.0, 0.05, Math.sin(a) * 3.0] }), 'frost');
  }
  return write(m);
});

// One bay of the Verge's broken aqueduct (the region's landmark): two piers 2 m wide, an
// arch spanning 6 m, and the water channel on top, 10 m tall. Bays repeat every 8 m along
// +X. `aqueduct_bay_broken` is the same bay with the channel snapped off mid-span.
for (const broken of [false, true]) {
  models.push(() => {
    const m = new Model(broken ? 'aqueduct_bay_broken' : 'aqueduct_bay');
    for (const x of [-4, 4]) {
      m.add(box([2, 7, 2], [x, 3.5, 0]), 'ash');
      m.add(box([2.3, 0.4, 2.3], [x, 0.2, 0]), 'ashDark');
    }
    const segs = 9;
    for (let i = 0; i < segs; i++) {
      if (broken && i >= 5) break;
      const a0 = Math.PI - (i / segs) * Math.PI, a1 = Math.PI - ((i + 1) / segs) * Math.PI;
      const p0 = [Math.cos(a0) * 3, 7 + Math.sin(a0) * 2.2, 0], p1 = [Math.cos(a1) * 3, 7 + Math.sin(a1) * 2.2, 0];
      const voussoir = box([Math.hypot(p1[0] - p0[0], p1[1] - p0[1]) + 0.1, 0.8, 2]);
      const tilt = Math.atan2(p1[1] - p0[1], p1[0] - p0[0]);
      m.add(transform(voussoir, { axis: [0, 0, 1], angle: tilt, at: [(p0[0] + p1[0]) / 2, (p0[1] + p1[1]) / 2, 0] }), i % 2 ? 'stone' : 'ash');
    }
    // Spandrel fill over the piers, then the channel.
    for (const x of [-4.25, 4.25]) m.add(box([1.5, 2.6, 2], [x, 8.3, 0]), 'ash');
    const span = broken ? 4.6 : 8;
    const x0 = -4;
    m.add(box([span, 0.5, 2.2], [x0 + span / 2, 9.85, 0]), 'stoneDark');
    m.add(box([span, 0.5, 0.25], [x0 + span / 2, 10.35, 0.98]), 'stone');
    m.add(box([span, 0.5, 0.25], [x0 + span / 2, 10.35, -0.98]), 'stone');
    if (broken) {
      // Fallen blocks at the foot of the break.
      const r = rng(29);
      for (let i = 0; i < 5; i++) {
        const s = 0.5 + r() * 0.7;
        m.add(transform(box([s, s * 0.8, s]), { axis: [r(), 1, r()], angle: r() * 1.2, at: [1 + r() * 2.5, s * 0.35, (r() - 0.5) * 3] }), 'ashDark');
      }
    }
    return write(m);
  });
}

const results = [];
for (const make of models) results.push(await make());
for (const r of results) console.log(`${r.name.padEnd(26)} ${String(r.triangles).padStart(6)} triangles`);
console.log(`procedural: ${results.length} models written to ${outDir}`);
