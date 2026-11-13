// Which procedural model draws which actor: the snapshot's `kind` is the enemy definition's
// registry number (game/data/registry.ron), so this table is the model lookup of
// docs/game/07-presentation.md.

import { colossusDef } from '../bosses/b08/colossus';
import { colossusLegDef } from '../bosses/b08/legs';
import type { ModelDef } from './model';
import { acolyteDef } from './models/acolyte';
import { bellWardenDef } from './models/bellWarden';
import { brambleBruteDef } from './models/brambleBrute';
import { caveMothDef } from './models/caveMoth';
import { choirKnightDef } from './models/choirKnight';
import { drownedDef } from './models/drowned';
import { foundrySmithDef } from './models/foundrySmith';
import { glasswingStalkerDef } from './models/glasswingStalker';
import { grayCaptainDef } from './models/grayCaptain';
import { grayArcherDef } from './models/grayArcher';
import { grayKnightDef } from './models/grayKnight';
import { hollowSentryDef } from './models/hollowSentry';
import { huskDef } from './models/husk';
import { lampMiteDef } from './models/lampMite';
import { lanternThiefDef } from './models/lanternThief';
import { marshCrawlerDef } from './models/marshCrawler';
import { mimicChestDef } from './models/mimicChest';
import { reedStalkerDef } from './models/reedStalker';
import { rimeWolfDef } from './models/rimeWolf';
import { shardGolemDef } from './models/shardGolem';
import { skeletonDef } from './models/skeleton';
import { thornHoundDef } from './models/thornHound';
import { corvaneDef } from '../bosses/b03/corvane';
import { B06_BELLBEARER_KIND, B06_CANTOR_KIND, B06_LAMENTER_KIND, bellbearerDef, cantorDef, lamenterDef } from '../bosses/b06';

/**
 * Registry number of B01's body: `b01_hollow_sentry_body`. The boss definition
 * (`b01_hollow_sentry`, 9010) isn't an actor; the snapshot carries the body's enemy number.
 */
export const B01_KIND = 9011;

/** Registry number of B03's body: `b03_bridgewarden_corvane_body` (the boss is 32010). */
export const B03_KIND = 32011;
/** Registry number of B08's body: `b08_forgeheart_colossus_body`. Its legs are the next two. */
export const B08_KIND = 37011;

const defs = new Map<number, ModelDef>([
  [7000, huskDef('husk')],
  [7001, huskDef('servant')],
  [7002, huskDef('miner')],
  [7003, huskDef('frost')],
  [7010, thornHoundDef],
  [7020, grayKnightDef],
  [7030, grayArcherDef],
  [7040, skeletonDef],
  // WP-036 batch 2 (7050-7058).
  [7050, acolyteDef('acolyte')],
  [7051, drownedDef],
  [7052, reedStalkerDef],
  [7053, marshCrawlerDef],
  [7054, caveMothDef],
  [7055, lampMiteDef],
  [7056, shardGolemDef],
  [7057, glasswingStalkerDef],
  [7058, acolyteDef('bell')],
  // WP-037 batch 3 and the elites (7080-7087).
  [7080, lanternThiefDef],
  [7081, rimeWolfDef],
  [7082, mimicChestDef],
  [7083, brambleBruteDef],
  [7084, choirKnightDef],
  [7085, grayCaptainDef],
  [7086, foundrySmithDef],
  [7087, bellWardenDef],
  [B01_KIND, hollowSentryDef],
  [B03_KIND, corvaneDef],
  // WP-076: B08, the Forgeheart Colossus: its body, and its two legs (37010-37013).
  [B08_KIND, colossusDef],
  [B08_KIND + 1, colossusLegDef],
  [B08_KIND + 2, colossusLegDef]
]);

// WP-074: B06, the Drowned Choir (three bodies).
defs.set(B06_CANTOR_KIND, cantorDef);
defs.set(B06_BELLBEARER_KIND, bellbearerDef);
defs.set(B06_LAMENTER_KIND, lamenterDef);

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
