// What the renderer knows about gear, read from the generated UI digest (`gen/digest.json`)
// so no RON is parsed in the browser. Weapons and armor are looked up by registry number
// (what the menu state holds in the equipment slots) or by string id.

import digest from '../../gen/digest.json';
import type { WeaponClass } from '../../gen/WeaponClass';
import type { WeaponSpec } from './weapons';

interface DigestWeapon {
  id: string;
  num: number;
  name: string;
  class: string;
  unique: boolean;
  stub: boolean;
  weightTenths: number;
  catalystSchool: string | null;
  element: string | null;
}

interface DigestArmor {
  id: string;
  num: number;
  name: string;
  set: string;
  slot: string;
  weightTenths: number;
}

interface DigestSpell {
  id: string;
  num: number;
  name: string;
  school: string;
}

export interface ArmorSpec {
  id: string;
  num: number;
  name: string;
  /** The set's name; empty for a relic piece. */
  set: string;
  slot: 'Head' | 'Body';
}

const toSpec = (w: DigestWeapon): WeaponSpec => ({
  id: w.id,
  name: w.name,
  class: w.class as WeaponClass,
  unique: w.unique,
  element: w.element,
  school: w.catalystSchool,
  weightTenths: w.weightTenths
});

const d = digest as unknown as { weapons: DigestWeapon[]; armor?: DigestArmor[]; spells?: DigestSpell[] };

/** Every weapon, in content order. */
export const WEAPONS: WeaponSpec[] = d.weapons.map(toSpec);
const byNum = new Map<number, WeaponSpec>(d.weapons.map((w) => [w.num, toSpec(w)]));

export const ARMOR: ArmorSpec[] = (d.armor ?? []).map((a) => ({
  id: a.id,
  num: a.num,
  name: a.name,
  set: a.set,
  slot: a.slot === 'Head' ? 'Head' : 'Body'
}));
const armorByNum = new Map<number, ArmorSpec>(ARMOR.map((a) => [a.num, a]));

export const SPELLS: Array<{ id: string; name: string; school: string }> = d.spells ?? [];

export const weaponByNum = (n: number): WeaponSpec | undefined => byNum.get(n);
export const armorByNumber = (n: number): ArmorSpec | undefined => armorByNum.get(n);

/** Classes in the order the armory shows them. */
export const CLASS_ORDER: WeaponClass[] = [
  'Dagger',
  'StraightSword',
  'CurvedSword',
  'ThrustingSword',
  'Greatsword',
  'Axe',
  'Hammer',
  'Colossal',
  'Spear',
  'Halberd',
  'Fist',
  'Bow',
  'Shield',
  'Catalyst'
];
