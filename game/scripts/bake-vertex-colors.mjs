// Bakes each primitive's base color (material color times its base-color texture) into a
// COLOR_0 vertex attribute, then drops the textures and UVs.
//
// The kit renderer (`src/lib/latticefall/render/kits/pieceGeometry.ts`) keeps only
// position, normal, and color, and the art direction allows vertex colors only. The
// Kenney and KayKit packs color their models through a small palette texture, and
// Quaternius packs through flat material colors, so both need this step before they can
// be drawn. Each face's UVs sit inside one flat palette swatch, so sampling the texture
// at every vertex (nearest texel) keeps the colors exact.
//
// Usage: bun game/scripts/bake-vertex-colors.mjs OUT_DIR INPUT.glb|INPUT.gltf...
// Writes OUT_DIR/<input basename>.glb for each input. Never edits the inputs.

import { NodeIO } from '@gltf-transform/core';
import { ALL_EXTENSIONS } from '@gltf-transform/extensions';
import { dedup, prune, weld } from '@gltf-transform/functions';
import { getPixels } from 'ndarray-pixels';
import path from 'node:path';
import fs from 'node:fs';

const [outDir, ...inputs] = process.argv.slice(2);
if (!outDir || inputs.length === 0) {
  console.error('usage: bake-vertex-colors.mjs OUT_DIR INPUT...');
  process.exit(2);
}
fs.mkdirSync(outDir, { recursive: true });

const io = new NodeIO().registerExtensions(ALL_EXTENSIONS);

/** sRGB 0-255 to linear 0-1: glTF stores vertex colors in linear space. */
const toLinear = (c) => {
  const s = c / 255;
  return s <= 0.04045 ? s / 12.92 : Math.pow((s + 0.055) / 1.055, 2.4);
};

const pixelCache = new Map();
async function pixelsOf(texture) {
  if (!pixelCache.has(texture)) {
    pixelCache.set(texture, await getPixels(texture.getImage(), texture.getMimeType()));
  }
  return pixelCache.get(texture);
}

for (const input of inputs) {
  pixelCache.clear();
  const doc = await io.read(input);
  const root = doc.getRoot();

  for (const mesh of root.listMeshes()) {
    for (const prim of mesh.listPrimitives()) {
      const pos = prim.getAttribute('POSITION');
      if (!pos) continue;
      const count = pos.getCount();
      const mat = prim.getMaterial();
      const factor = mat ? mat.getBaseColorFactor() : [1, 1, 1, 1];
      const tex = mat ? mat.getBaseColorTexture() : null;
      const texInfo = mat ? mat.getBaseColorTextureInfo() : null;
      const uv = tex ? prim.getAttribute(`TEXCOORD_${texInfo.getTexCoord()}`) : null;
      const old = prim.getAttribute('COLOR_0');
      const pixels = tex && uv ? await pixelsOf(tex) : null;

      const out = new Float32Array(count * 3);
      const t = [0, 0];
      const c = [1, 1, 1, 1];
      for (let i = 0; i < count; i++) {
        let r = factor[0], g = factor[1], b = factor[2];
        if (pixels) {
          uv.getElement(i, t);
          const [w, h] = [pixels.shape[0], pixels.shape[1]];
          // Wrap (REPEAT is the glTF default) and take the nearest texel.
          const u = ((t[0] % 1) + 1) % 1;
          const v = ((t[1] % 1) + 1) % 1;
          const x = Math.min(w - 1, Math.floor(u * w));
          const y = Math.min(h - 1, Math.floor(v * h));
          r *= toLinear(pixels.get(x, y, 0));
          g *= toLinear(pixels.get(x, y, 1));
          b *= toLinear(pixels.get(x, y, 2));
        }
        if (old) {
          old.getElement(i, c);
          r *= c[0]; g *= c[1]; b *= c[2];
        }
        out.set([r, g, b], i * 3);
      }
      const color = doc.createAccessor().setType('VEC3').setArray(out).setBuffer(pos.getBuffer());
      prim.setAttribute('COLOR_0', color);
      for (const sem of prim.listSemantics()) {
        if (sem.startsWith('TEXCOORD_') || sem === 'TANGENT') prim.setAttribute(sem, null);
      }
    }
  }

  // One plain material per original material, white so the vertex colors show unchanged.
  // Emission stays, so a lit candle or torch flame can still glow.
  for (const mat of root.listMaterials()) {
    mat.setBaseColorTexture(null)
      .setNormalTexture(null)
      .setOcclusionTexture(null)
      .setMetallicRoughnessTexture(null)
      .setEmissiveTexture(null)
      .setBaseColorFactor([1, 1, 1, mat.getBaseColorFactor()[3]])
      .setMetallicFactor(0)
      .setRoughnessFactor(0.95);
  }

  await doc.transform(prune(), dedup(), weld());
  const name = path.basename(input).replace(/\.gltf\.glb$|\.glb$|\.gltf$/, '') + '.glb';
  await io.write(path.join(outDir, name), doc);
}
console.log(`bake-vertex-colors: ${inputs.length} files written to ${outDir}`);
