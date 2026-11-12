// Which procedural model draws which actor: the snapshot's `kind` is the enemy definition's
// registry number (game/data/registry.ron), so this table is the model lookup of
// docs/game/07-presentation.md.

import type { ModelDef } from './model';
import { grayArcherDef } from './models/grayArcher';
import { grayKnightDef } from './models/grayKnight';
import { hollowSentryDef } from './models/hollowSentry';
import { huskDef } from './models/husk';
import { skeletonDef } from './models/skeleton';
import { thornHoundDef } from './models/thornHound';

/** Registry number of the B01 enemy definition (block 9010-9019 is reserved for B01). */
export const B01_KIND = 9010;

const defs = new Map<number, ModelDef>([
  [7000, huskDef('husk')],
  [7001, huskDef('servant')],
  [7002, huskDef('miner')],
  [7003, huskDef('frost')],
  [7010, thornHoundDef],
  [7020, grayKnightDef],
  [7030, grayArcherDef],
  [7040, skeletonDef],
  [B01_KIND, hollowSentryDef]
]);

/** The model for an actor kind, or undefined for kinds drawn some other way (the player). */
export function modelForKind(kind: number): ModelDef | undefined {
  return defs.get(kind);
}

/** The model for a content id (`husk_frost`, `b01_hollow_sentry`), for the dev preview. */
export function modelForId(id: string): ModelDef | undefined {
  for (const d of defs.values()) if (d.id === id) return d;
  return undefined;
}

export function modelIds(): string[] {
  return [...defs.values()].map((d) => d.id);
}
