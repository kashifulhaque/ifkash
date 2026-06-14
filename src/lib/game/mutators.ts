// Daily-challenge mutators. Each day's seed picks a deterministic subset; the
// chosen mutators fold into a single `MutatorEnv` that the NpcManager (and the
// scoring path) read. Effects map only onto knobs the enemy sim already owns,
// so a daily run differs in feel without touching world generation.

export type MutatorEnv = {
  damageMult: number; // enemy shot damage scaling
  scoreMult: number; // score scaling (riskier days pay more)
  hpMult: number; // enemy toughness scaling
  popCapBonus: number; // +live population cap
  rusherBias: number; // added probability mass toward rushers
  snipersFromWave: number; // earliest wave snipers appear (default 3)
};

export type Mutator = {
  id: string;
  label: string;
  desc: string;
  apply(env: MutatorEnv): void;
};

export function defaultMutatorEnv(): MutatorEnv {
  return {
    damageMult: 1,
    scoreMult: 1,
    hpMult: 1,
    popCapBonus: 0,
    rusherBias: 0,
    snipersFromWave: 3
  };
}

export const ALL_MUTATORS: Mutator[] = [
  {
    id: 'glass-cannon',
    label: 'GLASS CANNON',
    desc: 'Enemies hit 60% harder — but every kill is worth 50% more.',
    apply: (e) => {
      e.damageMult *= 1.6;
      e.scoreMult *= 1.5;
    }
  },
  {
    id: 'swarm',
    label: 'SWARM',
    desc: 'More enemies on the field, heavily weighted toward rushers.',
    apply: (e) => {
      e.popCapBonus += 4;
      e.rusherBias += 0.25;
    }
  },
  {
    id: 'marksman',
    label: 'MARKSMAN',
    desc: 'Snipers show up from wave 1 and shots sting a little more.',
    apply: (e) => {
      e.snipersFromWave = 1;
      e.damageMult *= 1.15;
    }
  },
  {
    id: 'juggernaut',
    label: 'JUGGERNAUT',
    desc: 'Enemies are 50% tougher — but worth 40% more points.',
    apply: (e) => {
      e.hpMult *= 1.5;
      e.scoreMult *= 1.4;
    }
  },
  {
    id: 'bounty',
    label: 'BOUNTY',
    desc: 'Double points all run. Make it count.',
    apply: (e) => {
      e.scoreMult *= 2;
    }
  }
];

/** Deterministically pick `count` distinct mutators from a seeded stream. */
export function pickDailyMutators(rng: () => number, count = 2): Mutator[] {
  const pool = [...ALL_MUTATORS];
  const out: Mutator[] = [];
  for (let i = 0; i < count && pool.length; i++) {
    const idx = Math.floor(rng() * pool.length);
    out.push(pool.splice(idx, 1)[0]);
  }
  return out;
}

/** Fold a list of mutators into a single env. */
export function buildMutatorEnv(mutators: Mutator[]): MutatorEnv {
  const env = defaultMutatorEnv();
  for (const m of mutators) m.apply(env);
  return env;
}
