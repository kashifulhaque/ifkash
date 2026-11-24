/**
 * Browser chrome color for each design and theme. The values come from
 * each design's `meta.ts`. Keep them in sync with the inline script in
 * `src/app.html`. This file must not import Svelte components.
 */
import { DEFAULT_DESIGN, DESIGN_METAS, isDesign, themeColor, type Design } from '$lib/designs/metas';
import type { Theme } from '$lib/designs/shared';

export type { Design, Theme };
export { isDesign };

export const THEME_COLORS: Record<Design, Record<Theme, string>> = {
	almanac: DESIGN_METAS.almanac.themeColors,
	japandi: DESIGN_METAS.japandi.themeColors,
	ember: DESIGN_METAS.ember.themeColors,
};

export function readDesign(): Design {
	const value = document.documentElement.getAttribute('data-design');
	return isDesign(value) ? value : DEFAULT_DESIGN;
}

export function readTheme(): Theme {
	return document.documentElement.getAttribute('data-theme') === 'dark' ? 'dark' : 'light';
}

export function applyThemeColor(design: Design, theme: Theme): void {
	const meta = document.querySelector('meta[name="theme-color"]');
	if (meta) meta.setAttribute('content', themeColor(design, theme));
}
