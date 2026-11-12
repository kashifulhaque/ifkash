// Engine entry point: wires the WASM core, input, fixed-step loop, snapshot interpolation
// and the Three.js scene together. Import dynamically from the game route only.

import { KeyboardInput } from './input';
import { loadWasm } from './loader';
import { FixedStepLoop } from './loop';
import { SceneView } from './scene';
import { SnapshotBuffer } from './snapshot';

export interface EngineStats {
  tick: number;
  fps: number;
  hash: string;
  paused: boolean;
}

export interface EngineOptions {
  seed?: bigint;
  scenario?: string;
  /** Called about four times a second with the dev readout values. */
  onStats?: (stats: EngineStats) => void;
}

export interface Engine {
  setPaused(paused: boolean): void;
  dispose(): void;
}

export async function startEngine(canvas: HTMLCanvasElement, opts: EngineOptions = {}): Promise<Engine> {
  const wasm = await loadWasm();
  const world = new wasm.glue.LfWorld(opts.seed ?? 1n, opts.scenario ?? 'm0_flat');
  const snapshots = new SnapshotBuffer(wasm, world);
  const input = new KeyboardInput();
  input.attach();
  const view = new SceneView(canvas);

  let fps = 60;
  let sinceStats = 0;
  const loop = new FixedStepLoop({
    tick() {
      world.tick(input.frame());
      snapshots.push();
      world.drain_events(); // Nothing consumes events yet; drain so the queue stays empty.
    },
    render(alpha, frameMs) {
      view.render(snapshots.interpolate(alpha), frameMs);
      if (frameMs > 0) fps += (1000 / frameMs - fps) * 0.1;
      sinceStats += frameMs;
      if (sinceStats >= 250 && opts.onStats) {
        sinceStats = 0;
        opts.onStats({
          tick: world.tick_count(),
          fps: Math.round(fps),
          hash: world.state_hash(),
          paused: loop.isPaused
        });
      }
    }
  });
  loop.start();

  return {
    setPaused: (paused) => loop.setPaused(paused),
    dispose() {
      loop.stop();
      input.detach();
      view.dispose();
      world.free();
    }
  };
}
