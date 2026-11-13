// Engine entry point: wires the WASM core, input, fixed-step loop, snapshot interpolation
// and the Three.js scene together. Import dynamically from the game route only.

import { InputController, ReplayRecorder, downloadBytes, encodeReplay } from '../input';
import { parseDevParams } from '../dev/params';
import { AudioEngine } from '../audio/engine';
import { cameraOptions } from '../render/camera';
import { BriarFx } from '../render/bosses/b02';
import { digestMoveResolver } from '../render/actors/moveTable';
import { parseSection, sectionCam } from '../render/equipment/layout';
import { equippedFromMenuWords, type Equipped } from '../render/equipment/playerGear';
import type { DevTools } from '../dev';
import { decodeEvents } from './events';
import { loadWasm } from './loader';
import { SaveManager, SaveStore, restedIn } from './save';
import { FixedStepLoop } from './loop';
import { SceneView } from './scene';
import { FLAG_LOCKED, SnapshotBuffer } from './snapshot';
import { parseHud, type NameLookup } from '../ui/hud/hudState';
import { messageIndexesFromWords, type HudSink } from '../ui/hud/hudStore';
import { COMMAND, createMenuHost, type MenuHost, type MenuSink } from '../ui/menus/menuHost';
import { createDialogueHost, type DialogueHost, type DialogueSink } from '../ui/dialogue/dialogueHost';
import { dialogueEventsFromWords } from '../ui/dialogue/dialogueState';
import { createMapHost, type MapHost } from '../ui/map/mapState';

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
  /** Receives the menu and map presses and the Lantern rest, to open the menus (WP-046). */
  menu?: MenuSink;
  /** Receives the conversation events of the hub's NPCs and stations (WP-068). */
  dialogue?: DialogueSink;
  /**
   * Travel to a Lantern in another area (WP-069) replaces the world's level, so the engine stops
   * and hands over a save of the new world: start a fresh engine from it (`save`, `scenario`).
   */
  onTravel?: (save: Uint8Array, scenario: string) => void;
}

export interface Engine {
  setPaused(paused: boolean): void;
  /** Ticks recorded for the replay so far. */
  replayTicks(): number;
  /** The three save slots and the autosave hooks (`onAreaChange` is for the level code). */
  saves: SaveManager;
  /** What the menus read and the commands they send (WP-046). */
  menus: MenuHost;
  /** What the dialogue reads and the commands it sends (WP-068). */
  dialogue: DialogueHost;
  /** What the map reads and the commands it sends (WP-069). */
  map: MapHost;
  /** Rests at the Lantern in reach, as the Interact button does (the Lantern menu opens). */
  rest(): void;
  /** The input controller, for the controls menu (bindings and options). */
  input: InputController;
  /** Downloads everything recorded since the world started as a `.lfr` replay file. */
  downloadReplay(): void;
  dispose(): void;
}

export async function startEngine(canvas: HTMLCanvasElement, opts: EngineOptions = {}): Promise<Engine> {
  const wasm = await loadWasm();
  const params = parseDevParams(window.location.search);
  const seed = opts.seed ?? params.seed ?? 1n;
  const scenario = opts.scenario ?? params.scenario ?? 'm0_flat';
  // The dev armory (`&scene=armory`) starts its camera at a section unless `cam` says otherwise.
  if (params.scene === 'armory' && !params.cam) {
    const c = sectionCam(parseSection(params.armorySection));
    params.cam = { kind: 'fixed', pos: c.pos, yaw: c.yaw, pitch: c.pitch };
  }
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
    // Waking Hollow has its own sound bed (WP-043); other areas keep the default voice.
    if (scenario === 'waking_hollow') audio.setRegion('waking-hollow');
    // The Verge (WP-044) is open and windy.
    if (scenario === 'verge') audio.setRegion('verge');
    // Hollowhearth (WP-068) crackles by its hearth.
    if (scenario === 'hollowhearth') audio.setRegion('hollowhearth');
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
  // B02's patches, charge line and leap marker (WP-070); fed by the events and the drawn actors.
  const briar = new BriarFx(view.scene, digestMoveResolver(), view.vfx.shake);
  view.frameHooks.push((dt) => briar.update(dt));
  // Draws the area's baked level when it has a manifest; the flat scenarios keep the stand-in ground.
  void view.loadLevel(scenario);
  // The armory loads only for `&dev=1&scene=armory`; its code is a separate chunk.
  let armory: { dispose(): void } | null = null;
  if (params.scene === 'armory') {
    void import('../render/equipment/armory').then(({ Armory }) => {
      if (disposed) return;
      const a = new Armory();
      view.scene.add(a.group);
      view.frameHooks.push((dt, camera, px) => a.update(dt, camera, px));
      armory = a;
    });
  }
  let disposed = false;
  // What the player wears (WP-058): read from the menu state a few times a second, drawn by
  // the actor renderer. The renderer never writes the equipment back.
  let lastEquipped: Equipped | null = null;
  const syncEquipment = () => {
    const eq = equippedFromMenuWords(world.menu_state());
    if (lastEquipped && eq.right === lastEquipped.right && eq.left === lastEquipped.left && eq.head === lastEquipped.head && eq.body === lastEquipped.body) return;
    lastEquipped = eq;
    view.setEquipment(eq);
  };
  syncEquipment();

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
  // The Lantern prompt (HUD word 17) and the interact edge turn a press of Interact into a
  // `Rest` command; the menus' commands are recorded with their tick for replays.
  let lastPrompt = 0;
  let prevInteract = false;
  let paused = false;
  let travelled = false;
  const commandLog: number[] = [];
  function sendCommand(kind: number, a = 0, b = 0): boolean {
    if (!world.queue_command(kind, a, b)) return false;
    commandLog.push(world.tick_count(), kind, a, b);
    return true;
  }
  function runTick(frame: Uint8Array): void {
    recorder.record(frame); // Every tick's input goes into the replay.
    if (tools) {
      const t0 = performance.now();
      world.tick(frame);
      tools.recordTick(performance.now() - t0);
    } else world.tick(frame);
    snapshots.push();
    if (world.tick_count() % 6 === 0) syncEquipment();
    if (view.props.active) view.props.setState(world.mechanics_state());
    const hudState = parseHud(world.hud_state(), names);
    // The core's interact prompt also names doors, levers, messages, and pickups (code 1 is the
    // Lantern, the same as the HUD's own word 17).
    hudState.prompt = world.interact_prompt();
    lastPrompt = hudState.prompt;
    opts.hud?.state(hudState);
    // Events go to presentation only; the renderer reads them and never answers.
    const words = world.drain_events();
    if (opts.autosave !== false) saves.onEventWords(words);
    if (words.length) {
      view.voss.events(words);
      opts.hud?.events(words);
      // Authored ground messages (the cave's tutorial lines): the HUD shows the text.
      for (const i of messageIndexesFromWords(words)) opts.hud?.message(world.message_text(i));
      if (restedIn(words)) opts.menu?.rested();
      // Conversations with the hub's NPCs and stations (WP-068).
      if (opts.dialogue) for (const e of dialogueEventsFromWords(words)) opts.dialogue.event(e);
    }
    // A trip to another area replaced the world: the level this view drew is gone (WP-069).
    if (!travelled && world.scenario_name() !== scenario) {
      travelled = true;
      loop.stop();
      opts.onTravel?.(world.save(), world.scenario_name());
      return;
    }
    const events = decodeEvents(words);
    if (events.length) {
      audio?.handleEvents(events);
      view.vfx.handleEvents(events);
      briar.handleEvents(events);
      tools?.onEvents(events);
    }
  }
  const menus = createMenuHost(world, {
    command(kind, a, b) {
      if (!sendCommand(kind, a, b)) return;
      // A paused world takes no ticks, so a menu command spends one with no input.
      if (paused) runTick(new Uint8Array(4));
    }
  });
  const dialogue = createDialogueHost(world, {
    command(kind, a, b) {
      if (!sendCommand(kind, a, b)) return;
      if (paused) runTick(new Uint8Array(4));
    }
  });
  const map = createMapHost(world, {
    command(kind, a, b) {
      if (!sendCommand(kind, a, b)) return;
      if (paused) runTick(new Uint8Array(4));
    }
  });
  const loop = new FixedStepLoop({
    tick() {
      input.poll();
      const frame = input.frame();
      const interact = (frame[2] & 0x80) !== 0;
      if (interact && !prevInteract && lastPrompt === 1) sendCommand(COMMAND.Rest);
      prevInteract = interact;
      runTick(frame);
    },
    render(alpha, frameMs) {
      for (const action of input.drainUi()) opts.menu?.ui(action);
      input.updateLook(frameMs);
      view.setCamera(input.cameraYaw, input.cameraPitch);
      const drawn = snapshots.interpolate(alpha);
      briar.setActors(drawn);
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
    setPaused(p) {
      paused = p;
      if (tools) tools.commands.setFrozen(p);
      else loop.setPaused(p);
      input.setSuspended(p && opts.menu !== undefined);
    },
    saves,
    menus,
    dialogue,
    map,
    rest() {
      if (sendCommand(COMMAND.Rest) && paused) runTick(new Uint8Array(4));
    },
    input,
    replayTicks: () => recorder.ticks,
    downloadReplay() {
      const bytes = encodeReplay(wasm.glue, recorder, seed, scenario, commandLog);
      downloadBytes(bytes, `latticefall-${seed}-${recorder.ticks}t.lfr`);
    },
    dispose() {
      disposed = true;
      armory?.dispose();
      loop.stop();
      audio?.dispose();
      tools?.dispose();
      input.detach();
      briar.dispose();
      view.dispose();
      world.free();
    }
  };
}
