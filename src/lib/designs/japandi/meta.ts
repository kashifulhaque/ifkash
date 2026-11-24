import type { DesignMeta } from '../shared';

export const meta: DesignMeta<'japandi'> = {
	id: 'japandi',
	label: 'Japandi',
	// Keep these in sync with the color table in `src/app.html`.
	themeColors: { light: '#f3efe7', dark: '#1b1b1a' },
	themeCopy: {
		toLight: 'Switch to washi',
		toDark: 'Switch to charcoal',
		light: 'Washi',
		dark: 'Charcoal',
	},
};
