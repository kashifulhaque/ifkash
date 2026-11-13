// B12, the Unspooled (WP-080): its model (the Weaver, then the Unspooled) and its loom lines.
// The snapshot's `kind` is the actor's enemy number (game/data/registry.ron), so the model maps
// in `procedural/registry.ts`; the loom effects hang off the engine's event stream.

export { B12_KIND, resetUnspooled, unspooledDef } from './unspooled';
export { LoomFx } from './loomFx';
