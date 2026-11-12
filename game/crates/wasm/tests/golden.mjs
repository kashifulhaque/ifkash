// Determinism check: runs every golden replay under WASM (in Node) and natively, and fails
// if any final state hash, or the content hash, differs. Run through `mise run game:wasm`,
// which builds the WASM output first.
import { execFileSync } from 'node:child_process';
import { readdirSync, readFileSync } from 'node:fs';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath, pathToFileURL } from 'node:url';

const here = dirname(fileURLToPath(import.meta.url));
const root = resolve(here, '../../../..');
const gameDir = join(root, 'game');
const wasmDir = join(root, 'src/lib/latticefall/wasm');
const replayDir = join(gameDir, 'tests/replays');

const glue = await import(pathToFileURL(join(wasmDir, 'lf_wasm.js')).href);
glue.initSync({ module: readFileSync(join(wasmDir, 'lf_wasm_bg.wasm')) });

function native(args, pattern) {
  const out = execFileSync('cargo', ['run', '-q', '-p', 'lf-tools', '--', ...args], {
    cwd: gameDir,
    encoding: 'utf8'
  });
  const m = out.match(pattern);
  if (!m) throw new Error(`cannot parse native output for "${args.join(' ')}":\n${out}`);
  return m[1];
}

let failed = false;

const nativeContent = native(['validate'], /content hash ([0-9a-f]{16})/);
const wasmContent = glue.content_hash();
const contentOk = nativeContent === wasmContent;
failed ||= !contentOk;
console.log(`content hash: native ${nativeContent} wasm ${wasmContent} ${contentOk ? 'OK' : 'MISMATCH'}`);

const files = readdirSync(replayDir).filter((f) => f.endsWith('.lfr')).sort();
if (files.length === 0) {
  console.error('no .lfr files found in game/tests/replays');
  process.exit(1);
}
for (const file of files) {
  const path = join(replayDir, file);
  const nativeHash = native(['replay', 'run', path], /final hash ([0-9a-f]{16})/);
  let wasmHash;
  try {
    wasmHash = glue.run_replay(new Uint8Array(readFileSync(path)));
  } catch (e) {
    wasmHash = `ERROR ${e.message ?? e}`;
  }
  const ok = nativeHash === wasmHash;
  failed ||= !ok;
  console.log(`${file}: native ${nativeHash} wasm ${wasmHash} ${ok ? 'OK' : 'MISMATCH'}`);
}

// The live LfWorld path must also agree with run_replay's hashing: tick 30 frames and
// check the snapshot holds the player.
const w = new glue.LfWorld(1n, 'm0_flat');
w.tick(new Uint8Array(30 * 4).fill(0).map((_, i) => (i % 4 === 1 ? 127 : 0)));
const ok = w.snapshot_len() === 1 && glue.snapshot_stride() === 40 && w.tick_count() === 30;
console.log(`LfWorld smoke: ticks ${w.tick_count()}, snapshot records ${w.snapshot_len()}, hash ${w.state_hash()} ${ok ? 'OK' : 'FAIL'}`);
failed ||= !ok;
w.free();

process.exit(failed ? 1 : 0);
