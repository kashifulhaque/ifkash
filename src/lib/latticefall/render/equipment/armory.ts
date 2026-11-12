// The dev armory (`/game?engine=lf&dev=1&scene=armory`): every weapon on a plinth, every armor
// set on a mannequin, and the catalyst and spell effects looping, all labeled. It sits off the
// playable ground (see `layout.ts`) and is loaded only for that URL. Render-only.
//
// Sections: blades, heavy, hands, catalysts, armor. Pick the camera start with `&armory=NAME`
// or aim it yourself with `&cam=fixed:X,Y,Z,YAW,PITCH`.

import {
  Box3,
  BoxGeometry,
  CanvasTexture,
  CylinderGeometry,
  Group,
  LinearFilter,
  Mesh,
  MeshStandardMaterial,
  type PerspectiveCamera,
  Sprite,
  SpriteMaterial,
  Vector3
} from 'three';
import { ARMOR, CLASS_ORDER, WEAPONS, type ArmorSpec } from './catalog';
import { Builder } from './build';
import { buildArmorPiece, SET_LOOK, setKey, type ArmorPieceModel } from './armor';
import { ARMORY_Z, sectionX, type SectionName } from './layout';
import { SpellFx } from './spellFx';
import { buildWeapon, ELEMENT_COLOR, SCHOOL_COLOR, type WeaponModel, type WeaponSpec } from './weapons';

const ROW_ITEMS = 9;
const ROW_DEPTH = 2.0;
const TIER_RISE = 0.45;
const PLINTH_H = 0.1;

const SECTION_CLASSES: Record<Exclude<SectionName, 'armor'>, string[]> = {
  blades: ['Dagger', 'StraightSword', 'CurvedSword', 'ThrustingSword', 'Greatsword'],
  heavy: ['Axe', 'Hammer', 'Colossal', 'Spear', 'Halberd'],
  hands: ['Fist', 'Bow', 'Shield'],
  catalysts: ['Catalyst']
};

/** A text plate as a camera-facing sprite. */
function label(text: string, sub: string | null, accent: number, width = 0.9): Sprite {
  const w = 512;
  const h = 128;
  const cv = document.createElement('canvas');
  cv.width = w;
  cv.height = h;
  const g = cv.getContext('2d')!;
  g.fillStyle = 'rgba(24,26,30,0.88)';
  g.fillRect(0, 0, w, h);
  g.fillStyle = `#${accent.toString(16).padStart(6, '0')}`;
  g.fillRect(0, 0, 10, h);
  g.fillStyle = '#f2efe6';
  g.textBaseline = 'middle';
  g.font = '600 44px system-ui, sans-serif';
  // Shrink long names to fit.
  let size = 44;
  while (g.measureText(text).width > w - 44 && size > 22) {
    size -= 2;
    g.font = `600 ${size}px system-ui, sans-serif`;
  }
  g.fillText(text, 26, sub ? 46 : 64);
  if (sub) {
    g.fillStyle = '#b8b3a5';
    g.font = '400 32px system-ui, sans-serif';
    g.fillText(sub, 26, 96);
  }
  const tex = new CanvasTexture(cv);
  tex.minFilter = LinearFilter;
  tex.generateMipmaps = false;
  const sp = new Sprite(new SpriteMaterial({ map: tex, depthWrite: false, transparent: true }));
  sp.scale.set(width, (width * h) / w, 1);
  return sp;
}

interface Station {
  label: string;
  school: 'Formula' | 'Litany';
  element: string | null;
  at: Vector3;
  next: number;
}

export interface ArmoryStats {
  weapons: number;
  armorPieces: number;
  armorSets: number;
  catalysts: number;
  stations: number;
}

export class Armory {
  readonly group = new Group();
  readonly fx = new SpellFx();
  readonly stats: ArmoryStats = { weapons: 0, armorPieces: 0, armorSets: 0, catalysts: 0, stations: 0 };
  /** Section anchors for aiming the camera by hand. */
  readonly anchors: Record<string, [number, number, number]> = {};
  private weapons: WeaponModel[] = [];
  private armor: ArmorPieceModel[] = [];
  private builders: Builder[] = [];
  private materials: MeshStandardMaterial[] = [];
  private labels: Sprite[] = [];
  private geoms: Array<CylinderGeometry | BoxGeometry> = [];
  private stations: Station[] = [];
  private clock = 0;
  private plinthGeo = new CylinderGeometry(0.28, 0.32, PLINTH_H, 12);
  private plinthMat = new MeshStandardMaterial({ color: 0x6a665c, roughness: 0.8, flatShading: true });
  private floorMat = new MeshStandardMaterial({ color: 0x2b2d31, roughness: 0.95, flatShading: true });

  constructor() {
    this.group.name = 'armory';
    this.group.add(this.fx.points);
    for (const s of ['blades', 'heavy', 'hands'] as const) this.weaponSection(s);
    this.catalystSection();
    this.armorSection();
    (globalThis as Record<string, unknown>).__lfArmory = this;
  }

  // -- weapons ---------------------------------------------------------------------------

  private floor(cx: number, rows: number, width: number): void {
    const depth = rows * ROW_DEPTH + 1.2;
    const geo = new BoxGeometry(1, 1, 1);
    this.geoms.push(geo);
    const f = new Mesh(geo, this.floorMat);
    f.scale.set(width, 0.1, depth);
    f.position.set(cx, 0.0, ARMORY_Z - ((rows - 1) * ROW_DEPTH) / 2 - 0.1);
    this.group.add(f);
  }

  private weaponSection(section: 'blades' | 'heavy' | 'hands'): void {
    const classes = SECTION_CLASSES[section];
    const specs = CLASS_ORDER.filter((c) => classes.includes(c)).flatMap((c) => WEAPONS.filter((w) => w.class === c));
    this.place(section, specs);
  }

  private catalystSection(): void {
    const specs = WEAPONS.filter((w) => w.class === 'Catalyst');
    // The stations stand on a second, higher row behind the weapons.
    this.place('catalysts', specs, 1.25, 2);
    const cx = sectionX('catalysts');
    const demos: Array<[string, 'Formula' | 'Litany', string | null]> = [
      ['Formula cast', 'Formula', null],
      ['Formula: Aether', 'Formula', 'Aether'],
      ['Formula: Storm', 'Formula', 'Storm'],
      ['Litany cast', 'Litany', null],
      ['Litany: Fire', 'Litany', 'Fire'],
      ['Litany: Radiance', 'Litany', 'Radiance']
    ];
    const z = ARMORY_Z - ROW_DEPTH;
    demos.forEach(([name, school, element], i) => {
      const x = cx + (i - (demos.length - 1) / 2) * 1.5;
      const color = element ? ELEMENT_COLOR[element] : SCHOOL_COLOR[school];
      const pl = new Mesh(this.plinthGeo, this.plinthMat);
      pl.position.set(x, TIER_RISE + PLINTH_H / 2, z);
      this.group.add(pl);
      const l = label(name, 'cast loop', color, 1.3);
      l.position.set(x, TIER_RISE + 0.11, z + 0.62);
      this.group.add(l);
      this.labels.push(l);
      // The anchor floats above the plinth, shimmers all the time, and casts toward the viewer.
      const anchor = new Group();
      anchor.position.set(x, TIER_RISE + 0.75, z);
      this.group.add(anchor);
      this.fx.track(anchor, [0, 0, 0], school, element);
      this.stations.push({ label: name, school, element, at: anchor.position.clone(), next: 0.3 + i * 0.2 });
    });
    this.stats.stations = demos.length;
  }

  /** Lays specs out in tiered rows of plinths, with labels. */
  private place(section: SectionName, specs: WeaponSpec[], spacing = 0.85, minRows = 1): void {
    const cx = sectionX(section);
    const rows = Math.max(minRows, Math.ceil(specs.length / ROW_ITEMS));
    const widest = Math.min(specs.length, ROW_ITEMS);
    this.floor(cx, rows, Math.max(widest * spacing, section === 'catalysts' ? 8.4 : 0) + 1.6);
    this.anchors[section] = [cx, 0, ARMORY_Z];
    // Each back row stands on a step.
    for (let r = 1; r < rows; r++) {
      const geo = new BoxGeometry(1, 1, 1);
      this.geoms.push(geo);
      const step = new Mesh(geo, this.floorMat);
      step.scale.set(widest * spacing + 1.2, r * TIER_RISE, 1.5);
      step.position.set(cx, (r * TIER_RISE) / 2, ARMORY_Z - r * ROW_DEPTH);
      this.group.add(step);
    }
    specs.forEach((spec, i) => {
      const row = Math.floor(i / ROW_ITEMS);
      const inRow = Math.min(ROW_ITEMS, specs.length - row * ROW_ITEMS);
      const col = i % ROW_ITEMS;
      const x = cx + (col - (inRow - 1) / 2) * spacing;
      const z = ARMORY_Z - row * ROW_DEPTH;
      const y = row * TIER_RISE;
      const model = buildWeapon(spec);
      this.weapons.push(model);
      // Stand the weapon on its plinth, centered, with its lowest point resting on top.
      model.group.rotation.y = 0.28;
      model.group.updateMatrixWorld(true);
      const box2 = new Box3().setFromObject(model.group);
      const c2 = box2.getCenter(new Vector3());
      model.group.position.set(x - c2.x, y + PLINTH_H - box2.min.y, z - c2.z);
      this.group.add(model.group);
      const pl = new Mesh(this.plinthGeo, this.plinthMat);
      pl.position.set(x, y + PLINTH_H / 2, z);
      this.group.add(pl);
      const accent = model.glow ?? (spec.unique ? 0xd0a64a : 0x8a9199);
      const sub = [spec.class, spec.element, spec.school].filter(Boolean).join(' · ');
      const l = label(spec.name, sub, accent, 0.82);
      l.position.set(x, y + 0.11, z + 0.62);
      l.renderOrder = 5;
      this.group.add(l);
      this.labels.push(l);
      if (model.castPoint && spec.school) {
        this.fx.track(model.group, model.castPoint, spec.school);
        this.stats.catalysts++;
      }
      this.stats.weapons++;
    });
    const t = label(section[0].toUpperCase() + section.slice(1), `${specs.length} weapons`, 0xd0a64a, 1.6);
    t.position.set(cx, rows * TIER_RISE + 2.6 + (rows > 1 ? 0.6 : 0), ARMORY_Z - (rows - 1) * ROW_DEPTH);
    this.group.add(t);
    this.labels.push(t);
  }

  // -- armor -----------------------------------------------------------------------------

  private mannequin(x: number, z: number): Builder {
    const b = new Builder();
    const clay = 0xcfc7b4;
    const g = b.group;
    b.cyl(0.2, 0.22, 0.04, [0, 0.02, 0], 0x4a4740, {}, g);
    for (const s of [1, -1]) {
      b.cyl(0.075, 0.06, 0.92, [s * 0.1, 0.5, 0], clay, {}, g);
      b.cyl(0.05, 0.05, 0.62, [s * 0.24, 1.16, 0], clay, { rot: [0, 0, s * 0.12] }, g);
      b.sphere(0.06, [s * 0.22, 1.43, 0], clay, {}, g);
    }
    b.box([0.28, 0.28, 0.16], [0, 1.03, 0], clay, {}, g);
    b.box([0.34, 0.4, 0.2], [0, 1.3, 0], clay, {}, g);
    b.cyl(0.05, 0.06, 0.1, [0, 1.52, 0], clay, {}, g);
    b.sphere(0.115, [0, 1.62, 0], clay, { scale: [0.9, 1.1, 1] }, g);
    g.position.set(x, 0, z);
    this.group.add(g);
    this.builders.push(b);
    return b;
  }

  private armorSection(): void {
    const cx = sectionX('armor');
    const sets = new Map<string, ArmorSpec[]>();
    for (const a of ARMOR) {
      const key = setKey(a.id);
      sets.set(key, [...(sets.get(key) ?? []), a]);
    }
    // The bare mannequin first, then each set, then the relic pieces on their own.
    const order = ['', ...[...sets.keys()].filter((k) => sets.get(k)!.length > 1), ...[...sets.keys()].filter((k) => sets.get(k)!.length === 1)];
    const spacing = 1.25;
    this.floor(cx, 1, order.length * spacing + 1.2);
    this.anchors.armor = [cx, 0, ARMORY_Z];
    order.forEach((key, i) => {
      const x = cx + (i - (order.length - 1) / 2) * spacing;
      const m = this.mannequin(x, ARMORY_Z);
      let name = 'Unarmored';
      let sub = 'mannequin';
      if (key) {
        const pieces = sets.get(key)!;
        name = pieces[0].set || pieces.map((p) => p.name).join(' + ');
        sub = pieces.length > 1 ? 'head + body set' : `relic ${pieces[0].slot.toLowerCase()}`;
        for (const p of pieces) {
          const model = buildArmorPiece(p.id);
          if (!model) continue;
          this.armor.push(model);
          for (const part of model.parts) m.group.add(part.object);
          this.stats.armorPieces++;
        }
        this.stats.armorSets++;
      }
      const l = label(name, sub, SET_LOOK[key]?.trim ?? 0x8a9199, 1.15);
      l.position.set(x, 0.12, ARMORY_Z + 0.55);
      this.group.add(l);
      this.labels.push(l);
    });
    const t = label('Armor sets', `${this.stats.armorSets} sets, ${this.stats.armorPieces} pieces`, 0xd0a64a, 1.6);
    t.position.set(cx, 2.3, ARMORY_Z - 0.6);
    this.group.add(t);
    this.labels.push(t);
  }

  // -- per frame -------------------------------------------------------------------------

  update(dt: number, camera: PerspectiveCamera, viewportPx: number): void {
    this.clock += dt;
    for (const s of this.stations) {
      s.next -= dt;
      if (s.next > 0) continue;
      s.next = 1.6;
      this.fx.cast(s.at, new Vector3(0, 0.05, 1), { school: s.school, element: s.element });
    }
    this.fx.update(dt, camera, viewportPx);
  }

  dispose(): void {
    this.group.removeFromParent();
    for (const w of this.weapons) w.dispose();
    for (const a of this.armor) a.dispose();
    for (const b of this.builders) b.dispose();
    for (const l of this.labels) {
      (l.material.map as CanvasTexture | null)?.dispose();
      l.material.dispose();
    }
    for (const g of this.geoms) g.dispose();
    this.plinthGeo.dispose();
    this.plinthMat.dispose();
    this.floorMat.dispose();
    this.fx.dispose();
    for (const m of this.materials) m.dispose();
    if ((globalThis as Record<string, unknown>).__lfArmory === this) delete (globalThis as Record<string, unknown>).__lfArmory;
  }
}
