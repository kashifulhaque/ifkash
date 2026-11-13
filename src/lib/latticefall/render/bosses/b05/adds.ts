// B05's adds (registry 34010-34012 the bone thralls, 34020-34021 the Drowned). They reuse the
// Whitewarden Skeleton's and the Drowned's bodies and idles, with choreography that keys off B05's
// own move names and strike frames (game/data/moves/bosses/b05.ron).

import type { ModelDef } from '../../procedural/model';
import { attack, type Choreo } from '../../procedural/pose';
import { drownedDef } from '../../procedural/models/drowned';
import { skeletonDef } from '../../procedural/models/skeleton';

const slash: Choreo = {
  total: 60,
  keys: (total) =>
    attack(22, 25, total, {
      wind: { shoulderR: [-1.4, 0, -1.5], elbowR: [-0.7], spine: [0, -0.8, 0], head: [0, 0.3, 0] },
      strike: { shoulderR: [-1.5, 0, 0.5], elbowR: [-0.1], spine: [0.15, 0.7, 0], hips: [0, 0.3, 0, 0, 0, 0.08] }
    })
};

const thrust: Choreo = {
  total: 76,
  keys: (total) =>
    attack(28, 31, total, {
      wind: { shoulderR: [-0.9, 0.2, -0.6], elbowR: [-2.2], spine: [-0.05, -0.7, 0], hipL: [0.4], hipR: [-0.3], kneeR: [0.5] },
      strike: {
        shoulderR: [-1.55, 0, -0.05],
        elbowR: [-0.05],
        spine: [0.3, 0.5, 0],
        hips: [0, 0, 0, 0, -0.14, 0.4],
        hipL: [-0.8],
        hipR: [0.6],
        kneeR: [0.5]
      }
    })
};

/** A bone thrall: the skeleton's body with B05's two strikes. */
export const boneThrallDef: ModelDef = {
  ...skeletonDef,
  id: 'b05_bone_thrall',
  moves: { thrall_slash: slash, thrall_thrust: thrust },
  fallback: slash
};

const swipe: Choreo = {
  total: 76,
  keys: (total) =>
    attack(30, 34, total, {
      wind: { shoulderR: [-1.4, 0, -1.6], elbowR: [-0.4], spine: [0, -0.7, 0], head: [0, 0.3, 0] },
      strike: { shoulderR: [-1.4, 0, 0.6], elbowR: [-0.1], spine: [0.3, 0.8, 0], hips: [0, 0.25, 0, 0, 0, 0.1] }
    })
};

/** One of the vault's Drowned: the Drowned's body with B05's slow swipe. */
export const vaultDrownedDef: ModelDef = {
  ...drownedDef,
  id: 'b05_vault_drowned',
  moves: { drowned_swipe: swipe },
  fallback: swipe
};
