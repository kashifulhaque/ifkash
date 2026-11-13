// B05, the Ossuary Keeper (WP-073): the model definitions and the registry numbers they draw
// (`game/data/registry.ron`, 34000 block). The snapshot's `kind` is the enemy definition's number.

export { ossuaryKeeperDef } from './keeper';
export { boneThrallDef, vaultDrownedDef } from './adds';

/** The Keeper's body: `b05_ossuary_keeper_body`. */
export const B05_KIND = 34001;
/** The three bone thralls (`b05_bone_thrall_a` to `_c`). */
export const B05_THRALL_KINDS = [34010, 34011, 34012];
/** The two Drowned (`b05_vault_drowned_a` and `_b`). */
export const B05_DROWNED_KINDS = [34020, 34021];
