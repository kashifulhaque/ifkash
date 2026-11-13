// B12's loom lines (render-only; docs/game/05-enemies-and-bosses.md, "B12 The Unspooled": "loom
// lines (glowing lines across the arena that become beams after 30 ticks)").
//
// The core's `Loom` events (WASM kind 50) say where each line lies, when it becomes a beam, and
// when it goes. A laid line draws as a thin gold strip on the floor that brightens as its beam
// nears (the warning); the beam is a taller, white-hot sheet of light along the same line. The
// `CLEARED` event (the fight ended or reset) removes every line, and on a reset turns the
// Unspooled's model back into the Weaver.

import { AdditiveBlending, DoubleSide, Group, Mesh, MeshBasicMaterial, PlaneGeometry, type Scene } from 'three';
import { LOOM_BEAM, LOOM_CLEARED, LOOM_FADED, LOOM_LAID, type GameEvent } from '../../../engine/events';
import { STRIPE_METERS, renderAccess, stripeTexture, telegraphColors } from '../../access';
import { resetUnspooled } from './unspooled';

const TICK_SECONDS = 1 / 30;
/** Line slots, as in the core (`ai::bosses::b12::SLOTS`). */
const SLOTS = 6;
/** Height of a beam's sheet of light, in meters. */
const BEAM_HEIGHT = 2.6;

interface Line {
  group: Group;
  strip: Mesh;
  /** Stripes over the strip while it warns; the beam is solid (a shape cue, not only color). */
  hatch: Mesh;
  hatchMat: MeshBasicMaterial;
  length: number;
  sheet: Mesh;
  stripMat: MeshBasicMaterial;
  sheetMat: MeshBasicMaterial;
  /** Seconds of warning left, and its full length, while laid. */
  warn: number;
  warnTotal: number;
  /** Seconds of beam left, and its full length. */
  beam: number;
  beamTotal: number;
  /** Seconds left of the fade-out after the beam. */
  fade: number;
}

export class LoomFx {
  private readonly root = new Group();
  private readonly plane = new PlaneGeometry(1, 1);
  private readonly lines: Line[] = [];
  private readonly stripes = stripeTexture();

  constructor(scene: Scene) {
    this.root.name = 'b12-loom';
    scene.add(this.root);
    for (let i = 0; i < SLOTS; i++) {
      const stripMat = new MeshBasicMaterial({ color: 0xffd98a, transparent: true, opacity: 0, depthWrite: false, side: DoubleSide, blending: AdditiveBlending });
      const sheetMat = new MeshBasicMaterial({ color: 0xfff6e0, transparent: true, opacity: 0, depthWrite: false, side: DoubleSide, blending: AdditiveBlending });
      const strip = new Mesh(this.plane, stripMat);
      strip.rotation.x = -Math.PI / 2;
      strip.renderOrder = 2;
      const map = this.stripes?.clone() ?? null;
      if (map) map.needsUpdate = true;
      // Normal blending (not additive) so the stripes read on a pale floor as well as a dark one.
      const hatchMat = new MeshBasicMaterial({ color: 0xffd98a, map, transparent: true, opacity: 0, depthWrite: false, side: DoubleSide });
      const hatch = new Mesh(this.plane, hatchMat);
      hatch.rotation.x = -Math.PI / 2;
      hatch.position.y = 0.01;
      hatch.renderOrder = 2;
      const sheet = new Mesh(this.plane, sheetMat);
      sheet.renderOrder = 3;
      const group = new Group();
      group.add(strip, hatch, sheet);
      group.visible = false;
      this.root.add(group);
      this.lines.push({ group, strip, hatch, hatchMat, length: 1, sheet, stripMat, sheetMat, warn: 0, warnTotal: 1, beam: 0, beamTotal: 1, fade: 0 });
    }
  }

  handleEvents(events: GameEvent[]): void {
    for (const e of events) {
      if (e.kind !== 'loom') continue;
      if (e.what === LOOM_CLEARED) {
        for (const l of this.lines) this.hide(l);
        if (e.slot === 0) resetUnspooled();
        continue;
      }
      const l = this.lines[e.slot];
      if (!l) continue;
      if (e.what === LOOM_LAID) {
        // Lay the line out: the group sits at the center, turned so local +X runs along it.
        l.group.position.set(e.center[0], e.center[1] + 0.03, e.center[2]);
        l.group.rotation.y = Math.atan2(-e.dir[1], e.dir[0]);
        const length = e.halfLength * 2;
        l.strip.scale.set(length, e.halfWidth * 2, 1);
        l.hatch.scale.set(length, e.halfWidth * 2, 1);
        l.length = length;
        l.hatchMat.map?.repeat.set(length / STRIPE_METERS, 1);
        l.sheet.scale.set(length, BEAM_HEIGHT, 1);
        l.sheet.position.set(0, BEAM_HEIGHT / 2, 0);
        l.warnTotal = Math.max(e.ticks, 1) * TICK_SECONDS;
        l.warn = l.warnTotal;
        l.beam = 0;
        l.fade = 0;
        l.group.visible = true;
      } else if (e.what === LOOM_BEAM) {
        l.warn = 0;
        l.beamTotal = Math.max(e.ticks, 1) * TICK_SECONDS;
        l.beam = l.beamTotal;
      } else if (e.what === LOOM_FADED) {
        l.beam = 0;
        l.fade = 0.25;
      }
    }
  }

  update(dt: number): void {
    const colors = telegraphColors();
    // Reduced motion: the warning only brightens, and the beam holds steady (no pulsing).
    const calm = renderAccess().reducedMotion;
    for (const l of this.lines) {
      if (!l.group.visible) continue;
      l.stripMat.color.setHex(colors.warn);
      l.hatchMat.color.setHex(colors.warn);
      l.sheetMat.color.setHex(colors.live);
      if (l.warn > 0) {
        l.warn = Math.max(0, l.warn - dt);
        // The warning brightens and quickens as the beam nears.
        const u = 1 - l.warn / l.warnTotal;
        const pulse = calm ? 0.5 : 0.5 + 0.5 * Math.sin(u * u * 40);
        // With stripes drawn, the flat strip underneath only tints the floor.
        l.stripMat.opacity = l.hatchMat.map ? 0.1 + u * 0.15 + pulse * 0.05 : 0.25 + u * 0.45 + pulse * 0.15;
        // Stripes mean "coming"; with no stripe texture the strip keeps its flat color.
        l.hatchMat.opacity = l.hatchMat.map ? 0.7 + u * 0.3 : 0;
        l.sheetMat.opacity = 0;
      } else if (l.beam > 0) {
        l.beam = Math.max(0, l.beam - dt);
        l.stripMat.opacity = 1;
        l.hatchMat.opacity = 0;
        l.sheetMat.opacity = calm ? 0.65 : 0.55 + 0.25 * Math.sin(l.beam * 60);
      } else if (l.fade > 0) {
        l.fade = Math.max(0, l.fade - dt);
        const u = l.fade / 0.25;
        l.stripMat.opacity = u;
        l.hatchMat.opacity = 0;
        l.sheetMat.opacity = u * 0.5;
        if (l.fade === 0) this.hide(l);
      }
    }
  }

  private hide(l: Line): void {
    l.group.visible = false;
    l.warn = 0;
    l.beam = 0;
    l.fade = 0;
    l.stripMat.opacity = 0;
    l.hatchMat.opacity = 0;
    l.sheetMat.opacity = 0;
  }

  dispose(): void {
    this.root.removeFromParent();
    this.plane.dispose();
    for (const l of this.lines) {
      l.stripMat.dispose();
      l.sheetMat.dispose();
      l.hatchMat.map?.dispose();
      l.hatchMat.dispose();
    }
    this.stripes?.dispose();
  }
}
