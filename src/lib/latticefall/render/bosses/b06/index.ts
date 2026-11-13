// B06, the Drowned Choir (WP-074): three procedural models, one per member. The snapshot's
// `kind` is the member's enemy number (game/data/registry.ron), so each maps in
// `procedural/registry.ts`.

export { bellbearerDef } from './bellbearer';
export { cantorDef } from './cantor';
export { lamenterDef } from './lamenter';

/** Registry numbers of the three members: `b06_cantor`, `b06_bellbearer`, `b06_lamenter`. */
export const B06_CANTOR_KIND = 35001;
export const B06_BELLBEARER_KIND = 35002;
export const B06_LAMENTER_KIND = 35003;
