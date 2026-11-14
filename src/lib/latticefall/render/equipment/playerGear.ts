// Puts the equipped weapons and armor on the player's model. The simulation holds the
// equipment (the menu state's slot words); this file only draws it: weapons ride the hand
// bones, armor parts ride the chest, abdomen, head and upper-arm bones, and the body armor
// recolors the model's cloth. Render-only.

import { Color, type Material, Matrix4, Object3D, Quaternion, Vector3 } from 'three';
import { armorByNumber, weaponByNum } from './catalog';
import { buildArmorPiece, type ArmorPieceModel } from './armor';
import { buildWeapon, type WeaponModel, type WeaponSpec } from './weapons';
import type { SpellFx } from './spellFx';

/** What the player has on, as registry numbers; 0 for an empty slot. */
export interface Equipped {
  right: number;
  left: number;
  head: number;
  body: number;
}

export const NO_EQUIPMENT: Equipped = { right: 0, left: 0, head: 0, body: 0 };

/** The slot words of `menu_state` that matter here (see `crates/wasm/src/menu.rs`). */
export function equippedFromMenuWords(w: ArrayLike<number>): Equipped {
  const rightActive = w[16] === 1 ? 1 : 0;
  const leftActive = w[17] === 1 ? 1 : 0;
  return { right: w[rightActive], left: w[2 + leftActive], head: w[4], body: w[5] };
}

/** How the player's template maps body-frame meters onto its rest pose. */
export interface RigFrame {
  /** Body-frame meters to template space (the model's own units, feet at its origin). */
  bodyToTemplate: Matrix4;
  /** A bone of the template (at rest) by Wayfarer bone name. */
  restBone(name: string): Object3D;
  /** The same bone on this instance. */
  bone(name: string): Object3D;
  /** Every mesh material of this instance, to tint. */
  materials(): Material[];
}

/** Angle between a held weapon and the forearm's continuation, toward the player's front. */
const HOLD_TILT = 0.95;
/** Roll of a held weapon about its own blade axis, radians. */
const GRIP_ROLL = Math.PI / 2;
/** Twist of a shield or bow about its upright axis, radians (the hand turns as the arm hangs). */
const SHIELD_ROLL = Math.PI / 6;

const tmpA = new Vector3();
const tmpB = new Vector3();

/** The grip's frame in template space for a hand: blade up-and-forward from the fist. */
function gripBasis(frame: RigFrame, side: 'L' | 'R', cls: string, unitsPerMeter: number): Matrix4 {
  const hand = frame.restBone(`Hand.${side}`).getWorldPosition(tmpA).clone();
  const fore = frame.restBone(`LowerArm.${side}`).getWorldPosition(tmpB).clone();
  if (cls === 'Shield' || cls === 'Bow') {
    // The idle arm hangs from a T-pose by a roll about the front axis, so what stands along the
    // arm's reverse at rest stands upright when the arm hangs, and the face keeps looking
    // forward (a roll about Z doesn't turn Z).
    const front = new Vector3(0, 0, 1);
    const y = hand.clone().sub(fore).normalize().multiplyScalar(-1);
    const z = front.clone().addScaledVector(y, -front.dot(y)).normalize();
    const x = new Vector3().crossVectors(y, z).normalize();
    // Held out from the hip and a little forward, so it doesn't sink into the torso.
    hand.x += (side === 'L' ? 1 : -1) * 0.1 * unitsPerMeter;
    hand.z += 0.1 * unitsPerMeter;
    return new Matrix4().makeBasis(x, y, z).multiply(new Matrix4().makeRotationY(SHIELD_ROLL)).setPosition(hand);
  }
  const d = hand.sub(fore).normalize();
  // The blade runs back along the arm and out toward the front, so a hanging arm holds it
  // up and forward.
  const y = d.clone().multiplyScalar(-Math.cos(HOLD_TILT)).add(new Vector3(0, 0, Math.sin(HOLD_TILT))).normalize();
  const up = new Vector3(0, 1, 0);
  const z = up.clone().sub(y.clone().multiplyScalar(up.dot(y)));
  if (z.lengthSq() < 1e-4) z.set(0, 0, 1);
  z.normalize();
  const x = new Vector3().crossVectors(y, z).normalize();
  // A quarter turn about the blade, so its flat faces the arm's outer side rather than edge-on.
  const m = new Matrix4().makeBasis(x, y, z).multiply(new Matrix4().makeRotationY(GRIP_ROLL));
  const gp = frame.restBone(`Hand.${side}`).getWorldPosition(new Vector3());
  m.setPosition(gp);
  return m;
}

/** Attaches `object` (laid out in `worldFromObject`, template space) to `bone` at its rest pose. */
function attachAtRest(object: Object3D, bone: Object3D, restBone: Object3D, templateFromObject: Matrix4): void {
  restBone.updateWorldMatrix(true, false);
  const local = restBone.matrixWorld.clone().invert().multiply(templateFromObject);
  local.decompose(object.position, object.quaternion, object.scale);
  bone.add(object);
}

export class PlayerGear {
  private weapons: Array<{ spec: WeaponSpec; model: WeaponModel }> = [];
  private armor: ArmorPieceModel[] = [];
  private attached: Object3D[] = [];
  private tinted = new Map<Material, { color: Color }>();
  private sig = '';
  /** The off-hand item's holder, hidden while two-handing. */
  private offHand: Object3D | null = null;
  private offHandStowed = false;

  constructor(private fx: SpellFx | null = null) {}

  /** Redraws what the player wears if it changed since the last call. */
  apply(eq: Equipped, frame: RigFrame): void {
    const sig = `${eq.right},${eq.left},${eq.head},${eq.body}`;
    if (sig === this.sig) return;
    this.sig = sig;
    this.clear();

    const hands: Array<['L' | 'R', number]> = [
      ['R', eq.right],
      ['L', eq.left]
    ];
    for (const [side, num] of hands) {
      const spec = weaponByNum(num);
      if (!spec) continue;
      const model = buildWeapon(spec);
      const holder = new Object3D();
      holder.add(model.group);
      // meters -> template units, applied on top of the grip's orientation.
      const unit = new Vector3();
      frame.bodyToTemplate.decompose(new Vector3(), new Quaternion(), unit);
      const basis = gripBasis(frame, side, spec.class, unit.x);
      const toUnits = new Matrix4().makeScale(unit.x, unit.y, unit.z);
      attachAtRest(holder, frame.bone(`Hand.${side}`), frame.restBone(`Hand.${side}`), basis.multiply(toUnits));
      this.attached.push(holder);
      if (side === 'L') {
        this.offHand = holder;
        holder.visible = !this.offHandStowed;
      }
      this.weapons.push({ spec, model });
      if (model.castPoint && spec.school) this.fx?.track(model.group, model.castPoint, spec.school);
    }

    for (const num of [eq.head, eq.body]) {
      const spec = armorByNumber(num);
      if (!spec) continue;
      const piece = buildArmorPiece(spec.id);
      if (!piece) continue;
      this.armor.push(piece);
      for (const part of piece.parts) {
        attachAtRest(part.object, frame.bone(part.bone), frame.restBone(part.bone), frame.bodyToTemplate.clone());
        this.attached.push(part.object);
      }
      if (piece.tint) this.tint(frame, piece.tint);
    }
  }

  /**
   * Hides the off-hand item while the right weapon is two-handed: the off hand is on the
   * grip, so a shield or second weapon there would float. Cheap to call every frame.
   */
  setOffHandStowed(stowed: boolean): void {
    this.offHandStowed = stowed;
    if (this.offHand) this.offHand.visible = !stowed;
  }

  /** Recolors the cloth materials (`Green`, `LightGreen`) on this instance. */
  private tint(frame: RigFrame, t: { primary: number; secondary: number }): void {
    for (const m of frame.materials()) {
      const key = (m as Material & { color?: Color }).color;
      if (!key) continue;
      const slot = m.name === 'Green' ? t.primary : m.name === 'LightGreen' ? t.secondary : null;
      if (slot === null) continue;
      if (!this.tinted.has(m)) this.tinted.set(m, { color: key.clone() });
      key.set(slot);
    }
  }

  private clear(): void {
    for (const [m, o] of this.tinted) (m as Material & { color: Color }).color.copy(o.color);
    this.tinted.clear();
    for (const o of this.attached) {
      o.removeFromParent();
    }
    this.attached = [];
    this.offHand = null;
    for (const w of this.weapons) {
      this.fx?.untrack(w.model.group);
      w.model.dispose();
    }
    this.weapons = [];
    for (const a of this.armor) a.dispose();
    this.armor = [];
  }

  /** The player's weapon models, for effects that need a tip (muzzles, trails). */
  heldWeapons(): ReadonlyArray<{ spec: WeaponSpec; model: WeaponModel }> {
    return this.weapons;
  }

  dispose(): void {
    this.clear();
    this.sig = '';
  }
}
