// Trims a glTF animation library to the clips Latticefall uses and strips its meshes, so only the
// skeleton and animation tracks are committed.
//
//   bun game/scripts/trim-glb.mjs IN.glb OUT.glb --keep Idle_Loop,Walk_Loop
//
// Run from the repo root so `@gltf-transform/core` resolves from the root devDependencies.
import { NodeIO } from '@gltf-transform/core';

const args = process.argv.slice(2);
const [input, output] = args;
const opt = (name) => {
  const i = args.indexOf(name);
  return i >= 0 ? args[i + 1] : undefined;
};
const keep = new Set((opt('--keep') ?? '').split(',').filter(Boolean));
if (!input || !output || keep.size === 0) {
  console.error('usage: trim-glb.mjs IN.glb OUT.glb --keep A,B');
  process.exit(2);
}

const io = new NodeIO();
const doc = await io.read(input);
const root = doc.getRoot();

for (const anim of root.listAnimations()) if (!keep.has(anim.getName())) anim.dispose();
const missing = [...keep].filter((n) => !root.listAnimations().some((a) => a.getName() === n));
if (missing.length) {
  console.error(`clips not found: ${missing.join(', ')}`);
  process.exit(1);
}
for (const node of root.listNodes()) {
  node.setMesh(null);
  node.setSkin(null);
}
for (const skin of root.listSkins()) skin.dispose();
for (const mesh of root.listMeshes()) mesh.dispose();
await io.write(output, doc);
console.log(`wrote ${output} with ${keep.size} clips`);
