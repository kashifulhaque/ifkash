// The performance benchmark (WP-081). With `&dev=1&bench=1` it flies a fixed camera path
// through every area, region by region, on the medium quality preset, and reports frame
// work, draw calls, and triangles against the budgets in `budgets.ts`.
//
// URL: `/game?dev=1&engine=lf&bench=1` runs every area; `&bench=verge,castle` runs those.
// `&benchstep=M` changes how many meters the player walks per frame (default 1.5).
//
// It builds its own scene per area from the same renderers the game uses (level, props, actors),
// with a crowd of stand-in enemies around the player so the actor cost is in the numbers. It
// draws on the game's own `WebGLRenderer` and hides the game's scene while it runs.
//
// Frame time is "work": JS plus GPU, waited out with `gl.finish()`, so it doesn't hide behind
// vsync. It is the machine's number; a throttled or hidden tab, or a fast GPU, will flatter it.
// Draw calls and triangles are exact and don't depend on the machine.

import { PerspectiveCamera, Scene, Vector2, type WebGLRenderer } from 'three';
import { ActorRenderer } from '../actors/ActorRenderer';
import { fetchManifest, type LevelManifest } from '../kits/manifest';
import { getQuality, setQuality, type QualityName } from '../lighting/quality';
import { PropsRenderer } from '../props';
import { LevelRenderer } from '../terrain/LevelRenderer';
import type { DrawActor } from '../../engine/snapshot';
import { BUDGETS, judge, stats, type FrameSample, type Stats } from './budgets';
import { buildFrames, buildWaypoints, heightAt, type RouteFrame } from './route';
import { auditTextures, type TextureAudit } from './textures';

/** Every area that has a baked manifest in `static/latticefall/levels/`. */
export const BENCH_AREAS = [
  'waking_hollow',
  'hollowhearth',
  'verge',
  'mossgut_cave',
  'mirrormere',
  'drowned_belfry',
  'sunken_ossuary',
  'glasswing_grotto',
  'rimeshelf',
  'whitewarden',
  'cinderdeep_mine',
  'hollowdeep',
  'castle',
  'spire',
  'bound_ring',
  'mechanics_yard',
  'test_field'
] as const;

/** Enemy kinds for the stand-in crowd, one of each silhouette class (registry numbers). */
/** The simulated frame length in milliseconds (60 Hz). */
const FRAME_MS = 1000 / 60;

const CROWD_KINDS = [7000, 7010, 7020, 7030, 7040, 7050, 7051, 7054, 7056, 7081, 7083, 7084];
/** The active-actor budget, less the player: the worst case the activity cells allow. */
const CROWD_SIZE = BUDGETS.activeActors - 1;

export interface RegionRow {
  region: string;
  frames: number;
  workMs: Stats;
  calls: Stats;
  triangles: Stats;
}

export interface AreaRow {
  area: string;
  ok: boolean;
  broken: string[];
  frames: number;
  workMs: Stats;
  calls: Stats;
  triangles: Stats;
  /** Geometries and textures alive at the end of the route, before unloading. */
  geometries: number;
  textures: number;
  /** Geometries and textures left after the area was unloaded, less what was there before it loaded. */
  leakedGeometries: number;
  leakedTextures: number;
  /** Terrain chunks the area has, and the most that were built at once while the route ran. */
  terrainChunks: number;
  terrainBuiltMax: number;
  textureAudit: TextureAudit;
  regions: RegionRow[];
}

export interface BenchReport {
  preset: QualityName;
  viewport: [number, number];
  userAgent: string;
  /** `visible` or `hidden`: a hidden tab is throttled, so its frame work is unreliable. */
  visibility: string;
  gpu: string;
  startedAt: string;
  areas: AreaRow[];
  ok: boolean;
}

declare global {
  interface Window {
    __lfBench?: { status: 'running' | 'done' | 'failed'; progress: string; report: BenchReport | null; error?: string };
  }
}

/**
 * Waits for the next frame: an animation frame or a 4 ms timer, whichever comes first, so a
 * hidden tab (which fires no animation frames) still finishes. Work is timed inside the frame,
 * so the pacing doesn't change it.
 */
const nextFrame = (): Promise<number> =>
  new Promise((resolve) => {
    let done = false;
    const finish = (): void => {
      if (done) return;
      done = true;
      resolve(performance.now());
    };
    requestAnimationFrame(finish);
    setTimeout(finish, 4);
  });

function gpuName(gl: WebGL2RenderingContext | WebGLRenderingContext): string {
  const ext = gl.getExtension('WEBGL_debug_renderer_info');
  return ext ? String(gl.getParameter(ext.UNMASKED_RENDERER_WEBGL)) : 'unknown';
}

type Ring = Array<{ r: number; a: number; kind: number }>;

/** The stand-in crowd: fixed offsets around the player, so the same frame draws the same actors. */
function crowd(size: number): Array<{ r: number; a: number; kind: number }> {
  const out = [];
  for (let i = 0; i < size; i++) {
    out.push({ r: 5 + ((i * 7) % 29), a: i * 2.399963, kind: CROWD_KINDS[i % CROWD_KINDS.length] });
  }
  return out;
}

export interface BenchOptions {
  renderer: WebGLRenderer;
  /** The game's own scene; hidden while the benchmark draws. */
  gameScene: Scene;
  areas?: readonly string[];
  stepM?: number;
  /** Stand-in enemies around the player; default is the active-actor budget less the player. */
  crowd?: number;
  onProgress?: (text: string) => void;
}

/** Runs the benchmark and resolves with the report. Restores the renderer and preset after. */
export async function runBenchmark(opts: BenchOptions): Promise<BenchReport> {
  const { renderer } = opts;
  const gl = renderer.getContext();
  const previousPreset = getQuality().name;
  const previousRatio = renderer.getPixelRatio();
  const size = renderer.getSize(new Vector2());
  setQuality('medium');
  renderer.setPixelRatio(1);
  renderer.setSize(1920, 1080, false);
  const wasVisible = opts.gameScene.visible;
  opts.gameScene.visible = false;
  const report: BenchReport = {
    preset: 'medium',
    viewport: [1920, 1080],
    userAgent: navigator.userAgent,
    visibility: document.visibilityState,
    gpu: gpuName(gl),
    startedAt: new Date().toISOString(),
    areas: [],
    ok: true
  };
  try {
    for (const area of opts.areas ?? BENCH_AREAS) {
      opts.onProgress?.(`${area} (${report.areas.length + 1}/${(opts.areas ?? BENCH_AREAS).length})`);
      const row = await benchArea(renderer, area, opts.stepM ?? 1.5, opts.crowd ?? CROWD_SIZE);
      if (row) {
        report.areas.push(row);
        if (!row.ok) report.ok = false;
      }
    }
  } finally {
    opts.gameScene.visible = wasVisible;
    renderer.setPixelRatio(previousRatio);
    renderer.setSize(size.x, size.y, false);
    setQuality(previousPreset);
  }
  return report;
}

async function benchArea(renderer: WebGLRenderer, area: string, stepM: number, crowdSize: number): Promise<AreaRow | null> {
  const manifest = await fetchManifest(area);
  if (!manifest) return null;
  const gl = renderer.getContext();
  const geometriesBefore = renderer.info.memory.geometries;
  const texturesBefore = renderer.info.memory.textures;
  let terrainChunks = 0;
  let terrainBuiltMax = 0;
  const scene = new Scene();
  const camera = new PerspectiveCamera(60, 1920 / 1080, 0.1, 400);
  const level = new LevelRenderer(scene, renderer);
  const props = new PropsRenderer(scene);
  const actors = new ActorRenderer(scene, renderer, {});
  const samples: Array<FrameSample & { region: string }> = [];
  let textureAudit: TextureAudit = { count: 0, bytes: 0, largest: 0 };
  let geometries = 0;
  let textures = 0;
  try {
    await Promise.all([level.load(area), props.load(area), actors.load().catch(() => undefined)]);
    const waypoints = buildWaypoints(manifest);
    const frames = buildFrames(manifest, waypoints, stepM);
    const ring = crowd(crowdSize);
    // Let the first frame's streaming requests (piece models) land before timing starts.
    await warm(renderer, scene, camera, level, props, actors, frames[0], ring, manifest);
    for (const f of frames) {
      await nextFrame();
      // A fixed step, so the same route animates the same way on every machine.
      const frameMs = FRAME_MS;
      const start = performance.now();
      drawFrame(renderer, scene, camera, level, props, actors, f, ring, manifest, frameMs);
      gl.finish();
      const workMs = performance.now() - start;
      const info = renderer.info.render;
      samples.push({ workMs, calls: info.calls, triangles: info.triangles, region: f.region });
      const ts = level.terrainStats;
      if (ts) {
        terrainChunks = ts.chunks;
        terrainBuiltMax = Math.max(terrainBuiltMax, ts.built);
      }
    }
    geometries = renderer.info.memory.geometries;
    textures = renderer.info.memory.textures;
    textureAudit = auditTextures(scene);
  } finally {
    actors.prune(new Set());
    actors.dispose();
    props.dispose();
    level.dispose();
    scene.clear();
  }
  // Give the renderer a frame to release what was disposed.
  await nextFrame();
  const leakedGeometries = renderer.info.memory.geometries - geometriesBefore;
  const leakedTextures = renderer.info.memory.textures - texturesBefore;

  const all = samples;
  const verdict = judge(all);
  const byRegion = new Map<string, typeof samples>();
  for (const s of samples) {
    const list = byRegion.get(s.region) ?? [];
    list.push(s);
    byRegion.set(s.region, list);
  }
  const regions: RegionRow[] = [...byRegion.entries()].map(([region, list]) => ({
    region,
    frames: list.length,
    workMs: stats(list.map((s) => s.workMs)),
    calls: stats(list.map((s) => s.calls)),
    triangles: stats(list.map((s) => s.triangles))
  }));
  return {
    area,
    ok: verdict.ok,
    broken: verdict.broken,
    frames: all.length,
    workMs: stats(all.map((s) => s.workMs)),
    calls: stats(all.map((s) => s.calls)),
    triangles: stats(all.map((s) => s.triangles)),
    geometries,
    textures,
    leakedGeometries,
    leakedTextures,
    terrainChunks,
    terrainBuiltMax,
    textureAudit,
    regions
  };
}

function fakeActors(f: RouteFrame, ring: Ring, manifest: LevelManifest): DrawActor[] {
  const base = (id: number, kind: number, x: number, z: number, yaw: number): DrawActor => ({
    id,
    kind,
    flags: 0,
    x,
    y: groundAt(manifest, x, z, f.y),
    z,
    yaw,
    moveId: 0,
    moveFrame: 0,
    hpFrac: 1,
    poiseFrac: 1,
    statusMask: 0
  });
  const out = [base(1, 0, f.x, f.z, f.yaw)];
  ring.forEach((c, i) => out.push(base(100 + i, c.kind, f.x + Math.sin(c.a) * c.r, f.z + Math.cos(c.a) * c.r, c.a + Math.PI)));
  return out;
}

function groundAt(m: LevelManifest, x: number, z: number, fallback: number): number {
  return heightAt(m.terrain, x, z) ?? fallback;
}

function drawFrame(
  renderer: WebGLRenderer,
  scene: Scene,
  camera: PerspectiveCamera,
  level: LevelRenderer,
  props: PropsRenderer,
  actors: ActorRenderer,
  f: RouteFrame,
  ring: Ring,
  manifest: LevelManifest,
  frameMs: number
): void {
  const drawn = fakeActors(f, ring, manifest);
  const live = new Set<number>();
  for (const a of drawn) {
    live.add(a.id);
    actors.update(a, frameMs);
  }
  actors.playerId = 1;
  actors.prune(live);
  // The follow camera: 4.2 m behind the player's heading, 2.6 m above the feet, looking at the chest.
  const back = 4.2;
  camera.position.set(f.x - Math.sin(f.yaw) * back, f.y + 2.6, f.z - Math.cos(f.yaw) * back);
  camera.lookAt(f.x, f.y + 1.3, f.z);
  level.setVisibility(props.visibility);
  level.update(camera, { x: f.x, y: f.y, z: f.z }, frameMs);
  if (props.active) props.update(frameMs);
  actors.updateFx(Math.min(frameMs, 100) / 1000, camera, 1080);
  renderer.render(scene, camera);
}

async function warm(
  renderer: WebGLRenderer,
  scene: Scene,
  camera: PerspectiveCamera,
  level: LevelRenderer,
  props: PropsRenderer,
  actors: ActorRenderer,
  f: RouteFrame,
  ring: Ring,
  manifest: LevelManifest
): Promise<void> {
  for (let i = 0; i < 20; i++) {
    await nextFrame();
    drawFrame(renderer, scene, camera, level, props, actors, f, ring, manifest, FRAME_MS);
  }
}

/** Formats the report as a Markdown table, one row per area and per region. */
export function reportMarkdown(r: BenchReport): string {
  const f = (n: number, d = 1): string => n.toFixed(d);
  const lines = [
    `Benchmark: ${r.preset} preset, ${r.viewport[0]}x${r.viewport[1]}, ${r.gpu}`,
    `Budgets: frame p95 <= ${BUDGETS.frameMs} ms, draw calls <= ${BUDGETS.drawCalls}, triangles <= ${BUDGETS.triangles}`,
    `Draw calls and triangles are exact. Work ms is this machine's JS plus GPU time per frame (gl.finish), not a 2021 integrated GPU's. Tab was ${r.visibility}.`,
    '',
    '| Area / region | Frames | Calls mean / max | Triangles mean / max | Work ms mean / p95 / max | Verdict |',
    '|---|---|---|---|---|---|'
  ];
  for (const a of r.areas) {
    lines.push(
      `| **${a.area}** | ${a.frames} | ${f(a.calls.mean, 0)} / ${a.calls.max} | ${f(a.triangles.mean, 0)} / ${a.triangles.max} | ${f(a.workMs.mean)} / ${f(a.workMs.p95)} / ${f(a.workMs.max)} | ${a.ok ? 'pass' : 'FAIL: ' + a.broken.join('; ')} |`
    );
    for (const g of a.regions) {
      lines.push(`| &nbsp;&nbsp;${g.region} | ${g.frames} | ${f(g.calls.mean, 0)} / ${g.calls.max} | ${f(g.triangles.mean, 0)} / ${g.triangles.max} | ${f(g.workMs.mean)} / ${f(g.workMs.p95)} / ${f(g.workMs.max)} | |`);
    }
  }
  lines.push('', '| Area | Geometries | Textures | Left after unload (geo / tex) | Terrain chunks built max / total | Texture bytes (count, largest edge) |', '|---|---|---|---|---|---|');
  for (const a of r.areas) {
    lines.push(
      `| ${a.area} | ${a.geometries} | ${a.textures} | ${a.leakedGeometries} / ${a.leakedTextures} | ${a.terrainBuiltMax} / ${a.terrainChunks} | ${a.textureAudit.bytes} (${a.textureAudit.count}, ${a.textureAudit.largest}px) |`
    );
  }
  lines.push('', `Overall: ${r.ok ? 'PASS' : 'FAIL'}`);
  return lines.join('\n');
}

/**
 * Runs the benchmark with a progress panel over the canvas, then shows the report there, logs it,
 * and leaves it on `window.__lfBench`. Used by the dev tools for `&bench=...`.
 */
export async function runBenchmarkWithPanel(opts: BenchOptions & { parent: HTMLElement }): Promise<BenchReport | null> {
  const panel = document.createElement('pre');
  panel.setAttribute('data-lf-bench', '');
  panel.style.cssText =
    'position:absolute;inset:8px;margin:0;padding:12px;overflow:auto;z-index:50;background:var(--surface,#fff);color:var(--ink,#111);' +
    'font:12px/1.4 ui-monospace,monospace;white-space:pre;border:1px solid var(--line,#999);border-radius:6px';
  opts.parent.appendChild(panel);
  const state: NonNullable<Window['__lfBench']> = { status: 'running', progress: 'starting', report: null };
  window.__lfBench = state;
  const progress = (text: string): void => {
    state.progress = text;
    panel.textContent = `Benchmark running: ${text}`;
  };
  try {
    const report = await runBenchmark({ ...opts, onProgress: progress });
    state.report = report;
    state.status = 'done';
    const text = reportMarkdown(report);
    panel.textContent = text;
    console.log(`[latticefall bench]\n${text}`);
    return report;
  } catch (err) {
    state.status = 'failed';
    state.error = err instanceof Error ? (err.stack ?? err.message) : String(err);
    panel.textContent = `Benchmark failed: ${state.error}`;
    console.error('[latticefall bench] failed', err);
    return null;
  }
}
