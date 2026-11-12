// Engine entry point: wires the WASM core, input, fixed-step loop, snapshot interpolation
// and the Three.js scene together. Import dynamically from the game route only.

import { InputController, ReplayRecorder, downloadBytes, encodeReplay } from '../input';
import { parseDevCamera } from '../render/camera';
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
  /** Ticks recorded for the replay so far. */
  replayTicks(): number;
  /** Downloads everything recorded since the world started as a `.lfr` replay file. */
  downloadReplay(): void;
  dispose(): void;
}

export async function startEngine(canvas: HTMLCanvasElement, opts: EngineOptions = {}): Promise<Engine> {
  const wasm = await loadWasm();
  const seed = opts.seed ?? 1n;
  // Minimal dev-URL read for the camera; WP-014 owns the full parser.
  const dev = parseDevCamera(window.location.search);
  const scenario = opts.scenario ?? dev.scenario ?? 'm0_flat';
  dev.scenario = scenario;
  const world = new wasm.glue.LfWorld(seed, scenario);
  const snapshots = new SnapshotBuffer(wasm, world);
  const input = new InputController();
  input.attach(canvas);
  const recorder = new ReplayRecorder();
  // The camera's collision queries go through the core's read-only `raycast` export.
  const rayOrigin = new Float32Array(3);
  const rayDir = new Float32Array(3);
  const view = new SceneView(
    canvas,
    (o, d, max) => {
      rayOrigin[0] = o.x;
      rayOrigin[1] = o.y;
      rayOrigin[2] = o.z;
      rayDir[0] = d.x;
      rayDir[1] = d.y;
      rayDir[2] = d.z;
      return world.raycast(rayOrigin, rayDir, max);
    },
    dev.active ? dev : null
  );

  let fps = 60;
  let sinceStats = 0;
  const loop = new FixedStepLoop({
    tick() {
      input.poll();
      const frame = input.frame();
      recorder.record(frame); // Every tick's input goes into the replay.
      world.tick(frame);
      snapshots.push();
      world.drain_events(); // Nothing consumes events yet; drain so the queue stays empty.
    },
    render(alpha, frameMs) {
      input.updateLook(frameMs);
      view.setCamera(input.cameraYaw, input.cameraPitch);
      view.render(snapshots.interpolate(alpha), frameMs);
      // Movement is camera-relative, so when lock-on turns the camera, input follows it.
      if (view.yawDriven) input.cameraYaw = view.cameraYaw;
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
    replayTicks: () => recorder.ticks,
    downloadReplay() {
      const bytes = encodeReplay(wasm.glue, recorder, seed, scenario);
      downloadBytes(bytes, `latticefall-${seed}-${recorder.ticks}t.lfr`);
    },
    dispose() {
      loop.stop();
      input.detach();
      view.dispose();
      world.free();
    }
  };
}
