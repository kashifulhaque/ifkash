// Debug draw for the overlays. Each overlay owns a group in the scene that is hidden, and
// not updated, until its flag is on. Hitboxes, hurtboxes, the nav grid and AI state have
// no data to draw until those systems exist; their groups and counters are here so the
// flags work today and each system only has to fill in its `draw`.

import {
  CapsuleGeometry,
  Color,
  Group,
  Material,
  Mesh,
  MeshBasicMaterial,
  type BufferGeometry,
  type Scene
} from 'three';
import { TEST_FIELD_BOUNDS, buildLevelProxy, type RaycastFn } from '../render/camera';
import type { DrawActor } from '../engine/snapshot';
import { OVERLAY_NAMES, type OverlayName } from './params';

const CAPSULE_RADIUS = 0.35;
const CAPSULE_LENGTH = 1.0;

function token(name: string, fallback: string): Color {
  try {
    const raw = getComputedStyle(document.documentElement).getPropertyValue(name).trim();
    if (raw) return new Color(raw);
  } catch {
    // Fall through to the fallback.
  }
  return new Color(fallback);
}

interface Overlay {
  group: Group;
  /** Redraws from this frame's actors; returns how many shapes it drew. */
  draw(actors: DrawActor[]): number;
  /** Called the first time the overlay turns on, for lazy setup. */
  build?(): void;
}

export class DebugDraw {
  private overlays = new Map<OverlayName, Overlay>();
  private enabled = new Set<OverlayName>();
  private disposables: Array<BufferGeometry | Material> = [];
  private root = new Group();
  private capsules = new Map<number, Mesh>();
  private levelBuilt = false;
  /** Shapes drawn last frame, per overlay. */
  readonly counts: Record<OverlayName, number> = { colliders: 0, hitboxes: 0, hurtboxes: 0, nav: 0, ai: 0 };

  constructor(
    private scene: Scene,
    private raycast: RaycastFn,
    private scenario: string
  ) {
    this.root.name = 'debug-draw';
    for (const name of OVERLAY_NAMES) {
      const group = new Group();
      group.name = `debug-${name}`;
      group.visible = false;
      this.root.add(group);
    }
    this.overlays.set('colliders', this.colliders());
    // No hitbox, hurtbox, nav or AI data exists yet; these draw nothing until their systems land.
    for (const name of ['hitboxes', 'hurtboxes', 'nav', 'ai'] as const) {
      this.overlays.set(name, { group: this.group(name), draw: () => 0 });
    }
    this.scene.add(this.root);
  }

  private group(name: OverlayName): Group {
    return this.root.getObjectByName(`debug-${name}`) as Group;
  }

  /** The level's colliders (the field as a wireframe) and each actor's capsule. */
  private colliders(): Overlay {
    const group = this.group('colliders');
    const color = token('--fjord', '#2f5d74');
    const capsuleGeo = new CapsuleGeometry(CAPSULE_RADIUS, CAPSULE_LENGTH, 4, 10);
    const wire = new MeshBasicMaterial({ color, wireframe: true, depthTest: false, transparent: true, opacity: 0.9 });
    this.disposables.push(capsuleGeo, wire);
    return {
      group,
      build: () => {
        // The core exposes ray casts, not collider shapes, so the level is sampled the same way
        // the camera proxy is. Only the test field has bounds to sample.
        if (this.levelBuilt || this.scenario !== 'test_field') return;
        this.levelBuilt = true;
        const proxy = buildLevelProxy(this.raycast, TEST_FIELD_BOUNDS, color, color);
        const level = new Mesh(
          proxy.geometry,
          new MeshBasicMaterial({ color, wireframe: true, transparent: true, opacity: 0.35, depthTest: true })
        );
        level.position.y = 0.02;
        group.add(level);
        this.disposables.push(proxy.geometry, level.material as Material, proxy.material as Material);
      },
      draw: (actors) => {
        const live = new Set<number>();
        for (const a of actors) {
          live.add(a.id);
          let m = this.capsules.get(a.id);
          if (!m) {
            m = new Mesh(capsuleGeo, wire);
            group.add(m);
            this.capsules.set(a.id, m);
          }
          m.position.set(a.x, a.y + (CAPSULE_LENGTH + CAPSULE_RADIUS * 2) / 2, a.z);
          m.rotation.y = a.yaw;
        }
        for (const [id, m] of this.capsules) {
          if (!live.has(id)) {
            group.remove(m);
            this.capsules.delete(id);
          }
        }
        return actors.length + (this.levelBuilt ? 1 : 0);
      }
    };
  }

  setEnabled(name: OverlayName, on: boolean): void {
    const overlay = this.overlays.get(name);
    if (!overlay) return;
    if (on) {
      overlay.build?.();
      this.enabled.add(name);
    } else {
      this.enabled.delete(name);
      this.counts[name] = 0;
    }
    overlay.group.visible = on;
  }

  isEnabled(name: OverlayName): boolean {
    return this.enabled.has(name);
  }

  /** Redraws the enabled overlays from this frame's actors. Does nothing when none are on. */
  update(actors: DrawActor[]): void {
    for (const name of this.enabled) this.counts[name] = this.overlays.get(name)?.draw(actors) ?? 0;
  }

  dispose(): void {
    this.scene.remove(this.root);
    for (const d of this.disposables) d.dispose();
    this.disposables = [];
    this.capsules.clear();
  }
}
