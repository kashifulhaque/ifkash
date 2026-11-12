// The developer tools, started by the engine only when `&dev=1` is in the URL. The engine
// imports this module dynamically, so without `dev` none of it is even downloaded, and no
// overlay, panel or timer exists. See `params.ts` for the URL parameters.

import type { Scene, WebGLRenderer } from 'three';
import type { RaycastFn } from '../render/camera';
import type { DrawActor } from '../engine/snapshot';
import { DevCommands, type DevWorld } from './commands';
import { DebugDraw } from './debugDraw';
import { DevPanel } from './panel';
import type { DevParams } from './params';
import { spawnIds } from './spawns';

export type { DevParams, OverlayName, CamMode, FixedCam } from './params';

export interface DevToolsDeps {
  params: DevParams;
  scenario: string;
  scene: Scene;
  renderer: WebGLRenderer;
  raycast: RaycastFn;
  canvas: HTMLCanvasElement;
  world: DevWorld;
  /** Reads the player's starting position from the first snapshot, for `spawn=spawn`. */
  start: { x: number; y: number; z: number; yaw: number } | null;
  refresh(): void;
  setPaused(paused: boolean): void;
}

declare global {
  interface Window {
    /** Console access to the dev commands, set only when `&dev=1`. */
    __lfDev?: {
      commands: DevCommands;
      draw: DebugDraw;
      params: DevParams;
    };
  }
}

const PANEL_MS = 250;

export class DevTools {
  readonly commands: DevCommands;
  readonly draw: DebugDraw;
  private panel: DevPanel;
  private lastPanel = 0;
  private tickSum = 0;
  private tickMax = 0;
  private tickN = 0;
  private actors: DrawActor[] = [];
  private tickCount = 0;
  private frameMs = 0;

  constructor(private deps: DevToolsDeps) {
    const { params } = deps;
    this.commands = new DevCommands({
      world: deps.world,
      scenario: deps.scenario,
      start: deps.start,
      refresh: deps.refresh,
      setPaused: deps.setPaused
    });
    this.draw = new DebugDraw(deps.scene, deps.raycast, deps.scenario);
    for (const name of params.overlays) this.draw.setEnabled(name, true);

    const warnings = [...params.warnings];
    if (params.spawn !== null && !this.commands.available) warnings.push(`spawn=${params.spawn}: needs the dev WASM build`);
    if (params.spawn !== null && this.commands.available && !this.commands.teleport(params.spawn)) {
      warnings.push(`spawn=${params.spawn}: no such spawn point in ${deps.scenario} (${spawnIds(deps.scenario).join(', ')})`);
    }
    if (params.freeze) this.commands.setFrozen(true);
    for (const w of warnings) console.warn(`[latticefall dev] ${w}`);

    this.panel = new DevPanel({
      parent: deps.canvas.parentElement ?? document.body,
      scenario: deps.scenario,
      spawnIds: spawnIds(deps.scenario),
      commands: this.commands,
      draw: this.draw,
      warnings,
      onOverlayChange: () => this.draw.update(this.actors)
    });
    window.__lfDev = { commands: this.commands, draw: this.draw, params };
  }

  /** Records how long one core tick took. */
  recordTick(ms: number): void {
    this.tickSum += ms;
    this.tickMax = Math.max(this.tickMax, ms);
    this.tickN++;
  }

  /** Call just before the scene renders. */
  beforeRender(actors: DrawActor[]): void {
    this.actors = actors;
    this.draw.update(actors);
  }

  /** Call just after the scene renders; refreshes the panel about four times a second. */
  afterRender(frameMs: number, tick: number, hash: () => string): void {
    this.frameMs += (frameMs - this.frameMs) * 0.1;
    this.tickCount = tick;
    const now = performance.now();
    if (now - this.lastPanel < PANEL_MS) return;
    this.lastPanel = now;
    const info = this.deps.renderer.info.render;
    this.panel.update({
      tickMs: this.tickN ? this.tickSum / this.tickN : 0,
      tickMaxMs: this.tickMax,
      frameMs: this.frameMs,
      calls: info.calls,
      triangles: info.triangles,
      tick: this.tickCount,
      hash: hash(),
      actors: this.actors
    });
    this.tickSum = this.tickMax = this.tickN = 0;
  }

  dispose(): void {
    this.panel.dispose();
    this.draw.dispose();
    delete window.__lfDev;
  }
}
