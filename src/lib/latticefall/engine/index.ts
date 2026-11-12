// Engine entry point: wires the WASM core, input, fixed-step loop, snapshot interpolation
// and the Three.js scene together. Import dynamically from the game route only.

import { InputController, ReplayRecorder, downloadBytes, encodeReplay } from '../input';
import { parseDevParams } from '../dev/params';
import { cameraOptions } from '../render/camera';
import type { DevTools } from '../dev';
import { decodeEvents } from './events';
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
  const params = parseDevParams(window.location.search);
  const seed = opts.seed ?? params.seed ?? 1n;
  const scenario = opts.scenario ?? params.scenario ?? 'm0_flat';
  const dev = cameraOptions(params, scenario);
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

  let tools: DevTools | null = null;
  let fps = 60;
  let sinceStats = 0;
  const loop = new FixedStepLoop({
    tick() {
      input.poll();
      const frame = input.frame();
      recorder.record(frame); // Every tick's input goes into the replay.
      if (tools) {
        const t0 = performance.now();
        world.tick(frame);
        tools.recordTick(performance.now() - t0);
      } else world.tick(frame);
      snapshots.push();
      // Events go to presentation only; the renderer reads them and never answers.
      const events = decodeEvents(world.drain_events());
      if (events.length) {
        view.vfx.handleEvents(events);
        tools?.onEvents(events);
      }
    },
    render(alpha, frameMs) {
      input.updateLook(frameMs);
      view.setCamera(input.cameraYaw, input.cameraPitch);
      const drawn = snapshots.interpolate(alpha);
      tools?.beforeRender(drawn);
      view.render(drawn, frameMs, loop.isPaused);
      tools?.afterRender(frameMs, world.tick_count(), () => world.state_hash());
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

  // The dev tools load only with `&dev=1`; otherwise none of their code is fetched or run.
  if (params.dev) {
    const { DevTools: Tools } = await import('../dev');
    const first = snapshots.curr[0];
    tools = new Tools({
      params,
      scenario,
      scene: view.scene,
      renderer: view.renderer,
      raycast: (o, d, max) => world.raycast(new Float32Array([o.x, o.y, o.z]), new Float32Array([d.x, d.y, d.z]), max),
      canvas,
      world,
      start: first ? { x: first.x, y: first.y, z: first.z, yaw: first.yaw } : null,
      // A teleport jumps the snapshot, so push twice: no interpolation across the jump.
      refresh: () => {
        snapshots.push();
        snapshots.push();
      },
      setPaused: (p) => loop.setPaused(p),
      vfx: view.vfx
    });
  }

  loop.start();

  return {
    setPaused: (paused) => (tools ? tools.commands.setFrozen(paused) : loop.setPaused(paused)),
    replayTicks: () => recorder.ticks,
    downloadReplay() {
      const bytes = encodeReplay(wasm.glue, recorder, seed, scenario);
      downloadBytes(bytes, `latticefall-${seed}-${recorder.ticks}t.lfr`);
    },
    dispose() {
      loop.stop();
      tools?.dispose();
      input.detach();
      view.dispose();
      world.free();
    }
  };
}
