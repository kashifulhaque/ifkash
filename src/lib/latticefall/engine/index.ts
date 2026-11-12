// Engine entry point: wires the WASM core, input, fixed-step loop, snapshot interpolation
// and the Three.js scene together. Import dynamically from the game route only.

import { InputController, ReplayRecorder, downloadBytes, encodeReplay } from '../input';
import { parseDevParams } from '../dev/params';
import { AudioEngine } from '../audio/engine';
import { cameraOptions } from '../render/camera';
import type { DevTools } from '../dev';
import { decodeEvents } from './events';
import { loadWasm } from './loader';
import { SaveManager, SaveStore } from './save';
import { FixedStepLoop } from './loop';
import { SceneView } from './scene';
import { FLAG_LOCKED, SnapshotBuffer } from './snapshot';
import { parseHud, type NameLookup } from '../ui/hud/hudState';
import type { HudSink } from '../ui/hud/hudStore';

export interface EngineStats {
  tick: number;
  fps: number;
  hash: string;
  paused: boolean;
}

export interface EngineOptions {
  seed?: bigint;
  scenario?: string;
  /**
   * A blob from a save slot (`SaveManager.load(slot)`): starts from it instead of a fresh
   * world. Pass the slot's `scenario` and `seed` too, for the camera and the replay header.
   */
  save?: Uint8Array;
  /** Autosave when the player rests at a Lantern. On unless set to false. */
  autosave?: boolean;
  /** Called about four times a second with the dev readout values. */
  onStats?: (stats: EngineStats) => void;
  /** Receives the HUD's read-only view of the world each tick (WP-045). */
  hud?: HudSink;
}

export interface Engine {
  setPaused(paused: boolean): void;
  /** Ticks recorded for the replay so far. */
  replayTicks(): number;
  /** The three save slots and the autosave hooks (`onAreaChange` is for the level code). */
  saves: SaveManager;
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
  const world = opts.save ? wasm.glue.LfWorld.fromSave(opts.save) : new wasm.glue.LfWorld(seed, scenario);
  const saves = new SaveManager(new SaveStore(), world, { scenario, seed }, (err) =>
    console.warn('latticefall: autosave failed', err)
  );
  const snapshots = new SnapshotBuffer(wasm, world);
  const input = new InputController();
  input.attach(canvas);
  const recorder = new ReplayRecorder();
  // Audio is optional: a browser without Web Audio still plays the game, silently.
  let audio: AudioEngine | null = null;
  try {
    audio = new AudioEngine();
    audio.unlockOn(canvas);
  } catch (err) {
    console.warn('Latticefall audio unavailable', err);
  }
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
  // Draws the area's baked level when it has a manifest; the flat scenarios keep the stand-in ground.
  void view.loadLevel(scenario);

  let tools: DevTools | null = null;
  let fps = 60;
  let sinceStats = 0;
  // The HUD reads the core through `hud_state` and looks names up lazily, once per number.
  const nameCache = new Map<string, string>();
  const names: NameLookup = {
    item: (n) => cached(`i${n}`, () => world.item_name(n)),
    boss: (i) => cached(`b${i}`, () => world.boss_name(i))
  };
  function cached(key: string, read: () => string): string {
    let v = nameCache.get(key);
    if (v === undefined) {
      v = read();
      nameCache.set(key, v);
    }
    return v;
  }
  const pushHud = () => opts.hud?.state(parseHud(world.hud_state(), names));
  pushHud();
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
      pushHud();
      // Events go to presentation only; the renderer reads them and never answers.
      const words = world.drain_events();
      if (opts.autosave !== false) saves.onEventWords(words);
      if (words.length) opts.hud?.events(words);
      const events = decodeEvents(words);
      if (events.length) {
        audio?.handleEvents(events);
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
      // The first drawn actor is the player; the listener follows it and the camera's yaw.
      if (audio && drawn[0] && !loop.isPaused) {
        audio.updatePlayer({ x: drawn[0].x, y: drawn[0].y, z: drawn[0].z, yaw: view.cameraYaw });
      }
      if (opts.hud) {
        const locked = drawn.find((a) => a.flags & FLAG_LOCKED);
        opts.hud.lock(locked ? view.project(locked.x, locked.y + 1.0, locked.z) : null);
      }
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
    saves,
    replayTicks: () => recorder.ticks,
    downloadReplay() {
      const bytes = encodeReplay(wasm.glue, recorder, seed, scenario);
      downloadBytes(bytes, `latticefall-${seed}-${recorder.ticks}t.lfr`);
    },
    dispose() {
      loop.stop();
      audio?.dispose();
      tools?.dispose();
      input.detach();
      view.dispose();
      world.free();
    }
  };
}
