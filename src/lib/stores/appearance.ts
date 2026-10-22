/**
 * Browser chrome color for each design and theme.
 * Keep these hex values in sync with the inline script in `src/app.html`.
 */
export type Theme = 'light' | 'dark';
export type Design = 'nordisk' | 'broadsheet' | 'atelier';

export const THEME_COLORS: Record<Design, Record<Theme, string>> = {
	nordisk: { light: '#f4f1ea', dark: '#15181b' },
	broadsheet: { light: '#f3ecdf', dark: '#14110e' },
	atelier: { light: '#eef2ea', dark: '#121614' },
};

export function isDesign(value: string | null): value is Design {
	return value === 'nordisk' || value === 'broadsheet' || value === 'atelier';
}

export function readDesign(): Design {
	const value = document.documentElement.getAttribute('data-design');
	return isDesign(value) ? value : 'nordisk';
}

export function readTheme(): Theme {
	return document.documentElement.getAttribute('data-theme') === 'dark' ? 'dark' : 'light';
}

export function applyThemeColor(design: Design, theme: Theme): void {
	const meta = document.querySelector('meta[name="theme-color"]');
	if (meta) meta.setAttribute('content', THEME_COLORS[design][theme]);
}
