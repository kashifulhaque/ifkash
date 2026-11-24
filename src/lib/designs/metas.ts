/**
 * Every design's metadata, with no component imports. Stores, the theme
 * toggle, and anything that runs near `app.html` import from here.
 * Components live in `registry.ts`.
 *
 * To add a design, import its `meta.ts` and add it to `DESIGN_LIST`.
 */
import { meta as almanac } from './almanac/meta';
import { meta as japandi } from './japandi/meta';
import { meta as ember } from './ember/meta';
import type { DesignMeta, Theme } from './shared';

export const DESIGN_LIST = [almanac, japandi, ember] as const;

export type Design = (typeof DESIGN_LIST)[number]['id'];

export const DEFAULT_DESIGN: Design = 'almanac';

export const DESIGN_METAS: Record<Design, DesignMeta<Design>> = {
	almanac,
	japandi,
	ember,
};

/** The list the design dropdown shows. */
export const DESIGNS: { id: Design; label: string }[] = DESIGN_LIST.map((m) => ({
	id: m.id,
	label: m.label,
}));

/** True for a current design id. A removed or unknown id returns false. */
export function isDesign(value: string | null | undefined): value is Design {
	return !!value && value in DESIGN_METAS;
}

export function themeColor(design: Design, theme: Theme): string {
	return DESIGN_METAS[design].themeColors[theme];
}
