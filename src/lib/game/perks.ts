// Between-wave perk picks. On each wave clear the player chooses 1 of 3; the
// chosen perk applies to per-run tuning knobs the Game exposes via PerkContext.
// Perks are kept loosely coupled to Game through this interface so the two
// modules don't import each other's implementations.

export interface PerkContext {
  addMagSize(n: number): void;
  scaleReload(factor: number): void;
  healFull(): void;
  addComboWindow(seconds: number): void;
  addHeadshotBonus(frac: number): void;
  addDropChance(frac: number): void;
}

export type Perk = {
  id: string;
  label: string;
  desc: string;
  apply(ctx: PerkContext): void;
};

export const ALL_PERKS: Perk[] = [
  {
    id: 'quick-hands',
    label: 'QUICK HANDS',
    desc: 'Reload 25% faster.',
    apply: (c) => c.scaleReload(0.75)
  },
  {
    id: 'extended-mag',
    label: 'EXTENDED MAG',
    desc: '+4 rounds per magazine.',
    apply: (c) => c.addMagSize(4)
  },
  {
    id: 'combat-medic',
    label: 'COMBAT MEDIC',
    desc: 'Patch up to full health right now.',
    apply: (c) => c.healFull()
  },
  {
    id: 'adrenaline',
    label: 'ADRENALINE',
    desc: 'Combo window lasts 1.5s longer.',
    apply: (c) => c.addComboWindow(1.5)
  },
  {
    id: 'deadeye',
    label: 'DEADEYE',
    desc: 'Headshots score even higher.',
    apply: (c) => c.addHeadshotBonus(0.5)
  },
  {
    id: 'scavenger',
    label: 'SCAVENGER',
    desc: 'Enemies drop ammo far more often.',
    apply: (c) => c.addDropChance(0.2)
  }
];

/** Pick `count` distinct perks from a (possibly seeded) random stream. */
export function offerPerks(rng: () => number = Math.random, count = 3): Perk[] {
  const pool = [...ALL_PERKS];
  const out: Perk[] = [];
  for (let i = 0; i < count && pool.length; i++) {
    const idx = Math.floor(rng() * pool.length);
    out.push(pool.splice(idx, 1)[0]);
  }
  return out;
}
