/**
 * Maps each design id to its metadata and components. The root layout and
 * the home route read from here. To add a design, create its folder (see
 * DESIGN.md), then add it here and in `metas.ts`.
 */
import type { ComponentType, SvelteComponent } from 'svelte';
import { DESIGNS, DESIGN_METAS, type Design } from './metas';
import type { DesignMeta } from './shared';

import AlmanacShell from './almanac/Shell.svelte';
import AlmanacHome from './almanac/Home.svelte';
import JapandiShell from './japandi/Shell.svelte';
import JapandiHome from './japandi/Home.svelte';
import EmberShell from './ember/Shell.svelte';
import EmberHome from './ember/Home.svelte';

/** Props every Shell receives. A Shell also renders a default `<slot />`. */
export type ShellComponent = ComponentType<SvelteComponent<{ fullWidth?: boolean }>>;
/** A Home takes no props. */
export type HomeComponent = ComponentType<SvelteComponent>;

export interface DesignEntry {
	meta: DesignMeta<Design>;
	Shell: ShellComponent;
	Home: HomeComponent;
}

export const registry: Record<Design, DesignEntry> = {
	almanac: { meta: DESIGN_METAS.almanac, Shell: AlmanacShell, Home: AlmanacHome },
	japandi: { meta: DESIGN_METAS.japandi, Shell: JapandiShell, Home: JapandiHome },
	ember: { meta: DESIGN_METAS.ember, Shell: EmberShell, Home: EmberHome },
};

export { DESIGNS };
