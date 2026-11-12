// Save round trip under WASM (in Node): save a world, build another from the blob, and
// check that the persistent hash matches and that bad blobs throw. Run after
// `mise run game:wasm`: `node game/crates/wasm/tests/save.mjs`.
import { readFileSync } from 'node:fs';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath, pathToFileURL } from 'node:url';

const here = dirname(fileURLToPath(import.meta.url));
const wasmDir = join(resolve(here, '../../../..'), 'src/lib/latticefall/wasm');
const glue = await import(pathToFileURL(join(wasmDir, 'lf_wasm.js')).href);
glue.initSync({ module: readFileSync(join(wasmDir, 'lf_wasm_bg.wasm')) });

function check(ok, message) {
  if (!ok) {
    console.error(`FAIL: ${message}`);
    process.exit(1);
  }
  console.log(`ok: ${message}`);
}

const world = new glue.LfWorld(9n, 'test_field');
world.tick(new Uint8Array(4 * 30));
const before = world.persistent_hash();
const blob = world.save();
check(
  blob.length > 6 && String.fromCharCode(...blob.slice(0, 4)) === 'LFSV',
  `blob has the LFSV header (${blob.length} bytes)`
);

const loaded = glue.LfWorld.fromSave(blob);
check(loaded.persistent_hash() === before, `persistent hash matches after load (${before})`);
check(loaded.tick_count() === 0, 'a loaded world starts a fresh simulation');
loaded.tick(new Uint8Array(4 * 30));
check(loaded.persistent_hash() === before, 'the persistent hash is stable while the world runs');

for (const [label, bytes] of [
  ['empty', new Uint8Array(0)],
  ['wrong magic', new Uint8Array([1, 2, 3, 4, 5, 6, 7])],
  ['newer version', Uint8Array.from(blob, (b, i) => (i === 4 ? 99 : b))],
  ['truncated', blob.slice(0, 9)]
]) {
  let threw = false;
  try {
    glue.LfWorld.fromSave(bytes);
  } catch {
    threw = true;
  }
  check(threw, `fromSave rejects ${label}`);
}
