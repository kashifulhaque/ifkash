// B08's creeping lava (docs/game/05-enemies-and-bosses.md, "B08 Forgeheart Colossus": "Phase 2 at
// 40%: The heart ignites; lava creeps across the floor over time").
//
// The core sends one event when the lava begins (`MechanicEvent` 171) and burns the player (172);
// the depth is a pure function of the ticks since it began (`b08::lava_depth_m`), so the renderer
// recomputes it from the game clock instead of being told. Four slabs, one per wall, advance
// toward the middle and stop at the maximum depth, which leaves the middle dry. The union of the
// four is the burn zone, and the slabs overlap in the corners the same way the core's test does.
//
// Cheap on purpose: one shared plane geometry, four materials that share one texture source, and
// four thin additive strips marking the advancing edge.

import { AdditiveBlending, Color, DoubleSide, Group, Mesh, MeshBasicMaterial, PlaneGeometry, type CanvasTexture, type Scene } from 'three';
import { B08 } from './arenas';
import { lavaTexture } from './textures';

/** Slabs and strips sit this far above the floor, each a little higher than the last so overlaps never flicker. */
const LIFT = 0.05;
/** Width of the bright strip along the lava's advancing edge, in meters. */
const EDGE_WIDTH = 0.7;
/** Meters of floor one texture tile covers. */
const TILE_M = 5;
/** Ticks a burn flash lasts. */
const BURN_FLASH_TICKS = 9;

const WALLS = ['w', 'e', 's', 'n'] as const;

export class LavaFx {
  readonly group = new Group();
  private readonly plane = new PlaneGeometry(1, 1);
  private readonly base: CanvasTexture | null = lavaTexture();
  private readonly textures: Array<CanvasTexture | null> = [];
  private readonly slabs: Mesh[] = [];
  private readonly strips: Mesh[] = [];
  private readonly slabMats: MeshBasicMaterial[] = [];
  private readonly edgeMat = new MeshBasicMaterial({ color: 0xffd27a, transparent: true, opacity: 0.85, depthWrite: false, blending: AdditiveBlending, fog: false, side: DoubleSide });
  private readonly tint = new Color();
  private startTick: number | null = null;
  private burnedAt = -1e9;

  constructor(scene: Scene) {
    this.group.name = 'b08-lava';
    this.group.visible = false;
    for (let i = 0; i < WALLS.length; i++) {
      // A clone shares the canvas, so the GPU holds one upload; each only differs in its `repeat`.
      const map = this.base ? this.base.clone() : null;
      if (map) map.needsUpdate = true;
      this.textures.push(map);
      const mat = new MeshBasicMaterial({ color: 0xffa060, map, side: DoubleSide, fog: false, depthWrite: true });
      this.slabMats.push(mat);
      const slab = new Mesh(this.plane, mat);
      slab.rotation.x = -Math.PI / 2;
      slab.renderOrder = 1;
      const strip = new Mesh(this.plane, this.edgeMat);
      strip.rotation.x = -Math.PI / 2;
      strip.renderOrder = 2;
      this.slabs.push(slab);
      this.strips.push(strip);
      this.group.add(slab, strip);
    }
    scene.add(this.group);
  }

  /** Phase 2 began at game tick `now`. A second event while it is already creeping changes nothing. */
  start(now: number): void {
    if (this.startTick !== null) return;
    this.startTick = now;
    this.group.visible = true;
  }

  /** The lava burned the player at `now`: the surface flares for a moment. */
  burned(now: number): void {
    this.burnedAt = now;
  }

  /** The fight ended or reset: the forge dries. */
  stop(): void {
    this.startTick = null;
    this.group.visible = false;
  }

  get active(): boolean {
    return this.startTick !== null;
  }

  /** How deep the lava is at game tick `now`, in meters (the core's `lava_depth_m`). */
  depthAt(now: number): number {
    if (this.startTick === null) return 0;
    return Math.min(Math.max(now - this.startTick, 0) * B08.perTick, B08.max);
  }

  update(now: number): void {
    if (this.startTick === null) return;
    const d = this.depthAt(now);
    const [hx, hy, hz] = B08.home;
    const { w, e, s, n } = B08.edges;
    const x0 = hx - w;
    const x1 = hx + e;
    const z0 = hz - s;
    const z1 = hz + n;
    const y = hy + LIFT;
    const visible = d > 0.02;
    // The surface pulses slowly, and flares for a few ticks after a burn.
    const flare = Math.max(0, 1 - (now - this.burnedAt) / BURN_FLASH_TICKS);
    const glow = 0.82 + 0.1 * Math.sin(now * 0.12) + flare * 0.4;
    this.tint.setRGB(1, 0.62 + flare * 0.25, 0.38 + flare * 0.3).multiplyScalar(glow);
    // Each slab: world-space center and extent on the floor, and its edge strip. West and east
    // span the full depth of the room; south and north span the width between them.
    const geo: Array<[number, number, number, number, number, number, boolean]> = [
      // cx, cz, extent x, extent z, strip cx, strip cz, strip runs along z
      [x0 + d / 2, (z0 + z1) / 2, d, z1 - z0, x0 + d, (z0 + z1) / 2, true],
      [x1 - d / 2, (z0 + z1) / 2, d, z1 - z0, x1 - d, (z0 + z1) / 2, true],
      [(x0 + x1) / 2, z0 + d / 2, x1 - x0, d, (x0 + x1) / 2, z0 + d, false],
      [(x0 + x1) / 2, z1 - d / 2, x1 - x0, d, (x0 + x1) / 2, z1 - d, false]
    ];
    for (let i = 0; i < 4; i++) {
      const [cx, cz, ex, ez, sx, sz, alongZ] = geo[i];
      const slab = this.slabs[i];
      const strip = this.strips[i];
      slab.visible = strip.visible = visible;
      if (!visible) continue;
      slab.position.set(cx, y + i * 0.004, cz);
      // The plane is rotated flat, so its local y runs along world -z: scale.y is the z extent.
      slab.scale.set(ex, ez, 1);
      strip.position.set(sx, y + 0.03 + i * 0.004, sz);
      strip.scale.set(alongZ ? EDGE_WIDTH : ex, alongZ ? ez : EDGE_WIDTH, 1);
      this.slabMats[i].color.copy(this.tint);
      const map = this.textures[i];
      if (map) {
        map.repeat.set(ex / TILE_M, ez / TILE_M);
        // The crust drifts slowly toward the middle of the room.
        map.offset.set((i < 2 ? (i === 0 ? 1 : -1) : 0) * now * 0.002, (i >= 2 ? (i === 2 ? 1 : -1) : 0) * now * 0.002);
      }
    }
    this.edgeMat.opacity = 0.6 + 0.25 * Math.sin(now * 0.2) + flare * 0.15;
  }

  dispose(): void {
    this.group.removeFromParent();
    this.plane.dispose();
    for (const m of this.slabMats) m.dispose();
    this.edgeMat.dispose();
    for (const t of this.textures) t?.dispose();
    this.base?.dispose();
  }
}
