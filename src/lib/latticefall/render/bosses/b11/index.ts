// B11, the Bound Exile (WP-079): two procedural models, the real Exile and its image. The
// snapshot's `kind` is the actor's enemy number (game/data/registry.ron), so each maps in
// `procedural/registry.ts`.

export { exileDef, exileImageDef } from './exile';

/** Registry numbers of the two actors: `b11_exile` and `b11_exile_image`. */
export const B11_EXILE_KIND = 40001;
export const B11_IMAGE_KIND = 40002;
