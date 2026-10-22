/**
 * Browser chrome color for each design and theme.
 * Keep these hex values in sync with the inline script in `src/app.html`.
 */
export type Theme = 'light' | 'dark';
export type Design = 'nordisk' | 'broadsheet';

export const THEME_COLORS: Record<Design, Record<Theme, string>> = {
	nordisk: { light: '#f4f1ea', dark: '#15181b' },
	broadsheet: { light: '#f3ecdf', dark: '#14110e' },
};

export function readDesign(): Design {
	return document.documentElement.getAttribute('data-design') === 'broadsheet'
		? 'broadsheet'
		: 'nordisk';
}

export function readTheme(): Theme {
	return document.documentElement.getAttribute('data-theme') === 'dark' ? 'dark' : 'light';
}

export function applyThemeColor(design: Design, theme: Theme): void {
	const meta = document.querySelector('meta[name="theme-color"]');
	if (meta) meta.setAttribute('content', THEME_COLORS[design][theme]);
}
